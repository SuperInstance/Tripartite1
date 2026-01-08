# GitHub Actions Quick Reference

## Status Badges for README

```markdown
[![CI](https://github.com/SuperInstance/Tripartite1/actions/workflows/ci.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/ci.yml)
[![Documentation](https://github.com/SuperInstance/Tripartite1/actions/workflows/documentation.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/documentation.yml)
[![Security](https://github.com/SuperInstance/Tripartite1/actions/workflows/security.yml/badge.svg)](https://github.com/SuperInstance/Tripartite1/actions/workflows/security.yml)
```

## Common Commands

### Trigger Workflows Manually

```bash
# Trigger security scan
gh workflow run security.yml

# Trigger release with version
gh workflow run release.yml -f version=v0.2.0
```

### View Workflow Runs

```bash
# List recent runs
gh run list --workflow=ci.yml

# View specific run
gh run view [run-id]

# Watch logs in real-time
gh run watch [run-id]
```

### Cancel Runs

```bash
# Cancel latest run
gh run list --workflow=ci.yml --limit 1 | awk '{print $NF}' | xargs gh run cancel

# Cancel all runs for a workflow
gh run list --workflow=ci.yml --json databaseId --jq '.[].databaseId' | xargs -I {} gh run cancel {}
```

## Workflow Files

- `ci.yml` - Main CI pipeline
- `documentation.yml` - Documentation builds
- `security.yml` - Security scanning
- `release.yml` - Release automation

## Secrets Required

For releases:
- `CRATES_IO_TOKEN` - crates.io API token

## Environment Protection

Create `crates.io` environment with 1 reviewer approval.
