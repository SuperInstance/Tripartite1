# Manifest Commands Quick Reference

## Overview

The `synesis manifest` commands manage hardware manifests - profiles that map hardware capabilities to optimal model selections.

## Commands

### `synesis manifest show`

Displays the currently active hardware manifest based on detected hardware.

**Example:**
```bash
synesis manifest show
```

**Output includes:**
- Profile name and description
- RAM/VRAM requirements
- Current hardware capabilities
- Model recommendations (Pathos, Logos, Ethos, Embeddings)
- Total download size

**Use case:** See what models will be used for your hardware

---

### `synesis manifest validate <path>`

Validates a manifest JSON file without installing it.

**Example:**
```bash
synesis manifest validate ./custom-manifest.json
```

**What it checks:**
- JSON structure validity
- Required fields present
- RAM/VRAM requirements are reasonable
- Model recommendations are complete
- Context size is valid (1-131072)

**Output:** Success message with summary, or specific error details

**Use case:** Verify a custom manifest before installing

---

### `synesis manifest install <path> [name]`

Installs a manifest to `~/.synesis/manifests/` directory.

**Examples:**
```bash
# Install with default name (from filename)
synesis manifest install ./jetson-orin-nano.json

# Install with custom name
synesis manifest install ./my-profile.json jetson-custom
```

**What it does:**
- Validates the manifest
- Creates `~/.synesis/manifests/` if needed
- Copies manifest to the directory
- Returns confirmation with summary

**Use case:** Add custom hardware profiles for your devices

---

### `synesis manifest list`

Lists all available hardware profiles (built-in and custom).

**Example:**
```bash
synesis manifest list
```

**Output includes:**
- All built-in profiles (minimal, standard, performance, ultra, jetson-orin-nano)
- Any custom manifests in `~/.synesis/manifests/`
- Description and requirements for each

**Use case:** Discover available profiles and their requirements

---

## Manifest File Format

### Example Manifest

```json
{
  "name": "my-custom-profile",
  "description": "Custom profile for my hardware",
  "min_ram_bytes": 17179869184,
  "min_vram_bytes": 8589934592,
  "recommendations": {
    "pathos": {
      "model": "phi-3-mini",
      "quantization": "Q4",
      "repo_id": "microsoft/Phi-3-mini-4k-instruct-gguf",
      "filename": "Phi-3-mini-4k-instruct-q4.gguf",
      "size_bytes": 2200000000,
      "sha256": null
    },
    "logos": {
      "model": "llama-3.2-8b",
      "quantization": "Q4",
      "repo_id": "bartowski/Meta-Llama-3.2-8B-Instruct-GGUF",
      "filename": "Meta-Llama-3.2-8B-Instruct-Q4_K_M.gguf",
      "size_bytes": 4700000000,
      "sha256": null
    },
    "ethos": {
      "model": "phi-3-mini",
      "quantization": "Q4",
      "repo_id": "microsoft/Phi-3-mini-4k-instruct-gguf",
      "filename": "Phi-3-mini-4k-instruct-q4.gguf",
      "size_bytes": 0,
      "sha256": null
    },
    "embeddings": {
      "model": "bge-small",
      "quantization": "F16",
      "repo_id": "BAAI/bge-small-en-v1.5",
      "filename": "model.safetensors",
      "size_bytes": 130000000,
      "sha256": null
    }
  },
  "gpu_layers": 20,
  "context_size": 4096
}
```

### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique profile identifier |
| `description` | string | Yes | Human-readable description |
| `min_ram_bytes` | integer | Yes | Minimum system RAM required |
| `min_vram_bytes` | integer | Yes | Minimum VRAM required (0 for CPU-only) |
| `gpu_layers` | integer | Yes | Number of layers to offload to GPU |
| `context_size` | integer | Yes | Context window size (tokens) |
| `recommendations` | object | Yes | Model recommendations per agent |
| `recommendations.*.model` | string | Yes | Model name |
| `recommendations.*.quantization` | string | Yes | Q4, Q5, Q8, or F16 |
| `recommendations.*.repo_id` | string | Yes | HuggingFace repo ID |
| `recommendations.*.filename` | string | Yes | Model filename |
| `recommendations.*.size_bytes` | integer | Yes | Download size (0 if shared) |
| `recommendations.*.sha256` | string/null | No | Checksum for verification |

### Quantization Levels

- **Q4** - Smallest, fastest (Q4_K_M)
- **Q5** - Balanced (Q5_K_M)
- **Q8** - More accurate (Q8_0)
- **F16** - Full precision (FP16)

### Size Calculators

**RAM:**
```
8 GB  = 8589934592 bytes
16 GB = 17179869184 bytes
32 GB = 34359738368 bytes
64 GB = 68719476736 bytes
```

**VRAM:**
```
4 GB  = 4294967296 bytes
8 GB  = 8589934592 bytes
12 GB = 12884901888 bytes
16 GB = 17179869184 bytes
24 GB = 25769803776 bytes
```

## Built-in Profiles

### minimal
- **Description:** CPU-only with smallest models
- **RAM:** 8GB+
- **VRAM:** N/A
- **Models:** phi-3-mini (pathos/logos/ethos), bge-micro

### standard
- **Description:** Entry-level GPU or good CPU
- **RAM:** 16GB+
- **VRAM:** 4GB+
- **Models:** phi-3-mini, llama-3.2-8b, mistral-7b, bge-small

### performance
- **Description:** Mid-range GPU
- **RAM:** 32GB+
- **VRAM:** 8GB+
- **Models:** phi-3-medium, llama-3.1-8b, mistral-7b (Q5), bge-base

### ultra
- **Description:** High-end GPU
- **RAM:** 64GB+
- **VRAM:** 16GB+
- **Models:** phi-3-medium (Q8), llama-3.1-70b, mixtral-8x7b, bge-large

### jetson-orin-nano
- **Description:** NVIDIA Jetson Orin Nano edge device
- **RAM:** 8GB unified
- **VRAM:** N/A (unified memory)
- **Models:** phi-3-mini, llama-3.2-3b, bge-micro

## Manifest Selection Logic

When `detect_and_load()` is called:

1. **Detect Hardware**
   - CPU cores and capabilities
   - Total and available RAM
   - GPU type and VRAM (if present)

2. **Search Custom Manifests**
   - Look in `~/.synesis/manifests/`
   - Check each JSON manifest
   - Find first compatible match

3. **Fallback to Built-in**
   - If no custom match, use built-in profiles
   - Check from highest to lowest (ultra → minimal)
   - Select first compatible profile

4. **Return Manifest**
   - Hardware-optimized recommendations
   - Ready for model downloading

## Creating Custom Manifests

### Steps

1. **Detect your hardware:**
   ```bash
   synesis manifest show
   ```

2. **Start with similar built-in:**
   ```bash
   synesis manifest list
   ```

3. **Copy and modify:**
   ```bash
   cp ~/.synesis/manifests/standard.json my-profile.json
   # Edit my-profile.json
   ```

4. **Validate before installing:**
   ```bash
   synesis manifest validate my-profile.json
   ```

5. **Install:**
   ```bash
   synesis manifest install my-profile.json
   ```

### Tips

- **RAM/VRAM requirements:** Use minimum requirements, not what you have
- **GPU layers:** Higher = more GPU usage (99 for full offload)
- **Context size:** Balance between capability and memory (2048-8192 typical)
- **Shared models:** Set size to 0 if reusing same model (ethos = pathos)
- **Quantization:** Lower (Q4) for speed, higher (F16) for quality

## Troubleshooting

### Error: "Manifest name cannot be empty"
- Ensure the `name` field is not empty or missing

### Error: "min_ram_bytes must be > 0"
- Set a valid RAM requirement (at least 8GB recommended)

### Error: "Failed to parse manifest JSON"
- Check JSON syntax (use a JSON validator)
- Ensure all strings are quoted
- Verify no trailing commas

### Error: "Manifest not found"
- Check file path is correct
- Use absolute path if needed

### No manifest selected
- Run `synesis manifest show` to debug selection
- Check your hardware meets requirements
- Try installing a more permissive custom manifest

## Advanced Usage

### Programmatic Access

```rust
use synesis_models::HardwareManifest;
use std::path::Path;

// Load manifest
let manifest = HardwareManifest::load(Path::new("manifest.json"))?;

// Detect and auto-select
let manifest = HardwareManifest::detect_and_load()?;

// Validate
manifest.validate()?;

// Save modified manifest
manifest.save(Path::new("output.json"))?;

// Install to system
manifest.install("my-profile")?;
```

### Integration with Init

The `synesis init` command uses manifest loading:

1. Detects hardware
2. Loads appropriate manifest
3. Shows recommended models
4. Downloads selected models

### Custom Model Sources

Manifests can reference any HuggingFace repo:

```json
{
  "repo_id": "your-org/your-model-gguf",
  "filename": "your-model.Q4_K_M.gguf"
}
```

---

For more details, see:
- `CLAUDE_CODE_BUILD_GUIDE.md` - Session 20
- `crates/synesis-models/src/manifest.rs` - Implementation
- `SESSION_20_SUMMARY.md` - Implementation summary
