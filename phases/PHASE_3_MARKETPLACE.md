# Phase 3: Knowledge Marketplace
## Months 9-12

> **Goal**: Enable "Freelance Multiplier" - monetize expertise through LoRA licensing

---

## Phase Overview

Phase 3 transforms SuperInstance from a tool into a **platform**. Users who have trained domain-specific LoRA adapters can license them to others, creating passive income while buyers get instant expertise without training costs.

### Key Value Propositions

| Actor | Value |
|-------|-------|
| **LoRA Seller** | Passive income from expertise |
| **LoRA Buyer** | Skip training, instant capability |
| **Platform** | 10% marketplace fee |
| **Ecosystem** | Collective intelligence grows |

---

## Milestone 3.1: LoRA Registry
**Weeks 1-4**

### Objective
Create a centralized registry for discovering and managing LoRA adapters.

### Technical Components

#### Registry Schema
```sql
-- Cloudflare D1 Database
CREATE TABLE loras (
    id TEXT PRIMARY KEY,
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    domain TEXT NOT NULL,
    base_model TEXT NOT NULL,
    version TEXT NOT NULL,
    size_bytes INTEGER,
    r2_key TEXT NOT NULL,
    
    -- Licensing
    license_type TEXT DEFAULT 'proprietary',
    price_cents INTEGER DEFAULT 0,
    revenue_share_percent INTEGER DEFAULT 90,
    
    -- Metadata
    training_samples INTEGER,
    accuracy_benchmark REAL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    
    -- Discovery
    tags JSON,
    is_public BOOLEAN DEFAULT false,
    downloads INTEGER DEFAULT 0,
    rating_avg REAL,
    rating_count INTEGER DEFAULT 0
);

CREATE TABLE lora_versions (
    id TEXT PRIMARY KEY,
    lora_id TEXT REFERENCES loras(id),
    version TEXT NOT NULL,
    r2_key TEXT NOT NULL,
    changelog TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_loras_domain ON loras(domain);
CREATE INDEX idx_loras_public ON loras(is_public);
CREATE INDEX idx_loras_owner ON loras(owner_id);
```

#### Registry API
```typescript
// Workers API endpoints
interface LoraRegistryAPI {
    // Discovery
    'GET /loras': ListLoras;
    'GET /loras/:id': GetLora;
    'GET /loras/search': SearchLoras;
    
    // Management
    'POST /loras': CreateLora;
    'PUT /loras/:id': UpdateLora;
    'DELETE /loras/:id': DeleteLora;
    
    // Versions
    'GET /loras/:id/versions': ListVersions;
    'POST /loras/:id/versions': CreateVersion;
    
    // Usage
    'POST /loras/:id/download': InitiateDownload;
    'POST /loras/:id/rate': RateLora;
}
```

### CLI Commands
```bash
# List available LoRAs
synesis lora list --domain legal --sort downloads

# Get details
synesis lora info <lora-id>

# Download
synesis lora install <lora-id>

# Upload your own
synesis lora publish ./my-adapter --name "Legal Contract Expert"
```

### Acceptance Criteria
- [ ] Registry stores LoRA metadata
- [ ] Search by domain, tags, rating
- [ ] Version tracking with changelog
- [ ] R2 storage integration for binaries
- [ ] CLI can browse and install LoRAs

---

## Milestone 3.2: Licensing Engine
**Weeks 3-6**

### Objective
Enable creators to monetize LoRAs with flexible licensing models.

### License Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Free** | No charge, attribution optional | Community sharing |
| **Per-Download** | One-time purchase | Simple transactions |
| **Per-Query** | Pay per inference | Usage-based |
| **Subscription** | Monthly access | Heavy users |
| **Enterprise** | Custom terms | Organizations |

### Licensing Schema
```sql
CREATE TABLE licenses (
    id TEXT PRIMARY KEY,
    lora_id TEXT REFERENCES loras(id),
    buyer_id TEXT NOT NULL,
    license_type TEXT NOT NULL,
    
    -- Terms
    price_cents INTEGER,
    valid_from TIMESTAMP,
    valid_until TIMESTAMP,
    query_limit INTEGER,
    queries_used INTEGER DEFAULT 0,
    
    -- State
    status TEXT DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE license_usage (
    id TEXT PRIMARY KEY,
    license_id TEXT REFERENCES licenses(id),
    query_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    tokens_used INTEGER,
    cost_cents INTEGER
);
```

### Revenue Distribution
```typescript
interface RevenueShare {
    creatorPercent: 90;      // LoRA creator
    platformPercent: 10;     // SuperInstance
}

async function distributeRevenue(purchase: Purchase): Promise<void> {
    const creatorShare = purchase.amount * 0.90;
    const platformShare = purchase.amount * 0.10;
    
    await stripe.transfers.create({
        amount: creatorShare,
        destination: purchase.creator.stripeAccountId,
        currency: 'usd',
    });
    
    // Platform share stays in main account
    await recordPlatformRevenue(platformShare);
}
```

### Acceptance Criteria
- [ ] Multiple license types supported
- [ ] Automatic revenue distribution
- [ ] Usage tracking for per-query licenses
- [ ] License validation on every request
- [ ] Stripe Connect for creator payouts

---

## Milestone 3.3: Training Pipeline
**Weeks 5-8**

### Objective
Provide tools for users to train their own LoRA adapters.

### Training Workflow

```
┌─────────────────┐
│ Prepare Dataset │
│ (local)         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Upload to R2    │
│ (encrypted)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Train on Cloud  │ ◄── GPU Workers (A100/H100)
│ (Workers AI)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Download LoRA   │
│ (to local)      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Test & Validate │
│ (local)         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Publish         │
│ (optional)      │
└─────────────────┘
```

### Dataset Preparation
```rust
pub struct TrainingDataset {
    pub name: String,
    pub samples: Vec<TrainingSample>,
    pub config: TrainingConfig,
}

pub struct TrainingSample {
    pub instruction: String,
    pub input: Option<String>,
    pub output: String,
    pub metadata: Option<serde_json::Value>,
}

pub struct TrainingConfig {
    pub base_model: String,      // e.g., "llama-3.2-8b"
    pub lora_r: u32,             // Rank (8, 16, 32, 64)
    pub lora_alpha: f32,         // Scaling factor
    pub epochs: u32,
    pub batch_size: u32,
    pub learning_rate: f32,
    pub warmup_steps: u32,
}
```

### CLI Training Commands
```bash
# Prepare dataset from files
synesis train prepare ./documents --format alpaca --output dataset.json

# Validate dataset
synesis train validate dataset.json

# Estimate training cost
synesis train estimate dataset.json --base-model llama-3.2-8b

# Start training
synesis train start dataset.json \
    --name "Legal Contract Expert" \
    --base-model llama-3.2-8b \
    --lora-r 16 \
    --epochs 3

# Monitor progress
synesis train status <job-id>

# Download result
synesis train download <job-id>
```

### Cost Estimation
```typescript
interface TrainingEstimate {
    datasetSamples: number;
    epochs: number;
    estimatedGpuHours: number;
    estimatedCost: number;  // in cents
    baseModel: string;
    loraRank: number;
}

function estimateTrainingCost(dataset: Dataset, config: TrainingConfig): TrainingEstimate {
    const samplesPerHour = 1000; // Approximate for A100
    const gpuHours = (dataset.samples.length * config.epochs) / samplesPerHour;
    const costPerHour = 200; // $2.00 per GPU-hour
    
    return {
        datasetSamples: dataset.samples.length,
        epochs: config.epochs,
        estimatedGpuHours: gpuHours,
        estimatedCost: Math.ceil(gpuHours * costPerHour),
        baseModel: config.baseModel,
        loraRank: config.loraR,
    };
}
```

### Acceptance Criteria
- [ ] Dataset preparation from common formats
- [ ] Dataset validation with error reporting
- [ ] Secure upload to R2 (client-side encryption)
- [ ] Training job queuing and execution
- [ ] Progress monitoring with ETA
- [ ] Automatic download on completion
- [ ] Cost estimation before commit

---

## Milestone 3.4: Quality Assurance
**Weeks 7-10**

### Objective
Ensure marketplace LoRAs meet quality standards.

### Quality Metrics

| Metric | Threshold | Measurement |
|--------|-----------|-------------|
| **Accuracy** | >80% | Benchmark suite |
| **Latency** | <2x base | Inference timing |
| **Safety** | Pass | Red team prompts |
| **Stability** | <5% variance | Repeated runs |

### Benchmark System
```rust
pub struct BenchmarkSuite {
    pub domain: String,
    pub test_cases: Vec<BenchmarkCase>,
}

pub struct BenchmarkCase {
    pub id: String,
    pub prompt: String,
    pub expected_keywords: Vec<String>,
    pub expected_format: Option<String>,
    pub max_latency_ms: u64,
}

pub struct BenchmarkResult {
    pub lora_id: String,
    pub accuracy: f32,
    pub avg_latency_ms: u64,
    pub safety_pass: bool,
    pub stability_score: f32,
    pub overall_grade: Grade, // A, B, C, F
}
```

### Automated Review Pipeline
```
Upload LoRA
    │
    ▼
┌──────────────┐
│ Virus Scan   │───X── Reject
└──────┬───────┘
       │ ✓
       ▼
┌──────────────┐
│ Load Test    │───X── Reject (corrupted)
└──────┬───────┘
       │ ✓
       ▼
┌──────────────┐
│ Benchmark    │───X── Reject (<60% accuracy)
└──────┬───────┘
       │ ✓
       ▼
┌──────────────┐
│ Safety Check │───X── Reject (unsafe outputs)
└──────┬───────┘
       │ ✓
       ▼
┌──────────────┐
│ Human Review │──?── Manual approval queue
└──────┬───────┘
       │ ✓
       ▼
   Published
```

### Rating System
```sql
CREATE TABLE reviews (
    id TEXT PRIMARY KEY,
    lora_id TEXT REFERENCES loras(id),
    user_id TEXT NOT NULL,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    review_text TEXT,
    helpful_votes INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(lora_id, user_id)
);
```

### Acceptance Criteria
- [ ] Automated benchmark suite per domain
- [ ] Safety testing with red team prompts
- [ ] Latency regression detection
- [ ] Community rating system
- [ ] Flagging/reporting mechanism
- [ ] Quality badges (Verified, Top Rated)

---

## Milestone 3.5: Discovery & Recommendations
**Weeks 9-12**

### Objective
Help users find the right LoRAs for their needs.

### Discovery Features

#### Search & Filter
```typescript
interface LoraSearchParams {
    query?: string;           // Full-text search
    domain?: string[];        // Filter by domain
    baseModel?: string[];     // Compatible base models
    minRating?: number;       // Minimum rating
    maxPrice?: number;        // Price ceiling
    license?: string[];       // License type filter
    sortBy?: 'downloads' | 'rating' | 'recent' | 'relevance';
}
```

#### Recommendations
```rust
pub struct RecommendationEngine {
    // Based on user's:
    // - Domain (from queries)
    // - Installed LoRAs
    // - Rating history
    // - Similar users
}

impl RecommendationEngine {
    pub async fn get_recommendations(&self, user_id: &str) -> Vec<LoraRecommendation> {
        let user_profile = self.build_profile(user_id).await;
        
        let candidates = self.get_candidates(&user_profile).await;
        
        candidates
            .into_iter()
            .map(|lora| self.score_recommendation(&user_profile, &lora))
            .sorted_by(|a, b| b.score.cmp(&a.score))
            .take(10)
            .collect()
    }
}
```

#### Curated Collections
```sql
CREATE TABLE collections (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    curator_id TEXT,  -- NULL for official collections
    is_official BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE collection_items (
    collection_id TEXT REFERENCES collections(id),
    lora_id TEXT REFERENCES loras(id),
    position INTEGER,
    note TEXT,
    PRIMARY KEY (collection_id, lora_id)
);
```

### Featured Sections
- **Staff Picks**: Manually curated excellent LoRAs
- **Trending**: Most downloads this week
- **New Releases**: Recently published
- **By Domain**: Legal, Medical, Code, Creative, etc.
- **By Use Case**: Document analysis, Code review, Writing assistance

### Acceptance Criteria
- [ ] Full-text search with relevance ranking
- [ ] Multi-faceted filtering
- [ ] Personalized recommendations
- [ ] Curated collections
- [ ] Featured sections on homepage
- [ ] Similar LoRAs suggestions

---

## Technical Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    MARKETPLACE LAYER                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  Registry   │  │  Licensing  │  │  Training   │         │
│  │   Service   │  │   Engine    │  │   Pipeline  │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          │                                  │
│                    ┌─────┴─────┐                            │
│                    │    D1     │                            │
│                    │ Database  │                            │
│                    └───────────┘                            │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │     R2      │  │   Stripe    │  │   Workers   │         │
│  │   Storage   │  │   Connect   │  │     AI      │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### API Rate Limits

| Endpoint | Unauthenticated | Free Tier | Pro Tier |
|----------|-----------------|-----------|----------|
| Search | 10/min | 60/min | 300/min |
| Download | 0 | 5/day | Unlimited |
| Upload | 0 | 3/month | 20/month |
| Train | 0 | 1/month | 10/month |

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Listed LoRAs | 100+ | Registry count |
| Active Sellers | 20+ | Unique uploaders |
| Monthly Transactions | 50+ | License purchases |
| Marketplace GMV | $1,000+ | Total transaction value |
| Creator Payouts | $900+ | 90% of GMV |
| Avg Rating | 4.0+ | Mean of all ratings |
| Search → Install | 15%+ | Conversion rate |

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Low-quality LoRAs | High | High | Automated QA + human review |
| IP theft | Medium | High | Licensing enforcement, DMCA process |
| Training cost overruns | Medium | Medium | Hard caps, pre-payment |
| Market concentration | Medium | Medium | Featured rotation, discovery algorithms |
| Payment fraud | Low | High | Stripe Radar, manual review for large |

---

## Phase Exit Criteria

- [ ] 50+ public LoRAs in registry
- [ ] 10+ unique creators with published LoRAs
- [ ] 5+ successful training jobs completed
- [ ] 10+ license purchases processed
- [ ] $100+ in creator payouts distributed
- [ ] 4.0+ average marketplace rating
- [ ] <5% QA rejection rate
- [ ] All automated QA checks operational
