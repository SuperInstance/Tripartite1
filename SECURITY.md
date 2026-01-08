# Security Policy

## Supported Versions

Security updates are provided for the following versions:

| Version | Supported          | Release Status |
|---------|--------------------|----------------|
| 0.2.x   | ✅ Yes             | Current (Phase 2) |
| 0.1.x   | ⚠️ Security fixes only | Previous (Phase 1) |
| < 0.1   | ❌ No              | Unsupported |

**Note:** SuperInstance AI is currently in active development (v0.x). Until we reach
v1.0, security fixes will be provided for the latest release only.

---

## Reporting a Vulnerability

### Private Disclosure Process

**We take security seriously.** If you discover a security vulnerability, please
report it to us privately rather than creating a public issue.

**How to Report:**

1. **Email (Preferred):** security@superinstance.ai
   - Include "SECURITY: [Brief Description]" in the subject line
   - Use PGP encryption if possible (key available below)
   - You'll receive an acknowledgment within 48 hours

2. **Private GitHub Advisory:**
   - Go to: https://github.com/SuperInstance/Tripartite1/security/advisories
   - Click "Report a vulnerability"
   - Fill out the form with details
   - This creates a private draft advisory visible only to you and maintainers

3. **Private Message to Maintainers:**
   - Send a private message to any maintainer via GitHub
   - Include "SECURITY" in your message subject

**What to Include:**

Please include as much of the following information as possible:

- **Description:** What is the vulnerability? What is the impact?
- **Steps to Reproduce:** How can we trigger the vulnerability?
- **Affected Versions:** Which versions are affected?
- **Proof of Concept:** Code or screenshots demonstrating the issue (if applicable)
- **Suggested Fix:** Do you have a suggested solution? (optional but helpful)
- **Your PGP Key:** If you want encrypted communication

### What Happens Next?

1. **Acknowledgment (within 48 hours):**
   - We'll confirm we received your report
   - We may ask for additional information
   - We'll provide a timeline for resolution

2. **Investigation (1-7 days):**
   - We'll validate and reproduce the vulnerability
   - We'll assess the severity and impact
   - We'll determine affected versions

3. **Resolution (varies by severity):**
   - We'll develop and test a fix
   - You'll be credited for the discovery (unless you prefer otherwise)
   - We'll coordinate a public disclosure date

4. **Disclosure:**
   - We'll announce the fix when it's available
   - We'll publish a security advisory
   - We'll credit you for the discovery (with your permission)

### Timeline Expectations

| Severity Level | Target Resolution Time | Definition |
|---------------|------------------------|------------|
| **Critical** | 48-72 hours | Exploitable in default config, data loss, remote code execution |
| **High** | 1 week | Exploitable with user interaction, privilege escalation |
| **Medium** | 2-4 weeks | Exploitable under specific conditions, limited impact |
| **Low** | Next release | Minor issues, edge cases, hard to exploit |

We prioritize by severity, but we work to address all reported vulnerabilities
as quickly as possible.

### Safe Harbor

**We want to hear about security vulnerabilities.** We pledge not to pursue legal
action against security researchers who:

- Report vulnerabilities to us privately (following the process above)
- Give us reasonable time to fix the issue before public disclosure
- Do not access or exfiltrate data beyond what's necessary to demonstrate the issue
- Do not degrade system performance or availability for users

**What we don't want:**
- Public disclosure before we've had a chance to fix it
- Exploiting vulnerabilities for malicious purposes
- Accessing user data without permission
- Disrupting service for legitimate users

If you follow responsible disclosure, you're our ally, not our adversary.

---

## PGP Key

For encrypted communication with the security team:

```
-----BEGIN PGP PUBLIC KEY BLOCK-----

[PGP key will be added when we establish a security@superinstance.ai mailbox]

-----END PGP PUBLIC KEY BLOCK-----
```

**Note:** PGP key will be added before our first security release. For now, please
use GitHub's private advisory system for encrypted reports.

---

## Security Best Practices for Users

### For Developers

1. **Keep Dependencies Updated:**
   ```bash
   # Check for security vulnerabilities
   cargo audit
   cargo outdated
   ```

2. **Use Secrets Management:**
   - Never hardcode API keys or credentials
   - Use environment variables or secret managers
   - Never commit secrets to git (even private repos)

3. **Enable Security Features:**
   ```bash
   # Run with address sanitizer (development)
   cargo +nightly test -Z sanitizer=address

   # Enable Rust's safety features
   # - No unsafe code without documentation
   # - Use type-safe wrappers
   # - Prefer Arc<Mutex<T>> over Rc<RefCell<T>>
   ```

4. **Follow Thread Safety Guidelines:**
   - Never hold `MutexGuard` across `.await` points
   - Use `tokio::sync::Mutex` in async code, not `std::sync::Mutex`
   - Use `Arc<T>` for shared state, not `Rc<T>`

### For Deployments

1. **Network Security:**
   - Use TLS 1.3 for all connections (enforced by default)
   - Enable mTLS for cloud connections (implemented in Phase 2)
   - Run behind a firewall when possible

2. **Access Control:**
   - Restrict API access with authentication
   - Use principle of least privilege
   - Monitor access logs

3. **Data Protection:**
   - Enable privacy redaction by default
   - Never store raw PII or secrets
   - Encrypt sensitive data at rest (SQLite encryption)

4. **Updates:**
   - Subscribe to security announcements
   - Update promptly when security releases are available
   - Test updates in staging before production

### For Privacy

SuperInstance is designed with **privacy-first** principles:

1. **Local-First Processing:**
   - Sensitive data stays on your device when possible
   - Cloud escalation requires explicit consent
   - Redaction happens before data leaves your device

2. **Token Vault:**
   - Sensitive data is replaced with UUID tokens
   - Tokens are isolated per-session (prevents cross-session leaks)
   - Token-to-value mapping never leaves your device

3. **No Telemetry by Default:**
   - We don't collect usage data without consent
   - Opt-in only for metrics and diagnostics
   - All data collection is transparent and configurable

### Known Security Considerations

**Current Limitations (as of v0.2.0):**

1. **No End-to-End Encryption:**
   - Cloud connections use TLS, but not E2E encryption
   - Cloudflare can theoretically see redacted content
   - Future: We're exploring E2E encryption for cloud escalations

2. **No Signed Releases:**
   - We don't currently sign release artifacts
   - Future: We'll implement sigstore/cosign for binary verification

3. **No Security Audit:**
   - The codebase has not been professionally audited (yet)
   - We're planning a security audit for v1.0 release

**Security Strengths:**

1. **Memory Safety:**
   - Rust prevents buffer overflows, use-after-free, and data races
   - No unsafe code without clear documentation and justification

2. **Privacy by Design:**
   - 18 built-in redaction patterns
   - Token vault prevents token reuse across sessions
   - No credentials in codebase (verified)

3. **Zero Vulnerabilities:**
   - All dependencies audited with `cargo audit`
   - No high/critical severity vulnerabilities in dependencies
   - Regular updates to dependency versions

---

## Security Features

### Privacy Proxy

The privacy proxy is our core security feature:

```rust
// All data is redacted before leaving the device
let redacted = privacy_proxy.redact(user_input)?;

// Tokens are UUIDs, no reverse engineering possible
// [EMAIL_01] -> test@example.com (mapping stays local)

// Cloud never sees raw PII or secrets
cloud_escalate(redacted).await?;
```

**Redaction Patterns (18 built-in):**
- Email addresses
- Phone numbers
- Social Security Numbers
- Credit card numbers
- API keys and tokens
- Passwords
- IP addresses (IPv4 and IPv6)
- URLs
- File paths
- Database connection strings
- JWT tokens
- Session IDs
- UUIDs
- Hashes (MD5, SHA256, etc.)
- And more...

See `synesis-privacy` crate for complete list.

### QUIC/TLS Tunnel

Phase 2 implements secure cloud communication:

```rust
// TLS 1.3 enforced (no TLS 1.2 or below)
// mTLS required (mutual authentication)
// QUIC protocol (resistant to protocol downgrade attacks)

let tunnel = CloudTunnel::connect_with_config(
    server_addr,
    tls_config, // TLS 1.3 only, mTLS enabled
).await?;
```

**Security Features:**
- TLS 1.3 only (no legacy protocol support)
- Mutual TLS (mTLS) authentication
- Certificate pinning (prevents MITM)
- QUIC protocol (resistant to amplification attacks)
- Automatic reconnection with exponential backoff

### Token Vault

The token vault prevents credential exposure:

```rust
// Tokens are UUIDs, cannot be reverse-engineered
// Counters reset each session (prevents cross-session inference)
let token = vault.generate_token("email").unwrap?;
// Returns: "[EMAIL_01]"

// Token-to-value mapping stored locally only
// Never transmitted to cloud
let original = vault.lookup_value(&token).unwrap?;
// Returns: "test@example.com"
```

**Security Properties:**
- Tokens are UUID v4 (random, not sequential)
- Counters reset each session (prevents correlation attacks)
- Vault isolated per-process (no shared state)
- No disk persistence (cleared on exit)

---

## Dependency Security

We monitor dependencies for security vulnerabilities:

```bash
# Audit dependencies for vulnerabilities
cargo audit

# View dependency tree
cargo tree

# Check for outdated dependencies
cargo outdated
```

**Current Status (as of 2026-01-07):**
- ✅ Zero high/critical severity vulnerabilities
- ✅ All dependencies actively maintained
- ✅ Regular security updates
- ✅ No dependencies with known security issues

**Policy:**
- We update dependencies monthly
- Security updates are released immediately
- We prefer mature, well-maintained crates
- We avoid dependencies with security history

---

## Security Incidents

### Past Incidents

**None reported yet.** The project is still in early development (v0.2.0).

### How We Handle Incidents

If a security incident occurs:

1. **Immediate Response:**
   - Confirm the vulnerability
   - Assess severity and impact
   - Notify affected users (if applicable)

2. **Fix Development:**
   - Develop and test the fix
   - Code review and security review
   - Create security advisory

3. **Release:**
   - Publish fixed version
   - Announce security advisory
   - Update documentation

4. **Post-Incident:**
   - Conduct root cause analysis
   - Improve processes and testing
   - Update this security policy

---

## Security Testing

### Automated Testing

```bash
# Run all tests
cargo test --workspace

# Run with sanitizers (catches memory issues)
cargo +nightly test -Z sanitizer=address
cargo +nightly test -Z sanitizer=leak

# Audit dependencies
cargo audit

# Check for unsafe code
cargo geiger  # (optional tool)
```

### Manual Testing

We perform manual security testing:

- Code review for security issues
- Threat modeling for new features
- Penetration testing (before v1.0 release)
- Dependency security audits (monthly)

### Third-Party Audits

**Planned:**
- Professional security audit before v1.0 release
- We're currently evaluating security firms
- Target date: Q2 2026

---

## Security Communication

### Security Announcements

We announce security issues through:

1. **GitHub Security Advisories:** https://github.com/SuperInstance/Tripartite1/security/advisories
2. **Release Notes:** Included in release notes
3. **Email:** security-announce@superinstance.ai (subscribe for notifications)

### Security Discussion

For general security discussions (not vulnerability reports):

- **GitHub Discussions:** https://github.com/SuperInstance/Tripartite1/discussions
- **Email:** security@superinstance.ai (non-urgent)
- **Discord:** #security channel (coming soon)

### Private Disclosure

For vulnerability reports, see [Reporting a Vulnerability](#reporting-a-vulnerability) above.

---

## Security Team

The current security team consists of project maintainers:

- **Geoffrey Huntley** (Project Lead)
- Additional maintainers will be added as the project grows

**Contact:**
- Email: security@superinstance.ai
- GitHub: @SuperInstance (organization)

---

## Acknowledgments

We want to thank everyone who reports security vulnerabilities and helps us keep
SuperInstance AI secure. Your responsible disclosure helps protect all our users.

Notable contributors will be acknowledged in security advisories (with permission).

---

## Additional Resources

- **Rust Security:** https://www.rust-lang.org/policies/security
- **Cargo Audit:** https://github.com/RustSec/cargo-audit
- **RustSec Advisory Database:** https://github.com/RustSec/advisory-db
- **OWASP Rust Cheat Sheet:** https://cheatsheetseries.owasp.org/cheatsheets/Rust_Cheat_Sheet.html

---

*Last Updated: 2026-01-07*
*Version: 1.0*

For questions about this security policy, contact security@superinstance.ai
