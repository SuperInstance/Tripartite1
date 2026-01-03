# Session 19: Hardware Manifests - Implementation Summary

## Overview
Created JSON schema and example hardware manifests for SuperInstance AI as specified in Session 19 of the build guide.

## Files Created

### 1. Schema Definition
**File**: `/mnt/c/claudesuperinstance/manifests/schema.json`

Complete JSON Schema (Draft 07) defining the structure for hardware manifests with:
- Device identification (name, vendor, architecture)
- Compute capabilities (GPU type, tensor cores, NPU)
- Memory configuration (VRAM, RAM, unified memory)
- Model recommendations (agents, quantizations, sizes)
- Thermal limits (TDP, throttle temperature)

### 2. Example Manifests

#### NVIDIA Jetson Orin Nano
**File**: `/mnt/c/claudesuperinstance/manifests/jetson-orin-nano.json`

```json
Device: NVIDIA Jetson Orin Nano
Architecture: aarch64 (ARM64)
GPU: CUDA 8.7 with tensor cores
Memory: 8GB unified memory
NPU: Jetson NPU
TDP: 15W
Max Context: 4096 tokens

Recommended Models:
- Pathos: phi-3-mini-4k-instruct (Q4_K_M, 2.4GB)
- Logos: llama-3.2-3b-instruct (Q4_K_M, 2GB) - optimized for edge
- Ethos: phi-3-mini-4k-instruct (Q4_K_M, 2.4GB) - shared with Pathos
- Embeddings: bge-micro-v1.5 (F16, 50MB)
```

#### NVIDIA RTX 4050 Laptop
**File**: `/mnt/c/claudesuperinstance/manifests/rtx-4050-laptop.json`

```json
Device: NVIDIA RTX 4050 Laptop GPU
Architecture: x86_64
GPU: CUDA 8.9 with tensor cores
Memory: 6GB VRAM, 16GB RAM recommended
NPU: None
TDP: 65W
Max Context: 8192 tokens

Recommended Models:
- Pathos: phi-3-mini-4k-instruct (Q4_K_M, 2.4GB)
- Logos: llama-3.2-8b-instruct (Q4_K_M, 4.7GB)
- Ethos: mistral-7b-instruct (Q4_K_M, 4.1GB)
- Embeddings: bge-small-en-v1.5 (F16, 130MB)
```

#### Apple M2
**File**: `/mnt/c/claudesuperinstance/manifests/apple-m2.json`

```json
Device: Apple M2
Architecture: arm64
GPU: Metal with tensor cores
Memory: Unified memory (16GB recommended)
NPU: Apple ANE
TDP: 20W
Max Context: 8192 tokens

Recommended Models:
- Pathos: phi-3-mini-4k-instruct (Q4_K_M, 2.4GB)
- Logos: llama-3.2-8b-instruct (Q5_K_M, 5.5GB) - higher quality for Metal
- Ethos: mistral-7b-instruct (Q4_K_M, 4.1GB)
- Embeddings: bge-small-en-v1.5 (F16, 130MB)
```

### 3. Documentation
**File**: `/mnt/c/claudesuperinstance/manifests/README.md`

Comprehensive documentation including:
- Schema structure explanation
- Usage examples
- Hardware detection workflow
- Custom manifest creation guide
- Contribution guidelines

## Schema Features

### Comprehensive Validation
- All fields have type constraints
- Enums for controlled vocabularies
- Pattern validation for compute capability
- Minimum/maximum constraints for numeric values
- Detailed descriptions for each field

### GPU Type Support
- **cuda**: NVIDIA GPUs (Jetson, GeForce, RTX, Quadro)
- **rocm**: AMD GPUs (Radeon, Instinct)
- **metal**: Apple GPUs (M1, M2, M3)
- **vulkan**: Cross-platform GPU compute
- **none**: CPU-only systems

### NPU Detection
- Apple ANE (Apple Neural Engine)
- Intel GNA (Gaussian Neural Accelerator)
- Qualcomm Hexagon
- Google TPU
- Jetson NPU
- None (no NPU present)

### Quantization Support
- **q4_0**: Basic 4-bit quantization
- **q4_k_m**: 4-bit with medium complexity (recommended)
- **q5_k_m**: 5-bit with medium complexity
- **q6_k**: 6-bit quantization
- **q8_0**: 8-bit quantization
- **f16**: Half-precision floating point

### Agent Mapping
Each manifest specifies models for the tripartite council:
- **Pathos**: User intent extraction (small, fast model)
- **Logos**: Solution synthesis (larger, more capable model)
- **Ethos**: Verification and safety (may reuse Pathos model)
- **Embeddings**: RAG and vector search (specialized embedding model)

## Key Design Decisions

### 1. Unified Memory Handling
For systems with unified memory (Apple Silicon, Jetson):
- `vram_mb` set to 0
- `unified_memory: true`
- System RAM used for both CPU and GPU operations

### 2. Thermal Constraints
Thermal limits included to prevent:
- CPU/GPU thermal throttling
- Reduced performance over time
- Hardware damage in edge deployments

### 3. Model Optimization
Manifests specify hardware-appropriate models:
- Jetson: Smaller models (3B) for edge deployment
- RTX 4050: Medium models (8B) for balanced performance
- Apple M2: Higher quantization (Q5) to leverage Metal performance

### 4. Context Window Scaling
Maximum context size varies by hardware:
- Jetson: 4096 tokens (memory constrained)
- RTX 4050: 8192 tokens (discrete VRAM)
- Apple M2: 8192 tokens (unified memory advantage)

## Validation Status

All JSON files validated and conform to schema.json:
- schema.json: Valid JSON Schema
- jetson-orin-nano.json: Valid hardware manifest
- rtx-4050-laptop.json: Valid hardware manifest
- apple-m2.json: Valid hardware manifest

## Next Steps (Session 20)

Per the build guide, the next session should implement:
1. Manifest loader in `synesis-models/src/hardware.rs`
2. `detect_and_load()` function for automatic hardware detection
3. `synesis manifest` CLI subcommand with:
   - `synesis manifest show` - display current manifest
   - `synesis manifest validate <path>` - validate JSON
   - `synesis manifest install <path>` - copy to manifests directory

## Notes

- Legacy manifests (`minimal.json`, `standard.json`) remain in old format
- Migration to new schema will occur in Phase 2
- New schema fully backward compatible with existing functionality
- All manifests follow Session 19 specification exactly

## Directory Structure

```
/mnt/c/claudesuperinstance/manifests/
├── schema.json              # JSON Schema definition
├── jetson-orin-nano.json   # NVIDIA Jetson Orin Nano
├── rtx-4050-laptop.json    # NVIDIA RTX 4050 Laptop
├── apple-m2.json           # Apple M2
├── README.md               # Documentation
├── minimal.json            # Legacy CPU-only profile
└── standard.json           # Legacy GPU profile
```

## Compatibility

- Schema: JSON Schema Draft 07
- Validation: Compatible with ajv, jsonschema, and other standard validators
- Platform: Cross-platform (Linux, macOS, Windows via WSL)
- Rust Integration: Ready for serde and jsonschema crate integration

---

**Session Completed**: Session 19 - Hardware Manifests Schema
**Status**: Complete
**Deliverables**: 1 schema, 3 example manifests, 1 README
