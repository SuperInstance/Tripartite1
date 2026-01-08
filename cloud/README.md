# Cloudflare Workers for SuperInstance Cloud

**Status**: Stub implementation - TypeScript to be added in future session

This directory will contain the Cloudflare Workers implementation for the SuperInstance cloud backend.

## Architecture

```
cloud/
├── wrangler.toml          # Workers configuration
├── src/
│   ├── index.ts            # Main entry point
│   ├── tunnel.ts           # QUIC tunnel handler
│   ├── escalation.ts       # Cloud escalation handler
│   ├── billing.ts          # Billing Durable Object
│   ├── session.ts          # Session Durable Object
│   └── types.ts            # Shared types
└── package.json
```

## Implementation Status

### Session 2.7: Cloudflare Workers Durable Objects
- [ ] QUIC tunnel endpoint
- [ ] Session state Durable Object
- [ ] Billing Durable Object
- [ ] Escalation handler
- [ ] Stripe integration
- [ ] Workers AI integration

## Future Work

This will be implemented in a future session when Cloudflare Workers access is configured.

For now, the Rust client (synesis-cloud) can connect to a mock/stub server for testing.
