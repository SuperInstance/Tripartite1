# Migration Guides

This directory contains detailed migration guides for upgrading between versions of SuperInstance AI.

---

## Available Guides

### [v0.1.0 to v0.2.0 Migration Guide](./0.1.0-to-0.2.0.md)
**Status**: Draft (v0.2.0 in development)

**Overview**:
- Breaking changes
- New features
- Configuration updates
- Data migration
- Step-by-step instructions

---

## Migration Best Practices

### Before Migrating

1. **Backup your data**:
   ```bash
   # Backup knowledge vault
   cp -r ~/.synesis/knowledge ~/.synesis/knowledge.backup

   # Backup configuration
   cp ~/.synesis/config.toml ~/.synesis/config.backup.toml
   ```

2. **Check current version**:
   ```bash
   synesis --version
   ```

3. **Review breaking changes**:
   Read the migration guide for your target version carefully

### During Migration

1. **Stop all running instances**:
   ```bash
   # Ensure no synesis processes are running
   pkill synesis
   ```

2. **Install new version**:
   ```bash
   cargo build --release
   ```

3. **Run data migration**:
   ```bash
   synesis migrate --from 0.1.0 --to 0.2.0
   ```

4. **Verify migration**:
   ```bash
   synesis status
   synesis knowledge stats
   ```

### After Migration

1. **Test core functionality**:
   ```bash
   # Test query
   synesis ask "Test query"

   # Test knowledge vault
   synesis knowledge search "test"

   # Test privacy
   synesis ask "Contact test@example.com"
   ```

2. **Monitor logs**:
   ```bash
   journalctl -u synesis -f  # If running as service
   # or
   tail -f ~/.synesis/logs/synesis.log
   ```

3. **Rollback if needed**:
   ```bash
   # Restore from backup
   cp -r ~/.synesis/knowledge.backup ~/.synesis/knowledge
   cp ~/.synesis/config.backup.toml ~/.synesis/config.toml

   # Reinstall old version
   git checkout v0.1.0
   cargo build --release
   ```

---

## Version Support Policy

### Support Timeline

| Version | Release Date | Maintenance End | Status |
|---------|--------------|-----------------|--------|
| 0.1.0 | 2026-01-02 | 2026-03-01 | Maintained |
| 0.2.0 | TBD | TBD | In Development |

### Maintenance Policy

- **Critical bugs**: Fixed for all maintained versions
- **Security issues**: Fixed for all maintained versions
- **Features**: Added only to latest version
- **Deprecated versions**: 3 months notice before EOL

---

## Breaking Changes Policy

We strive to minimize breaking changes, but sometimes they're necessary for improvements.

### What Constitutes a Breaking Change

- **API changes**: Modifying public interfaces
- **Configuration changes**: Removing or changing config options
- **Data format changes**: Vault schema, token vault structure
- **Behavior changes**: Changes that affect how queries are processed

### What Is NOT a Breaking Change

- **Bug fixes**: Even if behavior changes slightly
- **Performance improvements**: Unless they change functionality
- **Additions**: New features, new config options
- **Documentation**: Typos, clarifications, reorganization

### Breaking Change Process

1. **Announce**: Document in [Unreleased] section of CHANGELOG.md
2. **Migration guide**: Provide detailed migration instructions
3. **Deprecation period**: Minimum 3 months for user code changes
4. **Clear communication**: Explain why the change is necessary

---

## Getting Help

### If You Encounter Issues

1. **Check the migration guide**: Ensure you followed all steps
2. **Check CHANGELOG.md**: Look for known issues
3. **Search GitHub Issues**: Someone may have encountered it already
4. **Open an issue**: Include:
   - Current version: `synesis --version`
   - Target version
   - Error messages
   - Steps to reproduce
   - System information: `synesis status`

### Resources

- [CHANGELOG.md](../CHANGELOG.md) - Complete version history
- [GitHub Issues](https://github.com/SuperInstance/Tripartite1/issues) - Issue tracker
- [GitHub Discussions](https://github.com/SuperInstance/Tripartite1/discussions) - Community help
- [Documentation](../docs/) - General documentation

---

## Contributing Migration Guides

If you've migrated between versions and have insights to share, please contribute!

### How to Contribute

1. Fork the repository
2. Create a new migration guide or update existing
3. Submit a pull request

### Migration Guide Template

```markdown
# vX.Y.Z to vA.B.C Migration Guide

**Status**: Draft | Stable

**Compatibility**: vX.Y.Z → vA.B.C

---

## Overview

Brief description of what's new in this version and why migration is needed.

---

## Breaking Changes

### Change 1: Title

**Why**: Reason for the change

**Impact**: Who is affected

**Migration**: How to update your code/config

### Change 2: Title

...

---

## New Features

List of new features you can use after migration

---

## Deprecated Features

Features that are deprecated and will be removed in future versions

---

## Pre-Migration Checklist

- [ ] Backup data
- [ ] Review breaking changes
- [ ] Stop running instances
- [ ] ...

---

## Step-by-Step Migration

### Step 1: Title

Instructions for first step

### Step 2: Title

...

---

## Post-Migration Verification

Steps to verify migration succeeded

---

## Rollback Instructions

How to revert if migration fails

---

## Known Issues

List of known issues and workarounds

---

## Additional Resources

Links to relevant documentation, issues, discussions
```

---

**Last Updated**: 2026-01-07
**Repository**: [SuperInstance/Tripartite1](https://github.com/SuperInstance/Tripartite1)
