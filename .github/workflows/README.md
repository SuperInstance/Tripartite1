# GitHub Actions Workflows

This directory contains CI/CD workflows for SuperInstance AI.

## Workflows

### CI (`ci.yml`)
**Trigger**: Push to `main`/`develop`, pull requests

**Matrix Builds**:
- OS: Ubuntu, macOS, Windows
- Rust: Stable, Nightly

**Checks**:
- Formatting (`cargo fmt --check`)
- Linting (`cargo clippy`)
- Unit tests (`cargo test`)
- Documentation tests (`cargo test --doc`)
- Code coverage (codecov)
- Benchmarks (on main branch)
- MSRV verification

**Status Badge**:
```markdown
[![CI](https://github.com/SuperInstance/Tripartite1/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/ci.yml)
```

---

### Documentation (`documentation.yml`)
**Trigger**: Push to `main`, pull requests

**Checks**:
- Build documentation (`cargo doc`)
- Run documentation tests (`cargo test --doc`)
- Check intra-doc links
- Check for broken links (cargo-deadlinks)
- Deploy to GitHub Pages (on main)

**Status Badge**:
```markdown
[![Documentation](https://github.com/SuperInstance/Tripartite1/actions/workflows/documentation.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/documentation.yml)
```

---

### Security (`security.yml`)
**Trigger**: Push, pull requests, daily cron (00:00 UTC)

**Checks**:
- Security audit (cargo-audit)
- Dependency review
- Outdated dependencies
- Spell checking (typos)
- License checks (cargo-deny)
- Secret scanning (trufflehog)

**Status Badge**:
```markdown
[![Security](https://github.com/SuperInstance/Tripartite1/actions/workflows/security.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/security.yml)
```

---

### Release (`release.yml`)
**Trigger**: Tag push (`v*.*.*`), manual dispatch

**Jobs**:
- Validate version format
- Build binaries for 5 platforms
  - Linux (x86_64, ARM64)
  - macOS (x86_64, ARM64)
  - Windows (x86_64)
- Test release binaries
- Create GitHub release
- Publish to crates.io
- Build and push Docker images

**Manual Trigger**:
```bash
gh workflow run release.yml -f version=v0.2.0
```

---

## Configuration

### Required Secrets

For release workflow:
- `CRATES_IO_TOKEN` - crates.io API token (optional, for publishing)
- `GITHUB_TOKEN` - Automatically provided

### Environment Protection

Create `.github/environments/cratesio.yml` (or in GitHub UI):
- Required reviewers: 1
- Wait timer: 0 minutes
- Deployment branches: Only `main`

---

## Caching Strategy

Workflows use two-tier caching:

1. **Cargo Registry Cache**: `~/.cargo/registry` and `~/.cargo/git`
   - Key: `{os}-cargo-{rust}-{lock_hash}`
   - Shared across all Rust projects

2. **Build Cache**: `target/` directory
   - Key: `{os}-build-{rust}-{lock_hash}`
   - Shared within this project

Example key: `ubuntu-latest-cargo-stable-a1b2c3d4`

---

## Concurrency

All workflows use concurrency limits:
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

This means:
- One run per branch per workflow
- Cancel in-progress runs for the same branch
- Parallel runs for different branches/PRs

---

## Coverage

Code coverage is tracked via Codecov:

**Badge**:
```markdown
[![codecov](https://codecov.io/gh/SuperInstance/Tripartite1/branch/main/graph/badge.svg)](https://codecov.io/gh/SuperInstance/Tripartite1)
```

---

## MSRV (Minimum Supported Rust Version)

Current MSRV: **1.75.0**

Verified by `ci.yml` → `msrv` job.

---

## Performance

Typical workflow runtimes:
- CI (single matrix entry): 3-5 minutes
- Full CI (all matrix): 15-20 minutes (parallel)
- Documentation: 2-3 minutes
- Security: 1-2 minutes
- Release: 20-30 minutes (sequential)

---

## Troubleshooting

### Failing Tests
1. Check the logs for the specific test
2. Look at the matrix entry (OS + Rust version)
3. Reproduce locally: `cargo test --workspace`

### Failing Clippy
1. Review the clippy warnings
2. Fix locally: `cargo clippy --all-targets --all-features`
3. CI runs with `-Dwarnings`, so all warnings fail the build

### Failing Security Audit
1. Check for vulnerable dependencies
2. Review advisory database
3. Update dependencies: `cargo update`
4. For false positives, add `Cargo.toml` entries

### Release Failing
1. Check version format matches `v*.*.*`
2. Verify tag exists: `git tag -l`
3. Check crates.io token is valid
4. Verify Docker registry permissions

---

## Best Practices

### For Contributors
1. **Test locally before pushing**: Run `cargo test --workspace`
2. **Format your code**: Run `cargo fmt --all`
3. **Check clippy**: Run `cargo clippy --all-targets`
4. **Update documentation**: Add doc comments for public APIs

### For Maintainers
1. **Review failed workflows promptly**
2. **Keep dependencies updated**: Run security audit regularly
3. **Test release process**: Use `workflow_dispatch` before actual release
4. **Monitor coverage**: Ensure new code is tested

---

## Workflow Status

View all workflow runs:
https://github.com/SuperInstance/Tripartite1/actions

---

## Related Documentation

- [CLAUDE.md](../../CLAUDE.md) - Development guide
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - Contribution guidelines
- [README.md](../../README.md) - Project overview
