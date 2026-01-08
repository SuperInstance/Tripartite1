# Phase 2 Security Considerations

**Version**: 2.0.0
**Last Updated**: 2026-01-02
**Classification**: Internal Engineering

---

## Executive Summary

Phase 2 introduces cloud connectivity, which expands the attack surface significantly. This document outlines security requirements, threat models, and implementation guidelines.

---

## Threat Model

### Threat Actors

| Actor | Capability | Motivation |
|-------|------------|------------|
| **Malicious User** | Has valid account | Abuse service, extract data |
| **Network Attacker** | MITM position | Intercept credentials, data |
| **Compromised Cloud** | Access to cloud infra | Mass data exfiltration |
| **Rogue Client** | Modified client | Bypass billing, abuse API |

### Attack Vectors

#### 1. Credential Theft
- API keys stolen from config files
- Device certificates exfiltrated
- Session tokens hijacked

#### 2. Data Exfiltration
- Sensitive data leaked to cloud
- Token vault contents exposed
- Knowledge vault synced insecurely

#### 3. Billing Fraud
- Bypass usage tracking
- Spoof credits
- Denial of wallet attacks

#### 4. Unauthorized Access
- Collaborator privilege escalation
- Cross-tenant data access
- Admin impersonation

---

## Security Requirements

### SR-1: Transport Security

**Requirement**: All communication MUST use TLS 1.3 or QUIC with TLS 1.3.

**Implementation**:
```rust
// QUIC tunnel configuration
let crypto = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_certs)
    .with_client_auth_cert(device_cert, device_key)?;

// Enforce TLS 1.3 only
crypto.alpn_protocols = vec![b"h3".to_vec()];
```

**Verification**:
- [ ] No TLS 1.2 fallback
- [ ] Certificate pinning for cloud endpoints
- [ ] HSTS headers on all HTTP responses

### SR-2: Device Authentication

**Requirement**: Each device MUST have a unique certificate for mTLS.

**Implementation**:
```rust
// Device certificate generation during init
pub fn generate_device_certificate() -> Result<(Certificate, PrivateKey)> {
    let key = PrivateKey::generate_ed25519()?;
    
    let mut params = CertificateParams::new(vec![format!("device-{}", device_id)]);
    params.not_after = now() + Duration::days(365);
    params.key_usages = vec![
        KeyUsage::DigitalSignature,
        KeyUsage::KeyEncipherment,
    ];
    params.extended_key_usages = vec![
        ExtendedKeyUsage::ClientAuth,
    ];
    
    let cert = Certificate::from_params(params)?;
    cert.sign(&ca_key)?;
    
    Ok((cert, key))
}
```

**Certificate Rotation**:
- Certificates valid for 1 year
- Auto-renewal 30 days before expiry
- Revocation via cloud CA

### SR-3: API Key Security

**Requirement**: API keys MUST be stored securely and never logged.

**Implementation**:
```rust
// API key storage
pub fn save_api_key(key: &str) -> Result<()> {
    let encrypted = encrypt_with_device_key(key)?;
    
    // Store in system keychain if available
    if let Ok(keyring) = keyring::Entry::new("superinstance", "api_key") {
        keyring.set_password(&encrypted)?;
    } else {
        // Fallback to encrypted file with 0600 permissions
        let path = dirs::home_dir().unwrap().join(".superinstance/credentials");
        fs::write(&path, encrypted)?;
        fs::set_permissions(&path, Permissions::from_mode(0o600))?;
    }
    
    Ok(())
}

// Never log API keys
impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey(****)")
    }
}
```

**Key Scopes**:
| Scope | Permissions |
|-------|-------------|
| `read` | View balance, usage |
| `write` | Create invites, upload LoRAs |
| `escalate` | Send queries to cloud |
| `admin` | Manage API keys, billing |

### SR-4: Privacy Proxy Enforcement

**Requirement**: NO sensitive data shall reach cloud without redaction.

**Implementation**:
```rust
// Escalation flow with mandatory privacy
pub async fn escalate(&self, query: &str) -> Result<String> {
    // MANDATORY: Redact before sending
    let redacted = self.redactor.redact(query).await?;
    
    // Verify no tokens leaked
    #[cfg(debug_assertions)]
    {
        assert!(!redacted.contains("@"));  // Email patterns
        assert!(!redacted.contains("sk-")); // API keys
    }
    
    let response = self.cloud_client.send(&redacted).await?;
    
    // Reinflate response
    let restored = self.redactor.reinflate(&response).await?;
    
    Ok(restored)
}
```

**Verification**:
- [ ] Integration test: sensitive data never appears in network traffic
- [ ] Audit log: all redactions recorded locally
- [ ] Runtime check: patterns validated before send

### SR-5: Billing Integrity

**Requirement**: Usage tracking MUST be tamper-evident.

**Implementation**:
```typescript
// Durable Object billing with HMAC
class BillingLedger extends DurableObject {
    private async recordUsage(event: UsageEvent): Promise<void> {
        // Sign event with server key
        const signature = await this.sign(event);
        event.signature = signature;
        
        // Append to ledger (immutable)
        const ledger = await this.state.storage.get<UsageEvent[]>('ledger') || [];
        ledger.push(event);
        await this.state.storage.put('ledger', ledger);
        
        // Verify chain integrity
        await this.verifyLedgerIntegrity(ledger);
    }
    
    private async sign(event: UsageEvent): Promise<string> {
        const data = JSON.stringify({
            id: event.id,
            requestId: event.requestId,
            timestamp: event.timestamp,
            tokens: event.tokensIn + event.tokensOut,
            cost: event.costBasisCents,
        });
        
        return await crypto.subtle.sign(
            'HMAC',
            this.signingKey,
            new TextEncoder().encode(data)
        );
    }
}
```

**Fraud Prevention**:
- Credit ceiling enforced server-side
- Anomaly detection on usage patterns
- Rate limiting per user and device

### SR-6: Collaborator Access Control

**Requirement**: Collaborators MUST only access permitted resources.

**Implementation**:
```typescript
// Role-based access control
const ROLE_PERMISSIONS: Record<CollaboratorRole, string[]> = {
    viewer: ['read:sessions', 'read:agents'],
    commenter: ['read:sessions', 'read:agents', 'write:comments'],
    editor: ['read:sessions', 'read:agents', 'write:comments', 'write:prompts'],
};

async function checkPermission(
    userId: string,
    projectId: string,
    action: string
): Promise<boolean> {
    const collaborator = await getCollaborator(userId, projectId);
    if (!collaborator) return false;
    
    const permissions = ROLE_PERMISSIONS[collaborator.role];
    return permissions.includes(action);
}

// Quota enforcement
async function chargeGuest(
    guestId: string,
    costCents: number
): Promise<boolean> {
    const session = await getGuestSession(guestId);
    if (session.remainingQuota < costCents) {
        throw new Error('Guest quota exceeded');
    }
    
    session.remainingQuota -= costCents;
    await saveGuestSession(session);
    
    // Bill host
    await chargeHost(session.hostUserId, costCents);
    
    return true;
}
```

### SR-7: Input Validation

**Requirement**: All inputs MUST be validated and sanitized.

**Implementation**:
```rust
// Request validation
impl EscalationRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Request ID format
        if !self.request_id.starts_with("req_") || self.request_id.len() != 20 {
            return Err(ValidationError::InvalidRequestId);
        }
        
        // Query length
        if self.query.len() > MAX_QUERY_LENGTH {
            return Err(ValidationError::QueryTooLong);
        }
        
        // Max tokens range
        if self.max_tokens < 1 || self.max_tokens > 4096 {
            return Err(ValidationError::InvalidMaxTokens);
        }
        
        // Context size
        if self.context.conversation_history.len() > MAX_HISTORY_LENGTH {
            return Err(ValidationError::HistoryTooLong);
        }
        
        Ok(())
    }
}
```

**SQL Injection Prevention**:
```typescript
// Always use parameterized queries
const result = await env.DB.prepare(`
    SELECT * FROM users WHERE id = ?
`).bind(userId).first();

// NEVER do this
// const result = await env.DB.exec(`SELECT * FROM users WHERE id = '${userId}'`);
```

### SR-8: Secrets Management

**Requirement**: Secrets MUST never be committed to repository.

**Implementation**:
```toml
# wrangler.toml - reference secrets, don't include them
[vars]
ENVIRONMENT = "production"

# Secrets stored in Cloudflare dashboard
# ANTHROPIC_API_KEY
# OPENAI_API_KEY
# STRIPE_SECRET_KEY
# SIGNING_KEY
```

**Local Development**:
```bash
# Use .dev.vars for local secrets (in .gitignore)
echo "ANTHROPIC_API_KEY=sk-ant-..." > .dev.vars
```

---

## Security Checklist

### Before Deployment

- [ ] All secrets removed from codebase
- [ ] TLS 1.3 enforced
- [ ] mTLS certificates generated
- [ ] Rate limits configured
- [ ] Credit ceilings set
- [ ] Input validation complete
- [ ] SQL injection audit passed
- [ ] Privacy proxy integration tested
- [ ] RBAC implemented for collaborators

### Ongoing

- [ ] Weekly dependency audit (`cargo audit`)
- [ ] Monthly certificate rotation check
- [ ] Quarterly penetration test
- [ ] Continuous anomaly monitoring

---

## Incident Response

### Credential Compromise

1. **Detect**: Monitor for unusual API patterns
2. **Revoke**: Immediately revoke compromised keys
3. **Rotate**: Force rotation of device certificates
4. **Notify**: Alert affected users
5. **Review**: Audit access logs

### Data Breach

1. **Contain**: Isolate affected systems
2. **Assess**: Determine scope of exposure
3. **Notify**: Legal/compliance notification
4. **Remediate**: Patch vulnerability
5. **Report**: Post-mortem and disclosure

---

## Compliance

### GDPR

- User data deletion on request
- Data portability (export)
- Consent for data processing
- Privacy policy updates

### SOC 2

- Access control documentation
- Encryption at rest and in transit
- Audit logging
- Incident response procedures

---

*Document Version: 2.0.0*
*Security Review: Pending*
*Next Review: Before Phase 2 deployment*
