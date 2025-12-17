# RiceCoder Migration Guides

This directory contains migration guides for upgrading between RiceCoder versions, including breaking changes, migration steps, and compatibility information.

## Available Migration Guides

### Major Version Migrations

- [Migration to v1.0](v1.0-migration.md) - Breaking changes and migration from v0.x
- [Migration to v2.0](v2.0-migration.md) - Planned v2.0 migration guide

### Minor Version Updates

- [v0.1.7 to v0.1.8](v0.1.7-to-v0.1.8.md) - Phase 8 updates
- [v0.1.6 to v0.1.7](v0.1.6-to-v0.1.7.md) - Phase 7 integration features
- [v0.1.5 to v0.1.6](v0.1.5-to-v0.1.6.md) - Phase 6 infrastructure features

### Patch Updates

- [v0.1.7.x Updates](patch-updates.md) - Patch release migration notes

## Migration Strategy

### Before Migrating

1. **Backup your data**
   ```bash
   # Backup configuration
   cp -r ~/.ricecoder ~/.ricecoder.backup

   # Backup project configurations
   cp -r .agent .agent.backup
   ```

2. **Check compatibility**
   ```bash
   # Check current version
   rice --version

   # Review changelog
   rice changelog
   ```

3. **Test in staging**
   ```bash
   # Test migration in separate environment
   rice migrate --dry-run
   ```

### Migration Process

1. **Update RiceCoder**
   ```bash
   # Update via Cargo
   cargo install ricecoder --force

   # Or use installation script
   curl -fsSL https://raw.githubusercontent.com/moabualruz/ricecoder/main/scripts/install | bash
   ```

2. **Run migration**
   ```bash
   # Automatic migration
   rice migrate

   # Or manual migration following guide
   ```

3. **Verify migration**
   ```bash
   # Test basic functionality
   rice --version
   rice init --dry-run

   # Test AI provider connections
   rice provider test
   ```

### Rollback Plan

If migration fails:

```bash
# Restore backup
cp -r ~/.ricecoder.backup ~/.ricecoder
cp -r .agent.backup .agent

# Reinstall previous version
cargo install ricecoder --version "previous-version"
```

## Breaking Changes by Version

### v1.0 Breaking Changes

- **Configuration format**: TOML configuration replaces YAML
- **Provider API**: Unified provider interface
- **Session storage**: SQLite backend replaces JSON files
- **Command structure**: CLI commands reorganized

### v0.1.x Breaking Changes

- **v0.1.7**: GitHub integration requires token configuration
- **v0.1.6**: Session management changes
- **v0.1.5**: Performance baseline updates

## Compatibility Matrix

| Current Version | Target Version | Migration Required | Breaking Changes |
|----------------|----------------|-------------------|------------------|
| v0.1.6         | v0.1.7        | Yes              | Minor           |
| v0.1.7         | v0.1.8        | Yes              | Minor           |
| v0.1.x         | v1.0.0        | Yes              | Major           |

## Automated Migration

RiceCoder includes automated migration tools:

```bash
# Dry run migration
rice migrate --dry-run

# Interactive migration
rice migrate --interactive

# Force migration (skip prompts)
rice migrate --force

# Validate migration
rice migrate --validate
```

## Post-Migration Tasks

After successful migration:

1. **Update dependencies**
   ```bash
   rice update dependencies
   ```

2. **Reconfigure providers**
   ```bash
   rice provider reconfigure
   ```

3. **Update project configurations**
   ```bash
   rice init --update
   ```

4. **Test workflows**
   ```bash
   rice test workflows
   ```

## Support

### Getting Help with Migration

- **Migration issues**: Create issue with `migration` label
- **Breaking changes**: Check [CHANGELOG.md](../../CHANGELOG.md)
- **Community support**: [GitHub Discussions](https://github.com/moabualruz/ricecoder/discussions)

### Enterprise Migration Support

For enterprise customers:

- **Dedicated support**: Contact enterprise support
- **Migration planning**: Schedule migration consultation
- **Rollback assistance**: 24/7 rollback support during migration window