// SuperInstance Cloud Router Worker
// Handles routing between local clients and cloud LLM providers
//
// Deploy: wrangler deploy

export interface Env {
  // KV Namespaces
  RATE_LIMITS: KVNamespace;
  API_KEYS: KVNamespace;
  
  // Durable Objects
  BILLING_LEDGER: DurableObjectNamespace;
  SESSION_STATE: DurableObjectNamespace;
  
  // R2 Buckets
  KNOWLEDGE_STORE: R2Bucket;
  MODEL_CACHE: R2Bucket;
  
  // D1 Database
  DB: D1Database;
  
  // Secrets
  ANTHROPIC_API_KEY: string;
  OPENAI_API_KEY: string;
  STRIPE_SECRET_KEY: string;
}

// Request from local Synesis client
interface EscalationRequest {
  // Unique request ID
  request_id: string;
  // User/device identifier
  user_id: string;
  // Session ID for conversation continuity
  session_id: string;
  // The redacted query (PII already removed locally)
  query: string;
  // Context from local processing
  context: {
    pathos_framing: string;
    local_knowledge: string[];
    conversation_history: Message[];
  };
  // Requested model preference
  model_preference?: 'claude' | 'gpt4' | 'auto';
  // Max tokens to generate
  max_tokens: number;
  // Whether to stream response
  stream: boolean;
}

interface Message {
  role: 'user' | 'assistant';
  content: string;
}

interface EscalationResponse {
  request_id: string;
  content: string;
  model_used: string;
  tokens_used: {
    prompt: number;
    completion: number;
  };
  cost_cents: number;
  latency_ms: number;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    
    // CORS preflight
    if (request.method === 'OPTIONS') {
      return handleCORS();
    }
    
    // Route based on path
    switch (url.pathname) {
      case '/v1/escalate':
        return handleEscalation(request, env, ctx);
      case '/v1/stream':
        return handleStreamingEscalation(request, env, ctx);
      case '/v1/status':
        return handleStatus(env);
      case '/v1/usage':
        return handleUsage(request, env);
      case '/health':
        return new Response('OK', { status: 200 });
      default:
        return new Response('Not Found', { status: 404 });
    }
  },
};

async function handleEscalation(
  request: Request, 
  env: Env, 
  ctx: ExecutionContext
): Promise<Response> {
  // Verify API key
  const apiKey = request.headers.get('Authorization')?.replace('Bearer ', '');
  if (!apiKey) {
    return jsonResponse({ error: 'Missing API key' }, 401);
  }
  
  const userId = await validateApiKey(apiKey, env);
  if (!userId) {
    return jsonResponse({ error: 'Invalid API key' }, 401);
  }
  
  // Check rate limits
  const rateLimitOk = await checkRateLimit(userId, env);
  if (!rateLimitOk) {
    return jsonResponse({ error: 'Rate limit exceeded' }, 429);
  }
  
  // Parse request
  const req: EscalationRequest = await request.json();
  
  // Select model
  const model = selectModel(req.model_preference);
  
  // Call cloud LLM
  const startTime = Date.now();
  let response: EscalationResponse;
  
  try {
    if (model.provider === 'anthropic') {
      response = await callAnthropic(req, model, env);
    } else {
      response = await callOpenAI(req, model, env);
    }
  } catch (error) {
    console.error('LLM call failed:', error);
    return jsonResponse({ error: 'LLM call failed' }, 500);
  }
  
  response.latency_ms = Date.now() - startTime;
  
  // Record usage in billing ledger
  ctx.waitUntil(recordUsage(userId, response, env));
  
  return jsonResponse(response);
}

async function handleStreamingEscalation(
  request: Request,
  env: Env,
  ctx: ExecutionContext
): Promise<Response> {
  // Similar to handleEscalation but returns a ReadableStream
  // for Server-Sent Events (SSE)
  
  const apiKey = request.headers.get('Authorization')?.replace('Bearer ', '');
  if (!apiKey) {
    return jsonResponse({ error: 'Missing API key' }, 401);
  }
  
  const userId = await validateApiKey(apiKey, env);
  if (!userId) {
    return jsonResponse({ error: 'Invalid API key' }, 401);
  }
  
  const req: EscalationRequest = await request.json();
  const model = selectModel(req.model_preference);
  
  // Create streaming response
  const { readable, writable } = new TransformStream();
  const writer = writable.getWriter();
  const encoder = new TextEncoder();
  
  // Start streaming in background
  ctx.waitUntil((async () => {
    try {
      // TODO: Implement actual streaming from LLM providers
      const chunks = ['Hello', ' from', ' cloud', '!'];
      
      for (const chunk of chunks) {
        await writer.write(encoder.encode(`data: ${JSON.stringify({ chunk })}\n\n`));
        await new Promise(r => setTimeout(r, 100));
      }
      
      await writer.write(encoder.encode('data: [DONE]\n\n'));
    } finally {
      await writer.close();
    }
  })());
  
  return new Response(readable, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
      ...corsHeaders(),
    },
  });
}

async function handleStatus(env: Env): Promise<Response> {
  return jsonResponse({
    status: 'operational',
    models: {
      claude: { available: true, model: 'claude-3-5-sonnet-20241022' },
      gpt4: { available: true, model: 'gpt-4-turbo-preview' },
    },
    timestamp: new Date().toISOString(),
  });
}

async function handleUsage(request: Request, env: Env): Promise<Response> {
  const apiKey = request.headers.get('Authorization')?.replace('Bearer ', '');
  if (!apiKey) {
    return jsonResponse({ error: 'Missing API key' }, 401);
  }
  
  const userId = await validateApiKey(apiKey, env);
  if (!userId) {
    return jsonResponse({ error: 'Invalid API key' }, 401);
  }
  
  // Get usage from billing ledger
  const usage = await getUsage(userId, env);
  
  return jsonResponse(usage);
}

// Helper functions

async function validateApiKey(apiKey: string, env: Env): Promise<string | null> {
  const userId = await env.API_KEYS.get(apiKey);
  return userId;
}

async function checkRateLimit(userId: string, env: Env): Promise<boolean> {
  const key = `rate:${userId}:${Math.floor(Date.now() / 60000)}`;
  const current = parseInt(await env.RATE_LIMITS.get(key) || '0');
  
  if (current >= 100) { // 100 requests per minute
    return false;
  }
  
  await env.RATE_LIMITS.put(key, String(current + 1), { expirationTtl: 120 });
  return true;
}

interface ModelConfig {
  provider: 'anthropic' | 'openai';
  model: string;
  costPerInputToken: number;
  costPerOutputToken: number;
}

function selectModel(preference?: string): ModelConfig {
  // Default to Claude
  if (preference === 'gpt4') {
    return {
      provider: 'openai',
      model: 'gpt-4-turbo-preview',
      costPerInputToken: 0.00001,
      costPerOutputToken: 0.00003,
    };
  }
  
  return {
    provider: 'anthropic',
    model: 'claude-3-5-sonnet-20241022',
    costPerInputToken: 0.000003,
    costPerOutputToken: 0.000015,
  };
}

async function callAnthropic(
  req: EscalationRequest,
  model: ModelConfig,
  env: Env
): Promise<EscalationResponse> {
  const response = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': env.ANTHROPIC_API_KEY,
      'anthropic-version': '2023-06-01',
    },
    body: JSON.stringify({
      model: model.model,
      max_tokens: req.max_tokens,
      messages: [
        ...req.context.conversation_history,
        { role: 'user', content: buildPrompt(req) },
      ],
    }),
  });
  
  const data: any = await response.json();
  
  return {
    request_id: req.request_id,
    content: data.content[0].text,
    model_used: model.model,
    tokens_used: {
      prompt: data.usage.input_tokens,
      completion: data.usage.output_tokens,
    },
    cost_cents: calculateCost(data.usage, model),
    latency_ms: 0, // Will be set by caller
  };
}

async function callOpenAI(
  req: EscalationRequest,
  model: ModelConfig,
  env: Env
): Promise<EscalationResponse> {
  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${env.OPENAI_API_KEY}`,
    },
    body: JSON.stringify({
      model: model.model,
      max_tokens: req.max_tokens,
      messages: [
        ...req.context.conversation_history,
        { role: 'user', content: buildPrompt(req) },
      ],
    }),
  });
  
  const data: any = await response.json();
  
  return {
    request_id: req.request_id,
    content: data.choices[0].message.content,
    model_used: model.model,
    tokens_used: {
      prompt: data.usage.prompt_tokens,
      completion: data.usage.completion_tokens,
    },
    cost_cents: calculateCost(data.usage, model),
    latency_ms: 0,
  };
}

function buildPrompt(req: EscalationRequest): string {
  let prompt = '';
  
  if (req.context.pathos_framing) {
    prompt += `Context: ${req.context.pathos_framing}\n\n`;
  }
  
  if (req.context.local_knowledge.length > 0) {
    prompt += `Relevant knowledge:\n${req.context.local_knowledge.join('\n')}\n\n`;
  }
  
  prompt += req.query;
  
  return prompt;
}

function calculateCost(usage: any, model: ModelConfig): number {
  const inputCost = (usage.input_tokens || usage.prompt_tokens) * model.costPerInputToken;
  const outputCost = (usage.output_tokens || usage.completion_tokens) * model.costPerOutputToken;
  return Math.ceil((inputCost + outputCost) * 100); // Convert to cents
}

async function recordUsage(userId: string, response: EscalationResponse, env: Env): Promise<void> {
  // Get billing ledger Durable Object
  const id = env.BILLING_LEDGER.idFromName(userId);
  const stub = env.BILLING_LEDGER.get(id);
  
  await stub.fetch('https://billing/record', {
    method: 'POST',
    body: JSON.stringify({
      request_id: response.request_id,
      tokens_in: response.tokens_used.prompt,
      tokens_out: response.tokens_used.completion,
      cost_cents: response.cost_cents,
      model: response.model_used,
      timestamp: Date.now(),
    }),
  });
}

async function getUsage(userId: string, env: Env): Promise<any> {
  const id = env.BILLING_LEDGER.idFromName(userId);
  const stub = env.BILLING_LEDGER.get(id);
  
  const response = await stub.fetch('https://billing/usage');
  return response.json();
}

function jsonResponse(data: any, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      'Content-Type': 'application/json',
      ...corsHeaders(),
    },
  });
}

function corsHeaders(): Record<string, string> {
  return {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type, Authorization',
  };
}

function handleCORS(): Response {
  return new Response(null, {
    status: 204,
    headers: corsHeaders(),
  });
}
