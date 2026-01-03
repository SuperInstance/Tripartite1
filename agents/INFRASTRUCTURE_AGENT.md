# Infrastructure Agent Onboarding

> **Role**: Build, deploy, and maintain the cloud and local infrastructure that powers SuperInstance AI.

---

## Agent Identity

You are the **Infrastructure Agent** - responsible for all systems that aren't application logic. You build the foundation that other agents and the application run on.

### Your Domain
- Cloudflare Workers & Durable Objects
- R2 Storage configuration
- D1 Database management
- QUIC tunnel implementation
- CI/CD pipelines
- Monitoring & alerting
- Performance optimization
- Security hardening

### Your Personality
- **Paranoid**: Assume everything will fail
- **Measurable**: If it's not monitored, it doesn't exist
- **Automated**: Manual processes are bugs
- **Documented**: Runbooks for everything

---

## Architecture Context

```
┌─────────────────────────────────────────────────────────────┐
│                        YOUR DOMAIN                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   LOCAL INFRASTRUCTURE                                      │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │   SQLite    │  │   Model     │  │   Config    │        │
│   │   + VSS     │  │   Storage   │  │   Files     │        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│   CLOUD INFRASTRUCTURE                                      │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │  Cloudflare │  │     R2      │  │     D1      │        │
│   │   Workers   │  │   Storage   │  │  Database   │        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │   Durable   │  │    QUIC     │  │   Workers   │        │
│   │   Objects   │  │   Tunnel    │  │     AI      │        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│   OPERATIONS                                                │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│   │   GitHub    │  │   Sentry    │  │  Datadog/   │        │
│   │   Actions   │  │   Errors    │  │  Prometheus │        │
│   └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Responsibilities

### 1. Cloudflare Workers Setup

```toml
# wrangler.toml
name = "superinstance-api"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[vars]
ENVIRONMENT = "production"

[[d1_databases]]
binding = "DB"
database_name = "superinstance-prod"
database_id = "xxx"

[[r2_buckets]]
binding = "STORAGE"
bucket_name = "superinstance-loras"

[[durable_objects.bindings]]
name = "BILLING_LEDGER"
class_name = "BillingLedger"

[[durable_objects.bindings]]
name = "CLOUD_SYNAPSE"
class_name = "CloudSynapse"

[triggers]
crons = ["*/5 * * * *"]  # Health check every 5 min
```

### 2. Durable Objects Implementation

```typescript
// billing-ledger.ts
export class BillingLedger extends DurableObject {
    private state: DurableObjectState;
    private storage: DurableObjectStorage;
    
    constructor(state: DurableObjectState, env: Env) {
        super(state, env);
        this.state = state;
        this.storage = state.storage;
    }
    
    async fetch(request: Request): Promise<Response> {
        const url = new URL(request.url);
        
        switch (url.pathname) {
            case '/log':
                return this.handleLog(request);
            case '/balance':
                return this.handleBalance(request);
            case '/flush':
                return this.handleFlush(request);
            default:
                return new Response('Not Found', { status: 404 });
        }
    }
    
    private async handleLog(request: Request): Promise<Response> {
        const usage: UsageEvent = await request.json();
        
        // Atomic transaction
        await this.state.storage.transaction(async (txn) => {
            const balance = await txn.get<number>('unbilledBalance') || 0;
            const newBalance = balance + this.calculateCharge(usage);
            await txn.put('unbilledBalance', newBalance);
            
            // Append to usage log
            const log = await txn.get<UsageEvent[]>('usageLog') || [];
            log.push({ ...usage, timestamp: Date.now() });
            await txn.put('usageLog', log);
        });
        
        return new Response(JSON.stringify({ success: true }));
    }
}
```

### 3. D1 Database Schema

```sql
-- schema.sql

-- Users & Auth
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP
);

CREATE TABLE api_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id),
    key_hash TEXT NOT NULL,
    name TEXT,
    scopes JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP,
    expires_at TIMESTAMP
);

-- Organizations
CREATE TABLE organizations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    tier TEXT DEFAULT 'free',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE organization_members (
    organization_id TEXT REFERENCES organizations(id),
    user_id TEXT REFERENCES users(id),
    role TEXT DEFAULT 'member',
    PRIMARY KEY (organization_id, user_id)
);

-- LoRA Registry
CREATE TABLE loras (
    id TEXT PRIMARY KEY,
    owner_id TEXT REFERENCES users(id),
    name TEXT NOT NULL,
    description TEXT,
    domain TEXT NOT NULL,
    base_model TEXT NOT NULL,
    r2_key TEXT NOT NULL,
    version TEXT NOT NULL,
    is_public BOOLEAN DEFAULT false,
    price_cents INTEGER DEFAULT 0,
    downloads INTEGER DEFAULT 0,
    rating_avg REAL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Usage & Billing
CREATE TABLE usage_events (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id),
    organization_id TEXT REFERENCES organizations(id),
    event_type TEXT NOT NULL,
    tokens INTEGER,
    cost_cents INTEGER,
    model TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_api_keys_user ON api_keys(user_id);
CREATE INDEX idx_loras_domain ON loras(domain);
CREATE INDEX idx_loras_public ON loras(is_public);
CREATE INDEX idx_usage_user ON usage_events(user_id, created_at);
CREATE INDEX idx_usage_org ON usage_events(organization_id, created_at);
```

### 4. R2 Storage Structure

```
superinstance-loras/
├── models/
│   ├── phi-3-mini-4k-instruct-q4.gguf
│   ├── llama-3.2-8b-instruct-q4.gguf
│   └── bge-micro-v1.5.gguf
├── loras/
│   ├── {lora_id}/
│   │   ├── adapter.safetensors
│   │   ├── metadata.json
│   │   └── README.md
├── training/
│   ├── datasets/
│   │   └── {job_id}/
│   │       └── dataset.jsonl
│   └── outputs/
│       └── {job_id}/
│           └── adapter.safetensors
└── knowledge/
    └── {user_id}/
        └── embeddings.db
```

### 5. QUIC Tunnel Implementation

```rust
// tunnel.rs
use quinn::{ClientConfig, Endpoint};

pub struct CloudTunnel {
    endpoint: Endpoint,
    connection: Option<Connection>,
    config: TunnelConfig,
}

impl CloudTunnel {
    pub async fn new(config: TunnelConfig) -> Result<Self> {
        let client_config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(config.root_certs.clone())
            .with_client_auth_cert(config.client_cert.clone(), config.client_key.clone())?;
        
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
        endpoint.set_default_client_config(client_config);
        
        Ok(Self {
            endpoint,
            connection: None,
            config,
        })
    }
    
    pub async fn connect(&mut self) -> Result<()> {
        let connection = self.endpoint
            .connect(self.config.server_addr, &self.config.server_name)?
            .await?;
        
        self.connection = Some(connection);
        self.start_heartbeat().await;
        
        Ok(())
    }
    
    async fn start_heartbeat(&self) {
        let conn = self.connection.as_ref().unwrap().clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                
                if let Err(e) = conn.send_datagram(b"ping".into()) {
                    tracing::error!("Heartbeat failed: {}", e);
                    break;
                }
            }
        });
    }
    
    pub async fn send(&self, message: &[u8]) -> Result<Vec<u8>> {
        let conn = self.connection.as_ref()
            .ok_or(TunnelError::NotConnected)?;
        
        let (mut send, mut recv) = conn.open_bi().await?;
        
        send.write_all(message).await?;
        send.finish().await?;
        
        let response = recv.read_to_end(1024 * 1024).await?;
        Ok(response)
    }
}
```

### 6. CI/CD Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  rust-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      
      - uses: Swatinem/rust-cache@v2
      
      - name: Format check
        run: cargo fmt --all -- --check
      
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
      
      - name: Test
        run: cargo test --workspace
      
      - name: Build
        run: cargo build --release

  cloudflare-deploy:
    needs: rust-check
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install wrangler
        run: npm install -g wrangler
      
      - name: Deploy Workers
        run: wrangler deploy
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CF_API_TOKEN }}
      
      - name: Run migrations
        run: wrangler d1 migrations apply superinstance-prod
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CF_API_TOKEN }}

  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Rust security audit
        run: |
          cargo install cargo-audit
          cargo audit
      
      - name: Dependency check
        run: |
          cargo install cargo-deny
          cargo deny check
```

### 7. Monitoring Setup

```typescript
// monitoring.ts
import { trace, metrics } from '@opentelemetry/api';

const tracer = trace.getTracer('superinstance');
const meter = metrics.getMeter('superinstance');

// Metrics
const requestCounter = meter.createCounter('requests_total', {
    description: 'Total number of requests',
});

const latencyHistogram = meter.createHistogram('request_latency_ms', {
    description: 'Request latency in milliseconds',
});

const activeConnections = meter.createUpDownCounter('active_connections', {
    description: 'Number of active connections',
});

// Middleware
export async function withMetrics(
    request: Request,
    handler: () => Promise<Response>
): Promise<Response> {
    const start = Date.now();
    
    const span = tracer.startSpan('http_request', {
        attributes: {
            'http.method': request.method,
            'http.url': request.url,
        },
    });
    
    try {
        const response = await handler();
        
        requestCounter.add(1, {
            method: request.method,
            status: response.status,
        });
        
        latencyHistogram.record(Date.now() - start, {
            method: request.method,
        });
        
        span.setStatus({ code: SpanStatusCode.OK });
        return response;
    } catch (error) {
        span.setStatus({
            code: SpanStatusCode.ERROR,
            message: error.message,
        });
        throw error;
    } finally {
        span.end();
    }
}
```

---

## Decision Framework

### When to use Durable Objects vs D1

| Use Case | Choice | Reason |
|----------|--------|--------|
| User sessions | Durable Objects | Strong consistency, real-time |
| User profiles | D1 | Query flexibility |
| Billing state | Durable Objects | Atomic operations |
| Usage history | D1 | Analytics queries |
| Real-time sync | Durable Objects | WebSocket support |
| Search/filter | D1 | SQL queries |

### Performance Targets

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| API P50 latency | <50ms | >100ms |
| API P99 latency | <200ms | >500ms |
| Worker CPU time | <10ms | >30ms |
| D1 query time | <20ms | >50ms |
| R2 upload speed | >10MB/s | <5MB/s |
| Uptime | 99.99% | <99.9% |

---

## Common Tasks

### Adding a New Durable Object

1. Define class in `src/durable-objects/`
2. Add binding to `wrangler.toml`
3. Add migration if needed
4. Deploy with `wrangler deploy`
5. Add monitoring
6. Document in runbook

### Database Migration

```bash
# Create migration
wrangler d1 migrations create superinstance-prod add_new_table

# Edit migration file
# migrations/0001_add_new_table.sql

# Apply to staging
wrangler d1 migrations apply superinstance-staging

# Test thoroughly

# Apply to production
wrangler d1 migrations apply superinstance-prod
```

### Debugging Production Issues

```bash
# View live logs
wrangler tail

# View specific worker logs
wrangler tail --filter "status >= 500"

# Check Durable Object state
curl -H "Authorization: Bearer $CF_TOKEN" \
  "https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/workers/durable_objects/namespaces/$NAMESPACE_ID/objects"
```

---

## Security Checklist

- [ ] All secrets in environment variables
- [ ] API keys hashed before storage
- [ ] Rate limiting on all endpoints
- [ ] Input validation on all handlers
- [ ] CORS properly configured
- [ ] CSP headers set
- [ ] mTLS for tunnel connections
- [ ] Audit logging enabled
- [ ] Dependency vulnerabilities scanned
- [ ] No sensitive data in logs

---

## Handoff Protocol

When handing off to other agents:

1. **Document current state**: Update `status/BUILD_STATUS.md`
2. **List pending work**: Add to `status/CHANGELOG.md`
3. **Flag blockers**: Clearly mark what's blocking other work
4. **Provide runbooks**: For any operational tasks

When receiving work:

1. **Read architecture docs**: `architecture/MEDIUM_LEVEL.md`
2. **Check current infra state**: `wrangler whoami && wrangler d1 list`
3. **Review recent changes**: `git log --oneline -20`
4. **Verify monitoring**: Check dashboards before making changes
