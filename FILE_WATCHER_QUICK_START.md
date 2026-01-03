# File Watcher Quick Start Guide

## Command Usage

```bash
# Basic usage - watch current directory
synesis knowledge watch .

# Watch specific directory
synesis knowledge watch ~/projects/my-rust-project

# Watch with custom file extensions
synesis knowledge watch ~/project --include rs,toml,md

# Watch documentation only
synesis knowledge watch ~/docs --include md,rst,adoc
```

## What Gets Watched?

### Default File Types:
- **Code**: `.rs`, `.py`, `.js`, `.ts`, `.go`
- **Docs**: `.md`, `.txt`
- **Config**: `.json`, `.yaml`, `.toml`

### Default Exclusions:
- `.git/`
- `node_modules/`
- `target/`
- `__pycache__/`
- `.venv/`
- Any hidden directory (starts with `.`)

## How It Works

### 1. Initial Scan
```
Watching: /home/user/project
Press Ctrl+C to stop

Scanning Initial indexing...
✓ Indexed 47 files (1,284 chunks)
```

### 2. Change Detection
When you save a file:
1. Watcher detects filesystem event
2. Computes SHA256 checksum
3. Compares with cached checksum
4. If changed → re-index

### 3. Auto-Indexing
```
Changed /home/user/project/main.rs - Indexing...
  ✓ /home/user/project/main.rs (12 chunks)
```

## Output Examples

### Successful Watch Session
```bash
$ synesis knowledge watch ~/my-project

Watching: /home/user/my-project
Press Ctrl+C to stop

Scanning Initial indexing...
✓ Indexed 23 files (542 chunks)

Watching for changes...

Changed /home/user/my-project/src/main.rs - Indexing...
  ✓ /home/user/my-project/src/main.rs (15 chunks)

Changed /home/user/my-project/README.md - Indexing...
  ✓ /home/user/my-project/README.md (6 chunks)

^C
Stopping watcher...
Done
```

### With Custom Extensions
```bash
$ synesis knowledge watch ~/api-docs --include md,yaml

Watching: /home/user/api-docs
Press Ctrl+C to stop

Scanning Initial indexing...
✓ Indexed 12 files (289 chunks)

Watching for changes...
```

## Configuration

### View Current Watch List
```bash
# TODO: Session 18+
synesis knowledge watch-list
```

### Add Directory to Permanent Watch List
```bash
# TODO: Session 18+
synesis knowledge watch-add ~/project --recursive
```

### Remove from Watch List
```bash
# TODO: Session 18+
synesis knowledge watch-remove ~/project
```

## Troubleshooting

### Path Not Found
```bash
$ synesis knowledge watch ~/nonexistent
Error: Path does not exist: ~/nonexistent
```

### Not a Directory
```bash
$ synesis knowledge watch ~/file.txt
Error: Path is not a directory: ~/file.txt
```

### Permission Denied
```bash
Changed /root/protected.rs - Indexing...
  ✗ /root/protected.rs: Permission denied
```

## Performance Tips

### For Large Projects
```bash
# Watch specific subdirectory instead of monorepo root
synesis knowledge watch ~/project/src

# Limit file types to reduce noise
synesis knowledge watch ~/project --include rs,toml
```

### For Active Development
The 2-second debounce prevents excessive re-indexing during rapid saves:
```bash
# Multiple rapid saves:
file.rs saved (1)  ← Ignored (debouncing)
file.rs saved (2)  ← Ignored (debouncing)
file.rs saved (3)  ← Processed (checksum changed)
  ✓ /path/to/file.rs (12 chunks)
```

### Optimize Exclusions
Create custom watch config (future feature):
```toml
# ~/.synesis/watch_config.toml
[watchers.my-project]
path = "~/project"
extensions = ["rs", "toml"]
exclude = ["target/", "assets/", "*.bak"]
debounce_secs = 5
```

## Integration with RAG

After watching files, query your knowledge:

```bash
# In another terminal
$ synesis ask "How does the file watcher detect changes?"

# System searches embeddings, retrieves relevant chunks
# Response includes specific code references
```

## Advanced Usage

### Watch Multiple Directories
```bash
# Terminal 1
synesis knowledge watch ~/project/src

# Terminal 2
synesis knowledge watch ~/project/docs
```

### Background Mode (Future)
```bash
# Run as daemon
synesis knowledge watch ~/project --daemon

# Check status
synesis knowledge watch-status

# Stop watcher
synesis knowledge watch-stop
```

### Custom Debounce
```bash
# Faster response for critical files
synesis knowledge watch ~/project --debounce 500ms  # 0.5s

# Slower for large projects
synesis knowledge watch ~/project --debounce 10s    # 10s
```

## File Type Support

### Currently Supported:
| Extension | Type | Chunking Strategy |
|-----------|------|-------------------|
| `.rs` | Code | Function/class (heuristic) |
| `.py` | Code | Function/class (heuristic) |
| `.js`, `.ts` | Code | Function/class (heuristic) |
| `.md` | Markdown | By heading |
| `.txt` | Text | Sliding window |
| `.json`, `.yaml`, `.toml` | Config | Sliding window |

### Planned:
- `.pdf` → PDF (text extraction then sliding window)
- `.ipynb` → Jupyter (code cell aware)
- `.html` → HTML (section aware)

## Best Practices

### 1. Granular Watching
```bash
# Good: Watch specific directories
synesis knowledge watch ~/project/src
synesis knowledge watch ~/project/docs

# Avoid: Watching entire monorepo with generated files
synesis knowledge watch ~/monorepo  # May include node_modules, target, etc.
```

### 2. Appropriate Extensions
```bash
# For Rust projects
synesis knowledge watch ~/project --include rs,toml,md

# For Python projects
synesis knowledge watch ~/project --include py,txt,md

# For docs-only
synesis knowledge watch ~/project --include md,rst
```

### 3. Exclude Build Artifacts
The watcher automatically excludes common patterns, but you can extend via config:

```bash
# Current defaults automatically exclude:
# - .git/, node_modules/, target/, __pycache__/, .venv/
```

## Monitoring

### What Gets Logged
```
Changed /path/to/file.rs - Indexing...     # File change detected
  ✓ /path/to/file.rs (12 chunks)            # Success
  ✗ /path/to/file.rs: error message        # Failure
Removed /path/to/file.rs                   # File deleted
Renamed /old/path → /new/path              # File moved
```

### Checking Vault Status
```bash
# In another terminal
$ synesis knowledge stats

Knowledge Vault Statistics
┌────────────────────────────────────┐
│ Total Documents      47            │
│ Total Chunks        1,284          │
│ Embedding Dimensions 384           │
│ Database Size       128 MB         │
│ Average Chunk Size  512 tokens     │
│ File Types          md, rs, py     │
│ Last Updated        2 minutes ago  │
│ Watched Directories 1              │
└────────────────────────────────────┘
```

## Stopping the Watcher

### Graceful Shutdown
```bash
# Press Ctrl+C
^C
Stopping watcher...
Done
```

The watcher will:
1. Stop watching for new changes
2. Finish processing current changes
3. Close database connection
4. Exit cleanly

## Session Context

This is part of **Session 17** of the SuperInstance AI build:
- Implements auto-indexing filesystem watcher
- Integrates with Sessions 15 (SQLite-VSS) and 16 (Embeddings)
- Foundation for Session 18 (RAG Integration)

For full implementation details, see:
`SESSION_17_IMPLEMENTATION_SUMMARY.md`
