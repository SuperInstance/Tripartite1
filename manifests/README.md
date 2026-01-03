# Hardware Manifests

This directory contains hardware capability manifests for SuperInstance AI. These JSON files describe the compute capabilities, memory constraints, and model recommendations for different hardware platforms.

## Schema

The manifest schema is defined in `schema.json` (JSON Schema Draft 07). All hardware manifests must validate against this schema.

### Manifest Structure

Each manifest consists of five main sections:

#### 1. Device Information
```json
"device": {
  "name": "Human-readable device name",
  "vendor": "nvidia|amd|apple|intel|qualcomm|generic",
  "architecture": "x86_64|aarch64|arm64|riscv64"
}
```

#### 2. Compute Capabilities
```json
"compute": {
  "gpu_type": "cuda|rocm|metal|vulkan|none",
  "compute_capability": "X.Y (e.g., 8.7 for NVIDIA)",
  "tensor_cores": true|false,
  "npu_type": "none|apple-ane|intel-gna|qualcomm-hexagon|google-tpu|jetson-npu"
}
```

#### 3. Memory Configuration
```json
"memory": {
  "vram_mb": 0 (for CPU-only/unified) | discrete VRAM in MB,
  "ram_recommended_mb": recommended system RAM,
  "unified_memory": true|false
}
```

#### 4. Model Recommendations
```json
"models": {
  "max_context": maximum context window size,
  "recommended": [
    {
      "name": "model name",
      "agent": "pathos|logos|ethos|embeddings",
      "quantization": "q4_0|q4_k_m|q5_k_m|q6_k|q8_0|f16",
      "size_mb": model size in MB
    }
  ],
  "supported_quantizations": ["array of supported quantizations"]
}
```

#### 5. Thermal Limits
```json
"thermal": {
  "tdp_watts": thermal design power,
  "throttle_temp_c": temperature at which throttling begins
}
```

## Available Manifests

### jetson-orin-nano.json
- **Target**: NVIDIA Jetson Orin Nano development kits
- **Architecture**: ARM64 (aarch64)
- **Memory**: 8GB unified memory
- **GPU**: CUDA with 1024 CUDA cores, tensor cores
- **TDP**: 15W
- **Best for**: Edge AI deployments, robotics, IoT

### rtx-4050-laptop.json
- **Target**: Laptops with NVIDIA RTX 4050 GPU
- **Architecture**: x86_64
- **Memory**: 6GB VRAM, 16GB RAM recommended
- **GPU**: CUDA compute capability 8.9, tensor cores
- **TDP**: 65W
- **Best for**: General-purpose development, balanced performance

### apple-m2.json
- **Target**: Apple M2 (MacBook Air, Mac mini, etc.)
- **Architecture**: ARM64
- **Memory**: Unified memory architecture
- **GPU**: Metal with Apple Neural Engine
- **TDP**: 20W
- **Best for**: Power-efficient inference, macOS development

## Creating Custom Manifests

To create a manifest for your hardware:

1. Copy an existing manifest as a template
2. Modify the values to match your hardware
3. Validate against the schema:
   ```bash
   # Using ajv-cli
   ajv validate -s schema.json -d your-manifest.json

   # Or using Python
   python3 -m json.tool your-manifest.json
   ```
4. Place in `~/.synesis/manifests/` directory
5. Use with: `synesis init --manifest your-manifest.json`

## Hardware Detection

When you run `synesis init`, the system will:
1. Detect your hardware automatically
2. Find the closest matching manifest
3. Merge detected capabilities with manifest recommendations
4. Suggest appropriate models for your system

## Legacy Manifests

The following manifests use the older format and will be migrated to the new schema:
- `minimal.json` - CPU-only systems
- `standard.json` - Entry-level GPU systems

These are maintained for backward compatibility but will be deprecated in Phase 2.

## Contributing

To add support for new hardware:
1. Create a new manifest following the schema
2. Add realistic values based on manufacturer specifications
3. Test with actual hardware when possible
4. Submit a pull request with performance benchmarks

See `CONTRIBUTING.md` for guidelines.
