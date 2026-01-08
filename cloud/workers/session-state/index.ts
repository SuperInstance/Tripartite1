// SuperInstance Session State - Durable Object
// Maintains conversation history and context for cloud sessions
//
// This Durable Object stores conversation state, allowing
// seamless continuation of conversations across requests.

import { DurableObject } from 'cloudflare:workers';

interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp: number;
}

interface ConversationState {
  session_id: string;
  user_id: string;
  title?: string;
  messages: Message[];
  context: {
    pathos_framing?: string;
    local_knowledge?: string[];
    constraints?: string[];
  };
  model_preference?: string;
  created_at: number;
  updated_at: number;
}

interface SessionSummary {
  session_id: string;
  title: string;
  message_count: number;
  created_at: number;
  updated_at: number;
}

export class SessionState implements DurableObject {
  private state: DurableObjectState;
  private env: Env;
  private session: ConversationState | null = null;

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.env = env;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);

    // Ensure session is loaded
    await this.ensureSessionLoaded();

    switch (url.pathname) {
      case '/get':
        return this.handleGet();
      case '/add-message':
        return this.handleAddMessage(request);
      case '/update-context':
        return this.handleUpdateContext(request);
      case '/list':
        return this.handleList();
      case '/delete':
        return this.handleDelete();
      case '/clear':
        return this.handleClear();
      default:
        return new Response('Not Found', { status: 404 });
    }
  }

  private async ensureSessionLoaded(): Promise<void> {
    if (this.session === null) {
      const stored = await this.state.storage.get<ConversationState>('session');
      this.session = stored || null;
    }
  }

  private async handleGet(): Promise<Response> {
    if (!this.session) {
      return new Response(JSON.stringify({ error: 'Session not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    return new Response(JSON.stringify(this.session), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleAddMessage(request: Request): Promise<Response> {
    const { role, content } = await request.json() as {
      role: 'user' | 'assistant';
      content: string;
    };

    if (!this.session) {
      // Create new session
      this.session = {
        session_id: this.id.toString(),
        user_id: '', // Will be set by router
        messages: [],
        context: {},
        created_at: Date.now(),
        updated_at: Date.now(),
      };
    }

    // Add message
    const message: Message = {
      role,
      content,
      timestamp: Date.now(),
    };

    this.session.messages.push(message);
    this.session.updated_at = Date.now();

    // Auto-generate title from first user message
    if (this.session.messages.length === 1 && role === 'user') {
      this.session.title = content.slice(0, 50) + (content.length > 50 ? '...' : '');
    }

    // Persist to storage
    await this.state.storage.put('session', this.session);

    return new Response(JSON.stringify({
      success: true,
      message,
      title: this.session.title,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleUpdateContext(request: Request): Promise<Response> {
    if (!this.session) {
      return new Response(JSON.stringify({ error: 'Session not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    const updates = await request.json() as Partial<ConversationState['context']>;

    // Update context
    if (updates.pathos_framing !== undefined) {
      this.session.context.pathos_framing = updates.pathos_framing;
    }

    if (updates.local_knowledge !== undefined) {
      this.session.context.local_knowledge = updates.local_knowledge;
    }

    if (updates.constraints !== undefined) {
      this.session.context.constraints = updates.constraints;
    }

    this.session.updated_at = Date.now();

    // Persist
    await this.state.storage.put('session', this.session);

    return new Response(JSON.stringify({
      success: true,
      context: this.session.context,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleList(): Promise<Response> {
    if (!this.session) {
      return new Response(JSON.stringify({ sessions: [] }), {
        headers: { 'Content-Type': 'application/json' },
      });
    }

    const summary: SessionSummary = {
      session_id: this.session.session_id,
      title: this.session.title || 'Untitled',
      message_count: this.session.messages.length,
      created_at: this.session.created_at,
      updated_at: this.session.updated_at,
    };

    return new Response(JSON.stringify({ sessions: [summary] }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleDelete(): Promise<Response> {
    if (!this.session) {
      return new Response(JSON.stringify({ error: 'Session not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    // Delete session
    await this.state.storage.delete('session');
    this.session = null;

    return new Response(JSON.stringify({ success: true }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleClear(): Promise<Response> {
    if (!this.session) {
      return new Response(JSON.stringify({ error: 'Session not found' }), {
        status: 404,
        headers: { 'Content-Type': 'application/json' },
      });
    }

    // Clear messages but keep session
    this.session.messages = [];
    this.session.updated_at = Date.now();

    await this.state.storage.put('session', this.session);

    return new Response(JSON.stringify({
      success: true,
      message_count: 0,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }
}

// Environment interface
interface Env {
  // D1 Database for persistent storage
  DB: D1Database;
  // Additional bindings can be added here
}

// Export for wrangler.toml
export default {
  async fetch() {
    return new Response('Session State Durable Object');
  },
};
