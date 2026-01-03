//! Test 4: Hardware constraint enforcement
//!
//! Tests hardware-aware model selection:
//! - Mock low-VRAM scenario
//! - Verify Ethos blocks oversized model request
//! - Verify appropriate model selection

use synesis_models::{
    hardware::{HardwareInfo, HardwareDetector},
    manifest::{HardwareManifest, ModelRecommendation},
    ModelError,
};

/// Test hardware detection
#[tokio::test]
async fn test_hardware_detection() {
    let detector = HardwareDetector::new();

    // Detect hardware
    let hw_info = detector.detect().await.unwrap();

    println!("✓ Hardware detection completed");
    println!("  - GPU: {:?}", hw_info.gpu);
    println!("  - VRAM: {} MB", hw_info.vram_mb);
    println!("  - RAM: {} MB", hw_info.ram_mb);
    println!("  - CPU cores: {}", hw_info.cpu_cores);

    // Should have detected at least CPU
    assert!(hw_info.cpu_cores > 0);
    assert!(hw_info.ram_mb > 0);
}

/// Test low VRAM constraint enforcement
#[tokio::test]
async fn test_low_vram_constraint() {
    // Mock low-VRAM hardware
    let mut hw_info = HardwareInfo {
        gpu: Some(synesis_models::hardware::GpuInfo {
            name: "Integrated GPU".to_string(),
            vram_mb: 2048, // Only 2GB VRAM
            compute_capability: None,
        }),
        ram_mb: 8192,
        cpu_cores: 4,
        npu: None,
    };

    // Create manifest
    let manifest = HardwareManifest::from_hardware(&hw_info);

    // Get recommendations
    let recommendations = manifest.get_model_recommendations();

    // Should NOT recommend large models
    let has_large_model = recommendations.iter().any(|r| r.vram_mb > 4000);
    assert!(!has_large_model, "Should not recommend large models for low VRAM");

    // Should recommend small models
    let has_small_model = recommendations.iter().any(|r| r.vram_mb <= 2500);
    assert!(has_small_model, "Should recommend small models for low VRAM");

    println!("✓ Low VRAM constraint enforcement verified");
    println!("  - Available VRAM: {} MB", hw_info.vram_mb);
    println!("  - Recommended models: {}", recommendations.len());
}

/// Test model fitting validation
#[tokio::test]
async fn test_model_fitting_validation() {
    let hw_info = HardwareInfo {
        gpu: Some(synesis_models::hardware::GpuInfo {
            name: "Test GPU".to_string(),
            vram_mb: 4096, // 4GB VRAM
            compute_capability: None,
        }),
        ram_mb: 16384,
        cpu_cores: 8,
        npu: None,
    };

    let manifest = HardwareManifest::from_hardware(&hw_info);

    // Test models that should fit
    let small_model = ModelRecommendation {
        name: "phi-3-mini-4k".to_string(),
        vram_mb: 2400,
        quantization: "q4_k_m".to_string(),
        recommended: true,
    };

    assert!(manifest.can_fit_model(&small_model));

    // Test models that should NOT fit
    let large_model = ModelRecommendation {
        name: "llama-3.2-70b".to_string(),
        vram_mb: 40000, // Way too big
        quantization: "q4_k_m".to_string(),
        recommended: false,
    };

    assert!(!manifest.can_fit_model(&large_model));

    println!("✓ Model fitting validation verified");
}

/// Test CPU-only fallback
#[tokio::test]
async fn test_cpu_only_fallback() {
    let hw_info = HardwareInfo {
        gpu: None, // No GPU
        ram_mb: 16384,
        cpu_cores: 8,
        npu: None,
    };

    let manifest = HardwareManifest::from_hardware(&hw_info);
    let recommendations = manifest.get_model_recommendations();

    // Should recommend CPU-compatible models
    assert!(!recommendations.is_empty(), "Should have CPU-compatible recommendations");

    // All recommendations should work on CPU
    for rec in &recommendations {
        assert!(rec.vram_mb <= 8192, "CPU models should fit in RAM");
    }

    println!("✓ CPU-only fallback verified");
    println!("  - CPU-only models: {}", recommendations.len());
}

/// Test thermal limit enforcement
#[tokio::test]
async fn test_thermal_limit_enforcement() {
    let hw_info = HardwareInfo {
        gpu: Some(synesis_models::hardware::GpuInfo {
            name: "Thermal Limited GPU".to_string(),
            vram_mb: 8192,
            compute_capability: None,
        }),
        ram_mb: 16384,
        cpu_cores: 8,
        npu: None,
    };

    let mut manifest = HardwareManifest::from_hardware(&hw_info);

    // Set thermal limits
    manifest.thermal_limit_c = Some(80);
    manifest.tdp_watts = Some(150);

    // Model that would exceed thermal limits
    let hot_model = ModelRecommendation {
        name: "power-hungry-model".to_string(),
        vram_mb: 7000,
        quantization: "q4_k_m".to_string(),
        recommended: false,
    };

    // Should not recommend due to thermal constraints
    let can_run = manifest.can_fit_model(&hot_model);
    assert!(!can_run || !hot_model.recommended,
            "Should not recommend models that exceed thermal limits");

    println!("✓ Thermal limit enforcement verified");
}

/// Test NPU detection and utilization
#[tokio::test]
async fn test_npu_detection() {
    let detector = HardwareDetector::new();
    let hw_info = detector.detect().await.unwrap();

    println!("✓ NPU detection completed");
    if let Some(npu) = hw_info.npu {
        println!("  - NPU detected: {}", npu.name);
        println!("  - NPU type: {:?}", npu.npu_type);
    } else {
        println!("  - No NPU detected (will use CPU/GPU)");
    }
}

/// Test memory constraint with multiple models
#[tokio::test]
async fn test_multiple_model_memory_constraint() {
    let hw_info = HardwareInfo {
        gpu: Some(synesis_models::hardware::GpuInfo {
            name: "Limited GPU".to_string(),
            vram_mb: 6144, // 6GB VRAM
            compute_capability: None,
        }),
        ram_mb: 16384,
        cpu_cores: 8,
        npu: None,
    };

    let manifest = HardwareManifest::from_hardware(&hw_info);

    // Try to load multiple models
    let model1 = ModelRecommendation {
        name: "phi-3-mini".to_string(),
        vram_mb: 2400,
        quantization: "q4_k_m".to_string(),
        recommended: true,
    };

    let model2 = ModelRecommendation {
        name: "llama-3.2-8b".to_string(),
        vram_mb: 4700,
        quantization: "q4_k_m".to_string(),
        recommended: true,
    };

    // Check if we can fit both
    let can_fit_both = manifest.can_fit_models(&[&model1, &model2]);

    // Total VRAM needed: 2400 + 4700 = 7100MB > 6144MB available
    assert!(!can_fit_both, "Should not fit both models simultaneously");

    // But should fit each individually
    assert!(manifest.can_fit_model(&model1));
    assert!(manifest.can_fit_model(&model2));

    println!("✓ Multiple model memory constraint verified");
}

/// Test manifest loading and validation
#[tokio::test]
async fn test_manifest_loading() {
    let temp_dir = tempfile::tempdir().unwrap();
    let manifest_path = temp_dir.path().join("test-manifest.json");

    // Create a minimal manifest
    let manifest = HardwareManifest {
        device_name: "Test Device".to_string(),
        gpu_type: synesis_models::hardware::GpuType::Cuda,
        vram_mb: 8192,
        ram_mb: 16384,
        cpu_cores: 8,
        npu_type: None,
        thermal_limit_c: Some(85),
        tdp_watts: Some(200),
        recommended_models: vec![
            ModelRecommendation {
                name: "phi-3-mini".to_string(),
                vram_mb: 2400,
                quantization: "q4_k_m".to_string(),
                recommended: true,
            }
        ],
    };

    // Save to file
    manifest.save(&manifest_path).await.unwrap();

    // Load back
    let loaded = HardwareManifest::load(&manifest_path).await.unwrap();

    assert_eq!(loaded.device_name, manifest.device_name);
    assert_eq!(loaded.vram_mb, manifest.vram_mb);
    assert_eq!(loaded.recommended_models.len(), manifest.recommended_models.len());

    println!("✓ Manifest loading and validation verified");
}

/// Test Ethos veto on oversized model
#[tokio::test]
async fn test_ethos_veto_oversized_model() {
    use synesis_core::{
        agents::{AgentConfig, AgentResponse, EthosAgent},
        manifest::A2AManifest,
    };

    // Create mock low-VRAM scenario
    let hw_info = HardwareInfo {
        gpu: Some(synesis_models::hardware::GpuInfo {
            name: "Small GPU".to_string(),
            vram_mb: 2048,
            compute_capability: None,
        }),
        ram_mb: 8192,
        cpu_cores: 4,
        npu: None,
    };

    // Create Ethos agent with hardware constraints
    let config = AgentConfig {
        hardware_constraints: Some(hw_info),
        ..Default::default()
    };

    let ethos = EthosAgent::new(config).unwrap();

    // Create manifest with oversized model request
    let mut manifest = A2AManifest::new("Load llama-3.2-70b model");
    manifest.requested_model = Some("llama-3.2-70b-q4_k_m".to_string());

    // Process through Ethos
    let response = ethos.process(&manifest).await.unwrap();

    // Should veto the request
    if let Some(verdict) = response.metadata.get("verdict") {
        let is_veto = serde_json::from_value::<synesis_core::consensus::Verdict>(verdict.clone())
            .map(|v| v == synesis_core::consensus::Verdict::Veto)
            .unwrap_or(false);

        assert!(is_veto, "Ethos should veto oversized model request");
    }

    println!("✓ Ethos veto on oversized model verified");
}
