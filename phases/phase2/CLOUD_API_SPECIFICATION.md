# SuperInstance Cloud API Specification

**Version**: 2.0.0
**Base URL**: `https://api.superinstance.ai/v2`
**Protocol**: HTTPS (REST) + QUIC (Tunnel)

---

## Authentication

### API Key Authentication

All API requests require authentication via Bearer token:

```http
Authorization: Bearer si_live_abc123xyz789
```

API keys have two prefixes:
- `si_live_` - Production keys
- `si_test_` - Test keys (sandbox environment)

### Device Authentication (QUIC Tunnel)

QUIC tunnel connections use mTLS with device certificates:

1. Device generates keypair during `synesis init`
2. Device registers with cloud, receives signed certificate
3. Subsequent connections use mTLS with device certificate

Certificate structure:
```
Subject: CN=device-{uuid}
Issuer: CN=SuperInstance Device CA
Valid: 1 year from registration
```

---

## REST API Endpoints

### User Management

#### Get Current User

```http
GET /v2/me
Authorization: Bearer {api_key}
```

Response:
```json
{
  "user_id": "usr_abc123",
  "email": "user@example.com",
  "name": "John Doe",
  "tier": "managed",
  "created_at": "2024-01-15T10:30:00Z",
  "devices": [
    {
      "device_id": "dev_xyz789",
      "name": "Desktop",
      "last_seen": "2024-03-01T15:45:00Z"
    }
  ]
}
```

#### Update User Settings

```http
PATCH /v2/me
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "name": "Jane Doe",
  "notification_email": "alerts@example.com"
}
```

---

### Billing

#### Get Balance

```http
GET /v2/billing/balance
Authorization: Bearer {api_key}
```

Response:
```json
{
  "unbilled_cents": 523,
  "credits_cents": 100,
  "credit_ceiling_cents": 10000,
  "tier": "managed",
  "tier_markup_percent": 3,
  "next_invoice_date": "2024-04-01T00:00:00Z",
  "stripe_customer_id": "cus_abc123"
}
```

#### Get Usage History

```http
GET /v2/billing/usage?days=30
Authorization: Bearer {api_key}
```

Response:
```json
{
  "period_start": "2024-03-01T00:00:00Z",
  "period_end": "2024-03-31T23:59:59Z",
  "total_cents": 1523,
  "total_requests": 450,
  "total_tokens_in": 125000,
  "total_tokens_out": 98000,
  "daily": [
    {
      "date": "2024-03-01",
      "requests": 15,
      "tokens_in": 4500,
      "tokens_out": 3200,
      "cost_cents": 52
    }
  ],
  "by_model": [
    {
      "model": "claude-3-5-sonnet-20241022",
      "requests": 300,
      "cost_cents": 1200
    },
    {
      "model": "gpt-4-turbo-preview",
      "requests": 150,
      "cost_cents": 323
    }
  ]
}
```

#### Create Checkout Session (Top-up)

```http
POST /v2/billing/checkout
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "amount_cents": 1000,
  "currency": "usd",
  "success_url": "synesis://payment-success",
  "cancel_url": "synesis://payment-cancel"
}
```

Response:
```json
{
  "checkout_url": "https://checkout.stripe.com/c/pay/cs_live_abc123",
  "session_id": "cs_live_abc123",
  "expires_at": "2024-03-15T12:00:00Z"
}
```

#### Set Billing Tier

```http
POST /v2/billing/tier
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "tier": "byok",
  "anthropic_api_key": "sk-ant-...",
  "openai_api_key": "sk-..."
}
```

Response:
```json
{
  "tier": "byok",
  "markup_percent": 30,
  "effective_from": "2024-03-15T00:00:00Z"
}
```

---

### Cloud Escalation

#### Escalate Query (Non-Streaming)

```http
POST /v2/escalate
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "request_id": "req_abc123",
  "session_id": "sess_xyz789",
  "query": "Explain quantum computing in simple terms",
  "context": {
    "pathos_framing": "User wants a simple explanation, likely beginner level",
    "local_knowledge": [
      "Previous conversation mentioned interest in physics"
    ],
    "conversation_history": [
      {"role": "user", "content": "I'm interested in learning about physics"},
      {"role": "assistant", "content": "Great! What aspect interests you most?"}
    ]
  },
  "model": "auto",
  "max_tokens": 1024
}
```

Response:
```json
{
  "request_id": "req_abc123",
  "content": "Quantum computing is like having a magical calculator...",
  "model_used": "claude-3-5-sonnet-20241022",
  "tokens_used": {
    "prompt": 245,
    "completion": 512
  },
  "cost_cents": 3,
  "latency_ms": 1250
}
```

#### Escalate Query (Streaming)

```http
POST /v2/escalate/stream
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "request_id": "req_abc123",
  "query": "Write a short story about AI",
  "stream": true,
  "max_tokens": 2048
}
```

Response (Server-Sent Events):
```
event: metadata
data: {"agent": "pathos", "phase": "analyzing"}

event: metadata
data: {"agent": "logos", "phase": "generating"}

event: content
data: {"text": "In the year 2045, "}

event: content
data: {"text": "an AI named Aurora "}

event: content
data: {"text": "discovered something extraordinary..."}

event: end
data: {"tokens_used": {"prompt": 120, "completion": 856}, "cost_cents": 4}
```

---

### LoRA Management

#### List Cloud LoRAs

```http
GET /v2/loras
Authorization: Bearer {api_key}
```

Response:
```json
{
  "loras": [
    {
      "id": "lora_abc123",
      "name": "legal-expert-v2",
      "base_model": "llama-3.1-8b",
      "size_bytes": 134217728,
      "uploaded_at": "2024-03-10T14:30:00Z",
      "last_used": "2024-03-15T09:15:00Z",
      "usage_count": 42
    }
  ]
}
```

#### Upload LoRA (Initiate)

```http
POST /v2/loras/upload
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "name": "my-custom-lora",
  "base_model": "llama-3.1-8b",
  "size_bytes": 134217728,
  "checksum_sha256": "abc123..."
}
```

Response:
```json
{
  "upload_id": "upl_xyz789",
  "upload_url": "https://upload.superinstance.ai/lora/upl_xyz789",
  "expires_at": "2024-03-15T14:00:00Z",
  "chunk_size": 5242880
}
```

#### Upload LoRA (Chunk)

```http
PUT /v2/loras/upload/{upload_id}/chunk/{chunk_index}
Authorization: Bearer {api_key}
Content-Type: application/octet-stream
Content-Length: 5242880

<binary data>
```

Response:
```json
{
  "chunk_index": 0,
  "received_bytes": 5242880,
  "total_received": 5242880,
  "remaining_chunks": 25
}
```

#### Upload LoRA (Complete)

```http
POST /v2/loras/upload/{upload_id}/complete
Authorization: Bearer {api_key}
```

Response:
```json
{
  "lora_id": "lora_def456",
  "name": "my-custom-lora",
  "status": "ready",
  "regions": ["us-east", "eu-west", "ap-south"]
}
```

#### Delete LoRA

```http
DELETE /v2/loras/{lora_id}
Authorization: Bearer {api_key}
```

Response:
```json
{
  "deleted": true,
  "lora_id": "lora_abc123"
}
```

---

### Collaborator System

#### Create Invite

```http
POST /v2/projects/{project_id}/invites
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "role": "viewer",
  "quota_cents": 500,
  "expires_hours": 48
}
```

Response:
```json
{
  "token": "inv_abc123xyz",
  "url": "https://app.superinstance.ai/join/inv_abc123xyz",
  "role": "viewer",
  "quota_cents": 500,
  "expires_at": "2024-03-17T14:00:00Z"
}
```

#### List Invites

```http
GET /v2/projects/{project_id}/invites
Authorization: Bearer {api_key}
```

Response:
```json
{
  "invites": [
    {
      "token": "inv_abc123xyz",
      "role": "viewer",
      "quota_cents": 500,
      "quota_used_cents": 120,
      "created_at": "2024-03-15T14:00:00Z",
      "expires_at": "2024-03-17T14:00:00Z",
      "accepted": true,
      "guest_email": "guest@example.com"
    }
  ]
}
```

#### Revoke Invite

```http
DELETE /v2/projects/{project_id}/invites/{token}
Authorization: Bearer {api_key}
```

#### Initiate Handover

```http
POST /v2/projects/{project_id}/handover
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "to_email": "newowner@example.com",
  "include_loras": true,
  "include_knowledge": true,
  "message": "Here's your project, ready to go!"
}
```

Response:
```json
{
  "handover_token": "ho_xyz789abc",
  "status": "pending",
  "to_email": "newowner@example.com",
  "expires_at": "2024-03-22T14:00:00Z",
  "incentive": {
    "first_year_price": 30.00,
    "regular_price": 60.00
  }
}
```

---

### Knowledge Sync

#### Sync Knowledge (Push)

```http
POST /v2/knowledge/sync
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "documents": [
    {
      "id": "doc_abc123",
      "path": "/project/README.md",
      "content_hash": "sha256:abc...",
      "chunks": [
        {
          "id": "chunk_001",
          "content": "# Project Overview...",
          "embedding": [0.123, -0.456, ...]
        }
      ]
    }
  ],
  "deleted_ids": ["doc_old456"]
}
```

Response:
```json
{
  "synced": 1,
  "deleted": 1,
  "total_chunks": 15,
  "sync_timestamp": 1710500000000
}
```

#### Sync Knowledge (Pull)

```http
GET /v2/knowledge/sync?since=1710400000000
Authorization: Bearer {api_key}
```

Response:
```json
{
  "documents": [...],
  "deleted_ids": ["doc_xyz"],
  "sync_timestamp": 1710500000000
}
```

---

## QUIC Tunnel Protocol

The QUIC tunnel uses binary protobuf messages over bidirectional streams.

### Message Types

| Type ID | Message | Direction |
|---------|---------|-----------|
| 0x01 | Heartbeat | Client → Server |
| 0x02 | HeartbeatAck | Server → Client |
| 0x03 | EscalationRequest | Client → Server |
| 0x04 | EscalationResponse | Server → Client |
| 0x05 | StreamChunk | Server → Client |
| 0x06 | PrewarmSignal | Client → Server |
| 0x07 | Error | Bidirectional |

### Frame Format

```
+----------------+----------------+----------------+
| Type (1 byte)  | Length (4 bytes, big-endian)    |
+----------------+----------------+----------------+
|              Payload (protobuf)                  |
+--------------------------------------------------+
```

### Heartbeat Flow

```
Client                          Server
   |                               |
   |------- Heartbeat (0x01) ----->|
   |                               |
   |<----- HeartbeatAck (0x02) ----|
   |                               |
   |  (repeat every 30 seconds)    |
```

### Escalation Flow

```
Client                          Server
   |                               |
   |-- EscalationRequest (0x03) -->|
   |                               |
   |                    [Process with cloud LLM]
   |                               |
   |<- EscalationResponse (0x04) --|
   |                               |
```

### Streaming Flow

```
Client                          Server
   |                               |
   |-- EscalationRequest (0x03) -->|
   |     (stream: true)            |
   |                               |
   |<---- StreamChunk (0x05) ------|
   |<---- StreamChunk (0x05) ------|
   |<---- StreamChunk (0x05) ------|
   |<---- StreamChunk (END) -------|
   |                               |
```

---

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 400 | bad_request | Invalid request format |
| 401 | unauthorized | Invalid or missing API key |
| 402 | payment_required | Credit ceiling exceeded |
| 403 | forbidden | Action not permitted |
| 404 | not_found | Resource not found |
| 429 | rate_limited | Too many requests |
| 500 | internal_error | Server error |
| 503 | service_unavailable | Temporary unavailability |

Error response format:
```json
{
  "error": {
    "code": "rate_limited",
    "message": "Rate limit exceeded. Retry after 60 seconds.",
    "retry_after": 60
  }
}
```

---

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| /v2/escalate | 100 | 1 minute |
| /v2/escalate/stream | 50 | 1 minute |
| /v2/loras/upload | 10 | 1 hour |
| All other endpoints | 1000 | 1 minute |

Rate limit headers:
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1710500060
```

---

## Webhooks

### Webhook Events

Configure webhooks in the dashboard to receive events:

- `billing.invoice.created` - New invoice generated
- `billing.payment.succeeded` - Payment processed
- `billing.payment.failed` - Payment failed
- `lora.upload.completed` - LoRA upload finished
- `handover.accepted` - Handover accepted by recipient
- `handover.completed` - Handover fully completed

### Webhook Payload

```json
{
  "id": "evt_abc123",
  "type": "billing.payment.succeeded",
  "created": 1710500000,
  "data": {
    "amount_cents": 1000,
    "currency": "usd"
  }
}
```

### Webhook Signature

Verify webhook signatures using the `X-Superinstance-Signature` header:

```python
import hmac
import hashlib

def verify_webhook(payload, signature, secret):
    expected = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(expected, signature)
```

---

## SDK Examples

### Rust (synesis-cloud)

```rust
use synesis_cloud::{CloudClient, EscalationRequest};

let client = CloudClient::new("si_live_abc123")?;

// Simple escalation
let response = client.escalate(EscalationRequest {
    query: "Explain quantum computing".to_string(),
    ..Default::default()
}).await?;

println!("Response: {}", response.content);

// Streaming
let mut stream = client.escalate_streaming(request).await?;
while let Some(chunk) = stream.next().await {
    print!("{}", chunk?.content);
}
```

### TypeScript

```typescript
import { SuperInstance } from '@superinstance/sdk';

const client = new SuperInstance('si_live_abc123');

// Simple escalation
const response = await client.escalate({
  query: 'Explain quantum computing',
});

console.log(response.content);

// Streaming
for await (const chunk of client.escalateStream(request)) {
  process.stdout.write(chunk.content);
}
```

### Python

```python
from superinstance import SuperInstance

client = SuperInstance('si_live_abc123')

# Simple escalation
response = client.escalate(
    query='Explain quantum computing'
)

print(response.content)

# Streaming
for chunk in client.escalate_stream(request):
    print(chunk.content, end='')
```

---

*API Version: 2.0.0*
*Last Updated: 2026-01-02*
