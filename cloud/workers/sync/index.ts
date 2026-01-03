/**
 * SuperInstance AI - Sync Worker
 * 
 * Handles synchronization between local client and cloud:
 * - Settings sync
 * - Knowledge vault sync
 * - LoRA sync
 * - Session history sync
 */

import { DurableObject } from 'cloudflare:workers';

interface Env {
  SYNC_STATE: DurableObjectNamespace;
  KNOWLEDGE_BUCKET: R2Bucket;
  LORA_BUCKET: R2Bucket;
  DB: D1Database;
}

interface SyncRequest {
  userId: string;
  type: 'settings' | 'knowledge' | 'lora' | 'history';
  direction: 'push' | 'pull';
  data?: any;
  lastSyncTimestamp?: number;
}

interface SyncResponse {
  success: boolean;
  data?: any;
  timestamp: number;
  conflicts?: SyncConflict[];
}

interface SyncConflict {
  key: string;
  localValue: any;
  remoteValue: any;
  localTimestamp: number;
  remoteTimestamp: number;
}

// Main worker
export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    
    // CORS headers
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // Authenticate request
      const authHeader = request.headers.get('Authorization');
      if (!authHeader?.startsWith('Bearer ')) {
        return new Response(JSON.stringify({ error: 'Unauthorized' }), {
          status: 401,
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        });
      }

      const token = authHeader.slice(7);
      const userId = await validateToken(token, env);
      if (!userId) {
        return new Response(JSON.stringify({ error: 'Invalid token' }), {
          status: 401,
          headers: { ...corsHeaders, 'Content-Type': 'application/json' },
        });
      }

      // Route to appropriate handler
      const path = url.pathname;

      if (path === '/sync/settings') {
        return handleSettingsSync(request, env, userId, corsHeaders);
      }

      if (path === '/sync/knowledge') {
        return handleKnowledgeSync(request, env, userId, corsHeaders);
      }

      if (path === '/sync/lora') {
        return handleLoraSync(request, env, userId, corsHeaders);
      }

      if (path === '/sync/history') {
        return handleHistorySync(request, env, userId, corsHeaders);
      }

      if (path === '/sync/status') {
        return handleSyncStatus(request, env, userId, corsHeaders);
      }

      return new Response(JSON.stringify({ error: 'Not found' }), {
        status: 404,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
      });

    } catch (error) {
      console.error('Sync error:', error);
      return new Response(JSON.stringify({ error: 'Internal server error' }), {
        status: 500,
        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
      });
    }
  },
};

// Token validation (placeholder - implement JWT validation)
async function validateToken(token: string, env: Env): Promise<string | null> {
  // TODO: Implement proper JWT validation
  // For now, assume token is userId
  return token;
}

// Settings sync handler
async function handleSettingsSync(
  request: Request,
  env: Env,
  userId: string,
  corsHeaders: Record<string, string>
): Promise<Response> {
  const syncRequest: SyncRequest = await request.json();

  if (syncRequest.direction === 'push') {
    // Store settings in D1
    await env.DB.prepare(`
      INSERT OR REPLACE INTO user_settings (user_id, settings, updated_at)
      VALUES (?, ?, ?)
    `).bind(userId, JSON.stringify(syncRequest.data), Date.now()).run();

    return new Response(JSON.stringify({
      success: true,
      timestamp: Date.now(),
    }), {
      headers: { ...corsHeaders, 'Content-Type': 'application/json' },
    });
  }

  // Pull settings
  const result = await env.DB.prepare(`
    SELECT settings, updated_at FROM user_settings WHERE user_id = ?
  `).bind(userId).first();

  return new Response(JSON.stringify({
    success: true,
    data: result ? JSON.parse(result.settings as string) : null,
    timestamp: result?.updated_at || 0,
  }), {
    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
  });
}

// Knowledge sync handler
async function handleKnowledgeSync(
  request: Request,
  env: Env,
  userId: string,
  corsHeaders: Record<string, string>
): Promise<Response> {
  const syncRequest: SyncRequest = await request.json();

  if (syncRequest.direction === 'push') {
    // Get list of documents to sync
    const documents = syncRequest.data?.documents || [];
    
    for (const doc of documents) {
      // Upload document content to R2
      const key = `${userId}/knowledge/${doc.id}`;
      await env.KNOWLEDGE_BUCKET.put(key, doc.content, {
        customMetadata: {
          filename: doc.filename,
          checksum: doc.checksum,
          updatedAt: String(Date.now()),
        },
      });

      // Store metadata in D1
      await env.DB.prepare(`
        INSERT OR REPLACE INTO knowledge_documents 
        (id, user_id, filename, checksum, r2_key, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
      `).bind(doc.id, userId, doc.filename, doc.checksum, key, Date.now()).run();
    }

    return new Response(JSON.stringify({
      success: true,
      timestamp: Date.now(),
      synced: documents.length,
    }), {
      headers: { ...corsHeaders, 'Content-Type': 'application/json' },
    });
  }

  // Pull knowledge - return list of documents updated since lastSyncTimestamp
  const lastSync = syncRequest.lastSyncTimestamp || 0;
  
  const results = await env.DB.prepare(`
    SELECT id, filename, checksum, r2_key, updated_at
    FROM knowledge_documents
    WHERE user_id = ? AND updated_at > ?
    ORDER BY updated_at DESC
    LIMIT 100
  `).bind(userId, lastSync).all();

  // Fetch content for each document
  const documents = await Promise.all(
    results.results.map(async (row) => {
      const object = await env.KNOWLEDGE_BUCKET.get(row.r2_key as string);
      return {
        id: row.id,
        filename: row.filename,
        checksum: row.checksum,
        content: object ? await object.text() : null,
        updatedAt: row.updated_at,
      };
    })
  );

  return new Response(JSON.stringify({
    success: true,
    data: { documents },
    timestamp: Date.now(),
  }), {
    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
  });
}

// LoRA sync handler
async function handleLoraSync(
  request: Request,
  env: Env,
  userId: string,
  corsHeaders: Record<string, string>
): Promise<Response> {
  const syncRequest: SyncRequest = await request.json();

  if (syncRequest.direction === 'push') {
    // Upload LoRA to R2
    const lora = syncRequest.data;
    const key = `${userId}/loras/${lora.id}`;
    
    await env.LORA_BUCKET.put(key, lora.weights, {
      customMetadata: {
        name: lora.name,
        baseModel: lora.baseModel,
        checksum: lora.checksum,
        updatedAt: String(Date.now()),
      },
    });

    // Store metadata
    await env.DB.prepare(`
      INSERT OR REPLACE INTO user_loras
      (id, user_id, name, base_model, checksum, r2_key, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?)
    `).bind(lora.id, userId, lora.name, lora.baseModel, lora.checksum, key, Date.now()).run();

    return new Response(JSON.stringify({
      success: true,
      timestamp: Date.now(),
    }), {
      headers: { ...corsHeaders, 'Content-Type': 'application/json' },
    });
  }

  // Pull LoRAs
  const lastSync = syncRequest.lastSyncTimestamp || 0;
  
  const results = await env.DB.prepare(`
    SELECT id, name, base_model, checksum, r2_key, updated_at
    FROM user_loras
    WHERE user_id = ? AND updated_at > ?
  `).bind(userId, lastSync).all();

  return new Response(JSON.stringify({
    success: true,
    data: {
      loras: results.results.map(row => ({
        id: row.id,
        name: row.name,
        baseModel: row.base_model,
        checksum: row.checksum,
        r2Key: row.r2_key,
        updatedAt: row.updated_at,
      })),
    },
    timestamp: Date.now(),
  }), {
    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
  });
}

// History sync handler
async function handleHistorySync(
  request: Request,
  env: Env,
  userId: string,
  corsHeaders: Record<string, string>
): Promise<Response> {
  const syncRequest: SyncRequest = await request.json();

  if (syncRequest.direction === 'push') {
    const sessions = syncRequest.data?.sessions || [];
    
    for (const session of sessions) {
      await env.DB.prepare(`
        INSERT OR REPLACE INTO chat_sessions
        (id, user_id, title, messages, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
      `).bind(
        session.id,
        userId,
        session.title,
        JSON.stringify(session.messages),
        session.createdAt,
        Date.now()
      ).run();
    }

    return new Response(JSON.stringify({
      success: true,
      timestamp: Date.now(),
      synced: sessions.length,
    }), {
      headers: { ...corsHeaders, 'Content-Type': 'application/json' },
    });
  }

  // Pull history
  const lastSync = syncRequest.lastSyncTimestamp || 0;
  
  const results = await env.DB.prepare(`
    SELECT id, title, messages, created_at, updated_at
    FROM chat_sessions
    WHERE user_id = ? AND updated_at > ?
    ORDER BY updated_at DESC
    LIMIT 50
  `).bind(userId, lastSync).all();

  return new Response(JSON.stringify({
    success: true,
    data: {
      sessions: results.results.map(row => ({
        id: row.id,
        title: row.title,
        messages: JSON.parse(row.messages as string),
        createdAt: row.created_at,
        updatedAt: row.updated_at,
      })),
    },
    timestamp: Date.now(),
  }), {
    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
  });
}

// Sync status handler
async function handleSyncStatus(
  request: Request,
  env: Env,
  userId: string,
  corsHeaders: Record<string, string>
): Promise<Response> {
  // Get last sync times for all types
  const settingsResult = await env.DB.prepare(`
    SELECT updated_at FROM user_settings WHERE user_id = ?
  `).bind(userId).first();

  const knowledgeResult = await env.DB.prepare(`
    SELECT MAX(updated_at) as last_updated FROM knowledge_documents WHERE user_id = ?
  `).bind(userId).first();

  const loraResult = await env.DB.prepare(`
    SELECT MAX(updated_at) as last_updated FROM user_loras WHERE user_id = ?
  `).bind(userId).first();

  const historyResult = await env.DB.prepare(`
    SELECT MAX(updated_at) as last_updated FROM chat_sessions WHERE user_id = ?
  `).bind(userId).first();

  return new Response(JSON.stringify({
    success: true,
    status: {
      settings: { lastSync: settingsResult?.updated_at || 0 },
      knowledge: { lastSync: knowledgeResult?.last_updated || 0 },
      lora: { lastSync: loraResult?.last_updated || 0 },
      history: { lastSync: historyResult?.last_updated || 0 },
    },
    timestamp: Date.now(),
  }), {
    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
  });
}

// Durable Object for managing sync state
export class SyncState extends DurableObject {
  private state: DurableObjectState;

  constructor(state: DurableObjectState, env: Env) {
    super(state, env);
    this.state = state;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    
    if (url.pathname === '/lock') {
      // Acquire sync lock
      const lockKey = url.searchParams.get('key') || 'default';
      const existing = await this.state.storage.get<number>(`lock:${lockKey}`);
      
      if (existing && Date.now() - existing < 30000) {
        return new Response(JSON.stringify({ locked: true }), {
          status: 423,
          headers: { 'Content-Type': 'application/json' },
        });
      }
      
      await this.state.storage.put(`lock:${lockKey}`, Date.now());
      return new Response(JSON.stringify({ locked: false, acquired: true }));
    }

    if (url.pathname === '/unlock') {
      const lockKey = url.searchParams.get('key') || 'default';
      await this.state.storage.delete(`lock:${lockKey}`);
      return new Response(JSON.stringify({ released: true }));
    }

    return new Response('Not found', { status: 404 });
  }
}
