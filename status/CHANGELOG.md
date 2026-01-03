# Changelog

All notable changes to SuperInstance AI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Code Quality (2026-01-02)
- **All library crates now have ZERO compiler warnings!**
- Fixed 30 total compiler warnings across synesis-core, synesis-knowledge, and synesis-models
- Applied all clippy suggestions for more idiomatic Rust code
- Formatted all code with rustfmt

### Fixed
- **synesis-core (19 warnings → 0)**:
  - Removed unused imports (A2AManifest, uuid::Uuid)
  - Prefixed unused variables with underscore
  - Fixed single-character push_str() calls to use push()
  - Added #[derive(Default)] to structs
  - Added #[allow(dead_code)] to intentionally unused fields
  - Fixed manual RangeInclusive::contains implementation
- **synesis-knowledge (7 warnings → 0)**:
  - Removed unused imports
  - Changed filter_map(|p| p) to flatten()
  - Fixed collapsible_if statements
  - Removed useless format! macro
- **synesis-models (4 warnings → 0)**:
  - Removed unused imports
  - Used is_some_and() instead of map_or(false, ...)
  - Removed unnecessary borrows in Command::args()

### Planning
- Initial project documentation complete
- Architecture documents created
- Phase breakdowns defined
- Agent onboarding guides written
- Build guide and prompts prepared

### Added
- Project structure and documentation framework
- Rust workspace with 5 crates (synesis-cli, synesis-core, synesis-privacy, synesis-models, synesis-knowledge)
- All crates compile successfully with cargo check
- Hardware detection system (Session 2)
  - CPU detection with model name, cores, threads, and features (AVX, AVX2, AVX512, NEON)
  - GPU detection supporting NVIDIA, AMD, Apple Silicon, and Intel
  - RAM detection using sysinfo crate
  - Disk space detection via df command (Unix) with fallbacks
  - Platform detection with OS version strings
  - Hardware tier classification (1-5)
  - Hardware compatibility checking for models
- **RAG Integration for Logos Agent (Session 18)**
  - Full retrieval pipeline with key term extraction from A2AManifest
  - Query embedding generation (placeholder using SHA256 hashing)
  - Vault search integration point with top-K retrieval (default: 5 chunks)
  - Multi-factor relevance scoring:
    - Cosine similarity base score
    - Recency boost: `1.0 + (0.1 * days_since_update).min(0.5)`
    - Source quality multiplier: code (1.0) > docs (0.9) > notes (0.8)
  - Enhanced prompt building with formatted context chunks
    - Language-specific syntax highlighting
    - Source path citations
    - Relevance score display
    - Instructions to cite sources using `[SOURCE: path]` notation
  - Updated confidence calculation factoring RAG quality:
    - Average relevance boost (up to +0.25)
    - Source relevance bonus (up to +0.15)
    - Code ratio bonus (up to +0.10)
  - New data structures:
    - `RetrievedChunk`: Enhanced with doc_type and days_since_update
    - `RawChunkResult`: Internal structure for pre-scoring results
  - Comprehensive test coverage:
    - Relevance scoring tests
    - Key term extraction tests
    - Enhanced confidence calculation tests
  - RAG enable/disable flag for flexible deployment

### Changed
- Updated CLAUDE.md to reflect orchestrator role with autoaccept-enabled agents
- Added sysinfo crate to workspace dependencies for system information gathering

### Fixed
- Removed invalid [[bench]] section from workspace Cargo.toml
- Added missing error variants (NotLoaded, InvalidPath) to synesis-models
- Fixed hardware field name mismatches (ram_bytes, gpu vs gpus)
- Added OptionalExtension trait import to synesis-knowledge
- Fixed regex escaping issues in synesis-privacy (raw string format)
- Implemented Display trait for ModelSource enum
- Fixed borrow-after-move error in synesis-core (ethos.rs)

### Security
- N/A

---

## Session Log

### [DATE] - Session 1: Project Setup

**Duration**: ~X hours
**Focus**: Documentation and planning

**Completed**:
- Created CLAUDE.md orchestrator guide
- Created PROJECT_ROADMAP.md
- Created architecture documents (HIGH/MEDIUM/LOW)
- Created phase documents (1-4)
- Created agent onboarding docs
- Created CLAUDE_CODE_BUILD_GUIDE.md
- Created prompt templates
- Created status tracking files

**Decisions Made**:
- Rust for local kernel, TypeScript for cloud
- Cloudflare for cloud infrastructure
- SQLite-VSS for local vector storage
- Cost-plus pricing model (3%/30%)

**Blockers**: None

**Next Session**:
- Begin Phase 1.1 implementation
- Create GitHub repository
- Set up Rust workspace

---

### 2026-01-02 - Session 2: Hardware Detection Implementation

**Duration**: ~1 hour
**Focus**: Hardware detection for SuperInstance AI

**Completed**:
- Implemented full CPU detection using sysinfo crate
  - Detects CPU model name from processor information
  - Counts physical cores vs logical threads
  - Detects CPU features (AVX, AVX2, AVX512, FMA, NEON)
  - Cross-platform architecture detection
- Implemented comprehensive GPU detection
  - NVIDIA GPU detection via nvidia-smi with VRAM and CUDA version
  - AMD GPU detection via rocm-smi with VRAM parsing
  - Apple Silicon detection with unified memory support
  - Intel GPU detection via sycl-ls
  - Graceful fallbacks when GPU tools unavailable
- Implemented RAM detection using sysinfo crate
  - Total system memory
  - Available memory
  - Cross-platform support
- Implemented disk space detection
  - Unix: Uses df command to get real disk stats
  - Windows: Falls back to defaults (to be implemented)
  - Detects mount point for data directory
- Implemented platform detection
  - Linux: Reads /etc/os-release for distribution info
  - macOS: Uses sw_vers for version
  - Windows: Uses ver command
  - Fallback to kernel version for Linux
- Added hardware utility functions
  - Hardware tier classification (1-5 based on RAM and VRAM)
  - Model compatibility checking
  - Minimum requirements validation
  - Human-readable summary strings
- Added comprehensive error handling and debug logging
- Created unit tests for hardware detection
  - test_hardware_detection: Verifies basic detection works
  - test_hardware_tier: Tests tier calculation logic
  - test_format_bytes: Tests byte formatting utility

**Decisions Made**:
- Use sysinfo crate for cross-platform CPU and RAM detection
- Shell out to system commands (nvidia-smi, rocm-smi, df) rather than using native bindings
  - Simpler and more reliable than GPU-specific crate bindings
  - Easier to maintain across different platforms
- Use df command for disk detection on Unix systems
  - sysinfo 0.30 API changed significantly, using df is more reliable
- Graceful degradation when detection tools unavailable
  - Return sensible defaults rather than failing
  - Log debug messages for troubleshooting

**Technical Notes**:
- sysinfo crate version 0.30 has different API than older versions
- nvidia-smi CSV parsing is more reliable than screen-scraping
- AMD ROCm output format varies, implemented flexible parser
- Apple Silicon uses unified memory (system RAM = VRAM)
- Disk detection uses df -P (POSIX format) for consistent parsing

**Dependencies Added**:
- sysinfo = "0.30" (workspace dependency)

**Blockers**: None

**Next Session**:
- Implement model downloader with progress tracking
- Add model verification (SHA256 checksums)
- Implement model format detection (GGUF, SafeTensors, etc.)

---

## Template for New Sessions

```markdown
### [DATE] - Session N: [Title]

**Duration**: X hours
**Focus**: [Area of focus]

**Completed**:
- [Task 1]
- [Task 2]

**Decisions Made**:
- [Decision and rationale]

**Blockers**: 
- [Blocker] - [mitigation plan]

**Technical Notes**:
- [Important technical observation]

**Next Session**:
- [Priority 1]
- [Priority 2]
```

---

## Release History

### v0.0.0 - [Future Date]
- Initial documentation release
- No functional code yet

---

## Migration Notes

### Breaking Changes Log

[Document any breaking changes that will affect users]

### Deprecation Log

[Document deprecated features and removal timeline]

---

## Contributors

- Casey (Project Lead, Systems Engineer)
- Claude (AI Orchestrator)

---

## References

- [Architecture Decisions](/architecture/)
- [Phase Plans](/phases/)
- [Build Guide](/CLAUDE_CODE_BUILD_GUIDE.md)
