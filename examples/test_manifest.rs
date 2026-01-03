//! Test manifest loading functionality

use synesis_models::HardwareManifest;
use std::path::Path;

fn main() {
    println!("Testing Manifest Loader Functionality\n");
    println!("======================================\n");

    // Test 1: Load and validate manifest
    println!("Test 1: Loading test manifest...");
    let result = HardwareManifest::load(Path::new("test_manifest.json"));
    match result {
        Ok(manifest) => {
            println!("✓ Manifest loaded successfully!");
            println!("  Name: {}", manifest.name);
            println!("  Description: {}", manifest.description);
            println!("  Total size: {} bytes\n", manifest.total_download_size());
        }
        Err(e) => {
            println!("✗ Failed to load manifest: {}\n", e);
        }
    }

    // Test 2: Detect and load
    println!("Test 2: Detect hardware and load manifest...");
    let result = HardwareManifest::detect_and_load();
    match result {
        Ok(manifest) => {
            println!("✓ Manifest detected and loaded!");
            println!("  Name: {}", manifest.name);
            println!("  Description: {}", manifest.description);
        }
        Err(e) => {
            println!("✗ Failed to detect and load: {}\n", e);
        }
    }

    // Test 3: List built-in profiles
    println!("\nTest 3: Built-in profiles:");
    let profiles = synesis_models::manifest::profiles::all();
    for profile in profiles {
        println!("  - {} (RAM: {}+, VRAM: {}+)",
            profile.name,
            profile.min_ram_bytes / 1024 / 1024 / 1024,
            if profile.min_vram_bytes > 0 {
                format!("{}", profile.min_vram_bytes / 1024 / 1024 / 1024)
            } else {
                "N/A".to_string()
            }
        );
    }
}
