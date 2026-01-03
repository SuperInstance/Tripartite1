// SuperInstance Billing Ledger - Durable Object
// Tracks per-user usage and handles billing thresholds
//
// This is a Durable Object that maintains consistent state
// for each user's billing information.

export interface UsageRecord {
  request_id: string;
  tokens_in: number;
  tokens_out: number;
  cost_cents: number;
  model: string;
  timestamp: number;
}

export interface BillingState {
  user_id: string;
  balance_cents: number;
  total_spent_cents: number;
  total_tokens_in: number;
  total_tokens_out: number;
  last_flush: number;
  pending_records: UsageRecord[];
}

// Cost multipliers
const LOCAL_MULTIPLIER = 1.0;      // No markup for local
const CLOUD_MULTIPLIER = 1.03;     // 3% markup for cloud escalation
const KNOWLEDGE_MULTIPLIER = 1.30; // 30% markup for knowledge credits

// Flush threshold - write to D1 when pending exceeds this
const FLUSH_THRESHOLD_CENTS = 500; // $5.00

export class BillingLedger implements DurableObject {
  private state: DurableObjectState;
  private billing: BillingState | null = null;

  constructor(state: DurableObjectState) {
    this.state = state;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    
    // Ensure state is loaded
    await this.ensureLoaded();
    
    switch (url.pathname) {
      case '/record':
        return this.handleRecord(request);
      case '/usage':
        return this.handleUsage();
      case '/balance':
        return this.handleBalance();
      case '/topup':
        return this.handleTopup(request);
      case '/flush':
        return this.handleFlush();
      default:
        return new Response('Not Found', { status: 404 });
    }
  }

  private async ensureLoaded(): Promise<void> {
    if (this.billing === null) {
      const stored = await this.state.storage.get<BillingState>('billing');
      this.billing = stored || {
        user_id: '',
        balance_cents: 0,
        total_spent_cents: 0,
        total_tokens_in: 0,
        total_tokens_out: 0,
        last_flush: Date.now(),
        pending_records: [],
      };
    }
  }

  private async handleRecord(request: Request): Promise<Response> {
    const record: UsageRecord = await request.json();
    
    // Apply cloud markup
    const adjusted_cost = Math.ceil(record.cost_cents * CLOUD_MULTIPLIER);
    record.cost_cents = adjusted_cost;
    
    // Update state
    this.billing!.pending_records.push(record);
    this.billing!.total_spent_cents += adjusted_cost;
    this.billing!.total_tokens_in += record.tokens_in;
    this.billing!.total_tokens_out += record.tokens_out;
    this.billing!.balance_cents -= adjusted_cost;
    
    // Check if we should flush to D1
    const pending_total = this.billing!.pending_records.reduce(
      (sum, r) => sum + r.cost_cents, 0
    );
    
    if (pending_total >= FLUSH_THRESHOLD_CENTS) {
      await this.flush();
    } else {
      // Just save to Durable Object storage
      await this.state.storage.put('billing', this.billing);
    }
    
    return new Response(JSON.stringify({
      success: true,
      new_balance_cents: this.billing!.balance_cents,
      cost_cents: adjusted_cost,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleUsage(): Promise<Response> {
    // Calculate usage for current period (month)
    const now = Date.now();
    const monthStart = new Date();
    monthStart.setDate(1);
    monthStart.setHours(0, 0, 0, 0);
    
    const recentRecords = this.billing!.pending_records.filter(
      r => r.timestamp >= monthStart.getTime()
    );
    
    const monthSpent = recentRecords.reduce((sum, r) => sum + r.cost_cents, 0);
    const monthTokensIn = recentRecords.reduce((sum, r) => sum + r.tokens_in, 0);
    const monthTokensOut = recentRecords.reduce((sum, r) => sum + r.tokens_out, 0);
    
    // Group by model
    const byModel: Record<string, { requests: number; cost: number }> = {};
    for (const record of recentRecords) {
      if (!byModel[record.model]) {
        byModel[record.model] = { requests: 0, cost: 0 };
      }
      byModel[record.model].requests++;
      byModel[record.model].cost += record.cost_cents;
    }
    
    return new Response(JSON.stringify({
      period: 'current_month',
      balance_cents: this.billing!.balance_cents,
      month: {
        spent_cents: monthSpent,
        tokens_in: monthTokensIn,
        tokens_out: monthTokensOut,
        requests: recentRecords.length,
      },
      all_time: {
        spent_cents: this.billing!.total_spent_cents,
        tokens_in: this.billing!.total_tokens_in,
        tokens_out: this.billing!.total_tokens_out,
      },
      by_model: byModel,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleBalance(): Promise<Response> {
    return new Response(JSON.stringify({
      balance_cents: this.billing!.balance_cents,
      total_spent_cents: this.billing!.total_spent_cents,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleTopup(request: Request): Promise<Response> {
    const { amount_cents } = await request.json() as { amount_cents: number };
    
    this.billing!.balance_cents += amount_cents;
    await this.state.storage.put('billing', this.billing);
    
    return new Response(JSON.stringify({
      success: true,
      new_balance_cents: this.billing!.balance_cents,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async handleFlush(): Promise<Response> {
    await this.flush();
    
    return new Response(JSON.stringify({
      success: true,
      flushed_records: this.billing!.pending_records.length,
    }), {
      headers: { 'Content-Type': 'application/json' },
    });
  }

  private async flush(): Promise<void> {
    // In a real implementation, this would write to D1
    // For now, just clear the pending records
    
    // TODO: Batch insert to D1
    // const records = this.billing!.pending_records;
    // await env.DB.batch([
    //   env.DB.prepare('INSERT INTO usage_records ...').bind(...),
    //   ...
    // ]);
    
    this.billing!.pending_records = [];
    this.billing!.last_flush = Date.now();
    await this.state.storage.put('billing', this.billing);
  }
}

// Export for wrangler.toml
export default {
  async fetch() {
    return new Response('Billing Ledger Durable Object');
  },
};
