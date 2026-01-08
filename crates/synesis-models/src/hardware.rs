//! Hardware Detection
//!
//! Detects system hardware capabilities for optimal model selection.
//!
//! # Hardware Detection Strategy
//!
//! The detector tries multiple approaches for each component:
//! - **CPU**: Uses sysinfo for accurate core/thread counts and feature detection
//! - **GPU**: Tries NVIDIA (nvidia-smi), AMD (rocm-smi), Apple Silicon (unified memory), Intel (sycl-ls)
//! - **RAM**: Uses sysinfo for total and available memory
//! - **Disk**: Uses df command on Unix, falls back to defaults on Windows
//!
//! # Performance
//!
//! Hardware detection is synchronous and can take 100-500ms depending on:
//! - Number of GPUs detected
//! - Speed of subprocess calls (nvidia-smi, rocm-smi, etc.)
//! - Disk I/O for df command
//!
//! It's recommended to cache the `HardwareInfo` result for the application lifetime.

use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::System;
use tracing::{debug, info, instrument, warn};

use crate::ModelResult;

// Constants for hardware requirements and calculations

/// Minimum RAM required for basic operation (8 GB)
const MIN_RAM_BYTES: u64 = 8 * 1024 * 1024 * 1024;

/// Minimum disk space required (10 GB)
const MIN_DISK_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// Tier thresholds for RAM (in GB)
const TIER_RAM_HIGH: u64 = 64;    // >= 64GB: Tier 4
const TIER_RAM_MID: u64 = 32;     // >= 32GB: Tier 3
const TIER_RAM_LOW: u64 = 16;     // >= 16GB: Tier 2

/// Tier thresholds for GPU VRAM (in GB)
const TIER_VRAM_HIGH: u64 = 24;   // >= 24GB: Tier 5
const TIER_VRAM_MID: u64 = 12;    // >= 12GB: Tier 4
const TIER_VRAM_LOW: u64 = 8;     // >= 8GB: Tier 3
const TIER_VRAM_MIN: u64 = 4;     // >= 4GB: Tier 2

/// Conversion factor: Megabytes to Bytes
const MB_TO_BYTES: u64 = 1024 * 1024;

/// Default disk space assumption for Windows (500 GB)
const DEFAULT_DISK_TOTAL: u64 = 500 * 1024 * 1024 * 1024;

/// Default available disk space for Windows (100 GB)
const DEFAULT_DISK_AVAILABLE: u64 = 100 * 1024 * 1024 * 1024;

/// Default VRAM for AMD GPU if detection fails (8 GB)
const DEFAULT_AMD_VRAM: u64 = 8 * 1024 * 1024 * 1024;

/// Detected hardware information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// CPU information
    pub cpu: CpuInfo,
    /// Total system RAM in bytes
    pub ram_bytes: u64,
    /// Available RAM in bytes
    pub ram_available_bytes: u64,
    /// GPU information (if available)
    pub gpu: Option<GpuInfo>,
    /// Disk information
    pub disk: DiskInfo,
    /// Platform information
    pub platform: PlatformInfo,
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU model name
    pub name: String,
    /// Number of physical cores
    pub cores: usize,
    /// Number of logical threads
    pub threads: usize,
    /// CPU architecture
    pub arch: String,
    /// CPU features (AVX, AVX2, etc.)
    pub features: Vec<String>,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU name
    pub name: String,
    /// GPU vendor (nvidia, amd, intel, apple)
    pub vendor: GpuVendor,
    /// Total VRAM in bytes
    pub vram_bytes: u64,
    /// Available VRAM in bytes
    pub vram_available_bytes: u64,
    /// CUDA compute capability (for NVIDIA)
    pub cuda_version: Option<String>,
    /// Whether the GPU supports the required features
    pub supported: bool,
}

/// GPU vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Other,
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Total disk space in bytes
    pub total_bytes: u64,
    /// Available disk space in bytes
    pub available_bytes: u64,
    /// Path to data directory
    pub data_path: String,
}

/// Platform information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Operating system
    pub os: String,
    /// OS version
    pub os_version: String,
    /// Architecture
    pub arch: String,
}

/// Hardware detector
pub struct HardwareDetector;

impl HardwareDetector {
    /// Detect all hardware
    #[instrument]
    pub fn detect() -> ModelResult<HardwareInfo> {
        info!("Detecting hardware...");

        let cpu = Self::detect_cpu()?;
        debug!(
            "CPU: {} ({} cores, {} threads)",
            cpu.name, cpu.cores, cpu.threads
        );

        let (ram_bytes, ram_available_bytes) = Self::detect_ram()?;
        debug!(
            "RAM: {} total, {} available",
            format_bytes(ram_bytes),
            format_bytes(ram_available_bytes)
        );

        let gpu = Self::detect_gpu()?;
        if let Some(ref g) = gpu {
            debug!("GPU: {} ({} VRAM)", g.name, format_bytes(g.vram_bytes));
        } else {
            debug!("No supported GPU detected");
        }

        let disk = Self::detect_disk()?;
        debug!("Disk: {} available", format_bytes(disk.available_bytes));

        let platform = Self::detect_platform()?;
        debug!(
            "Platform: {} {} {}",
            platform.os, platform.os_version, platform.arch
        );

        Ok(HardwareInfo {
            cpu,
            ram_bytes,
            ram_available_bytes,
            gpu,
            disk,
            platform,
        })
    }

    /// Detect CPU information
    fn detect_cpu() -> ModelResult<CpuInfo> {
        let mut sys = System::new_all();
        sys.refresh_cpu();

        let cpus = sys.cpus();
        let threads = cpus.len();
        let cores = sys.physical_core_count().unwrap_or(threads);

        // Get CPU name from first processor
        let name = cpus
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let arch = std::env::consts::ARCH.to_string();

        // Detect CPU features
        let mut features = vec![];

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx") {
                features.push("AVX".to_string());
            }
            if is_x86_feature_detected!("avx2") {
                features.push("AVX2".to_string());
            }
            if is_x86_feature_detected!("avx512f") {
                features.push("AVX512".to_string());
            }
            if is_x86_feature_detected!("fma") {
                features.push("FMA".to_string());
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            features.push("NEON".to_string());
        }

        Ok(CpuInfo {
            name,
            cores,
            threads,
            arch,
            features,
        })
    }

    /// Detect RAM
    fn detect_ram() -> ModelResult<(u64, u64)> {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let total = sys.total_memory();
        let available = sys.available_memory();

        Ok((total, available))
    }

    /// Detect GPU
    fn detect_gpu() -> ModelResult<Option<GpuInfo>> {
        // Try NVIDIA first
        if let Some(gpu) = Self::detect_nvidia_gpu()? {
            return Ok(Some(gpu));
        }

        // Try AMD
        if let Some(gpu) = Self::detect_amd_gpu()? {
            return Ok(Some(gpu));
        }

        // Try Apple Silicon
        if let Some(gpu) = Self::detect_apple_gpu()? {
            return Ok(Some(gpu));
        }

        // Try Intel
        if let Some(gpu) = Self::detect_intel_gpu()? {
            return Ok(Some(gpu));
        }

        Ok(None)
    }

    /// Detect NVIDIA GPU using nvidia-smi
    fn detect_nvidia_gpu() -> ModelResult<Option<GpuInfo>> {
        let output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,memory.total,memory.free",
                "--format=csv,noheader,nounits",
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = stdout.lines().collect();

                if let Some(line) = lines.first() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        let name = parts[0].trim().to_string();
                        // nvidia-smi reports VRAM in MB, convert to bytes
                        let vram_total: u64 = parts[1].trim().parse().unwrap_or(0) * MB_TO_BYTES;
                        let vram_free: u64 = parts[2].trim().parse().unwrap_or(0) * MB_TO_BYTES;
                        let vram_available = vram_free.min(vram_total);

                        // Get CUDA version
                        let cuda_version = Self::get_cuda_version();

                        debug!(
                            "Detected NVIDIA GPU: {} with {} VRAM",
                            name,
                            format_bytes(vram_total)
                        );

                        return Ok(Some(GpuInfo {
                            name,
                            vendor: GpuVendor::Nvidia,
                            vram_bytes: vram_total,
                            vram_available_bytes: vram_available,
                            cuda_version,
                            supported: true,
                        }));
                    }
                }
            },
            Ok(output) => {
                debug!(
                    "nvidia-smi failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            },
            Err(e) => {
                debug!("nvidia-smi not found: {}", e);
            },
        }

        Ok(None)
    }

    /// Detect AMD GPU using rocm-smi
    fn detect_amd_gpu() -> ModelResult<Option<GpuInfo>> {
        let output = Command::new("rocm-smi")
            .args(["--showmeminfo", "vram", "--csv"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse rocm-smi output to get VRAM
                // The format varies, so we'll do a simple parse
                if stdout.contains("GPU") {
                    // Try to extract VRAM info
                    let vram_total = Self::parse_amd_vram(&stdout);

                    if vram_total > 0 {
                        debug!("Detected AMD GPU with {} VRAM", format_bytes(vram_total));

                        return Ok(Some(GpuInfo {
                            name: "AMD GPU".to_string(),
                            vendor: GpuVendor::Amd,
                            vram_bytes: vram_total,
                            vram_available_bytes: vram_total, // ROCm doesn't easily give available
                            cuda_version: None,
                            supported: true,
                        }));
                    }
                }
            },
            Ok(output) => {
                debug!(
                    "rocm-smi failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            },
            Err(e) => {
                debug!("rocm-smi not found: {}", e);
            },
        }

        Ok(None)
    }

    /// Detect Apple Silicon GPU
    fn detect_apple_gpu() -> ModelResult<Option<GpuInfo>> {
        #[cfg(target_os = "macos")]
        {
            if std::env::consts::ARCH == "aarch64" {
                // Apple Silicon - unified memory
                // We can get system memory which is also GPU memory
                let (ram_total, ram_available) = Self::detect_ram()?;

                debug!("Detected Apple Silicon GPU with unified memory");

                return Ok(Some(GpuInfo {
                    name: "Apple Silicon GPU".to_string(),
                    vendor: GpuVendor::Apple,
                    vram_bytes: ram_total,
                    vram_available_bytes: ram_available,
                    cuda_version: None,
                    supported: true,
                }));
            }
        }

        Ok(None)
    }

    /// Detect Intel GPU
    fn detect_intel_gpu() -> ModelResult<Option<GpuInfo>> {
        // Check for Intel oneAPI or Arc GPUs
        let output = Command::new("sycl-ls").arg("--devices").output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                if stdout.contains("GPU") || stdout.contains("Intel") {
                    debug!("Detected Intel GPU");

                    return Ok(Some(GpuInfo {
                        name: "Intel GPU".to_string(),
                        vendor: GpuVendor::Intel,
                        vram_bytes: 0, // Intel integrated GPUs use system memory
                        vram_available_bytes: 0,
                        cuda_version: None,
                        supported: true,
                    }));
                }
            },
            Ok(_) => {},
            Err(_) => {},
        }

        Ok(None)
    }

    /// Get CUDA version from nvidia-smi
    fn get_cuda_version() -> Option<String> {
        let output = Command::new("nvidia-smi")
            .args(["--query-gpu=cuda_version", "--format=csv,noheader"])
            .output()
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }

        // Fallback: try to get from driver version
        let output = Command::new("nvidia-smi").output().ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse "CUDA Version: 12.0" from output
            if let Some(captures) = stdout.lines().find_map(|line| {
                let line = line.to_lowercase();
                if line.contains("cuda version") {
                    Some(line.clone())
                } else {
                    None
                }
            }) {
                // Extract version number
                if let Some(version_start) = captures.find(|c: char| c.is_ascii_digit()) {
                    let version: String = captures
                        .chars()
                        .skip(version_start)
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    if !version.is_empty() {
                        return Some(version);
                    }
                }
            }
        }

        None
    }

    /// Parse AMD VRAM from rocm-smi output
    fn parse_amd_vram(output: &str) -> u64 {
        // Try to find VRAM total in MB
        for line in output.lines() {
            if line.contains("VRAM Total") || line.contains("Memory Total") {
                // Extract numbers
                let numbers: Vec<u64> = line
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();

                if let Some(&mb) = numbers.first() {
                    return mb * MB_TO_BYTES;
                }
            }
        }

        // Default to 8GB if AMD GPU detected but VRAM unknown
        DEFAULT_AMD_VRAM
    }

    /// Detect disk space
    fn detect_disk() -> ModelResult<DiskInfo> {
        let data_path = dirs::home_dir()
            .map(|p| p.join(".superinstance").to_string_lossy().to_string())
            .unwrap_or_else(|| "/tmp/superinstance".to_string());

        // Try to get real disk information using df command (Unix-like systems)
        #[cfg(unix)]
        {
            use std::path::Path;

            // Get the directory to check
            let check_path = Path::new(&data_path);
            if let Some(parent) = check_path.ancestors().next() {
                if let Ok(output) = Command::new("df")
                    .args(["-P", parent.as_os_str().to_str().unwrap_or(".")])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        // Parse df output: skip header, get second line
                        let lines: Vec<&str> = stdout.lines().collect();
                        if lines.len() > 1 {
                            let parts: Vec<&str> = lines[1].split_whitespace().collect();
                            if parts.len() >= 4 {
                                // df output: Filesystem 1K-blocks Used Available Capacity Mounted
                                if let (Ok(total_kb), Ok(available_kb)) =
                                    (parts[1].parse::<u64>(), parts[3].parse::<u64>())
                                {
                                    let total = total_kb * 1024;
                                    let available = available_kb * 1024;

                                    debug!(
                                        "Disk: {} total, {} available at {}",
                                        format_bytes(total),
                                        format_bytes(available),
                                        parts.last().unwrap_or(&"unknown")
                                    );

                                    return Ok(DiskInfo {
                                        total_bytes: total,
                                        available_bytes: available,
                                        data_path,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback for Windows or if df fails
        debug!("Using default disk information");
        Ok(DiskInfo {
            total_bytes: DEFAULT_DISK_TOTAL,
            available_bytes: DEFAULT_DISK_AVAILABLE,
            data_path,
        })
    }

    /// Detect platform
    fn detect_platform() -> ModelResult<PlatformInfo> {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        // Get OS version
        let os_version = Self::get_os_version(&os);

        Ok(PlatformInfo {
            os,
            os_version,
            arch,
        })
    }

    /// Get OS version string
    fn get_os_version(os: &str) -> String {
        match os {
            "linux" => {
                // Try /etc/os-release first
                if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                    for line in content.lines() {
                        if line.starts_with("PRETTY_NAME=") {
                            if let Some(version) = line.strip_prefix("PRETTY_NAME=") {
                                return version.trim_matches('"').to_string();
                            }
                        }
                    }
                }

                // Fallback to uname
                if let Ok(output) = Command::new("uname").arg("-r").output() {
                    if output.status.success() {
                        return format!(
                            "Linux kernel {}",
                            String::from_utf8_lossy(&output.stdout).trim()
                        );
                    }
                }

                "Linux (Unknown version)".to_string()
            },
            "macos" => {
                // Use sw_vers for macOS
                if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        return format!("macOS {}", version);
                    }
                }

                "macOS (Unknown version)".to_string()
            },
            "windows" => {
                // Use ver command for Windows
                if let Ok(output) = Command::new("cmd").args(["/C", "ver"]).output() {
                    if output.status.success() {
                        return String::from_utf8_lossy(&output.stdout).trim().to_string();
                    }
                }

                "Windows (Unknown version)".to_string()
            },
            _ => format!("{} (Unknown version)", os),
        }
    }
}

impl HardwareInfo {
    /// Check if hardware meets minimum requirements
    ///
    /// # Requirements
    /// - RAM: >= 8 GB
    /// - Disk: >= 10 GB available
    ///
    /// # Returns
    /// `true` if system meets all minimum requirements
    pub fn meets_minimum_requirements(&self) -> bool {
        self.ram_bytes >= MIN_RAM_BYTES
            && self.disk.available_bytes >= MIN_DISK_BYTES
    }

    /// Check if hardware can run a model of given size
    ///
    /// # Arguments
    /// * `model_size_bytes` - Size of the model in bytes
    /// * `needs_gpu` - Whether the model requires GPU acceleration
    ///
    /// # Returns
    /// `true` if the system has sufficient resources:
    /// - If `needs_gpu`: Checks GPU VRAM availability and GPU support
    /// - If CPU-only: Checks system RAM availability
    ///
    /// # Example
    /// ```ignore
    /// let hw = HardwareDetector::detect()?;
    /// // Can we run a 4GB model on GPU?
    /// let can_run = hw.can_run_model(4 * 1024 * 1024 * 1024, true);
    /// ```
    pub fn can_run_model(&self, model_size_bytes: u64, needs_gpu: bool) -> bool {
        if needs_gpu {
            if let Some(ref gpu) = self.gpu {
                gpu.supported && gpu.vram_available_bytes >= model_size_bytes
            } else {
                false
            }
        } else {
            self.ram_available_bytes >= model_size_bytes
        }
    }

    /// Get hardware tier (1-5, higher is better)
    ///
    /// # Tier Calculation
    ///
    /// **RAM-based tiers:**
    /// - Tier 4: >= 64 GB RAM
    /// - Tier 3: >= 32 GB RAM
    /// - Tier 2: >= 16 GB RAM
    /// - Tier 1: < 16 GB RAM
    ///
    /// **GPU-based tiers (can upgrade RAM tier):**
    /// - Tier 5: >= 24 GB VRAM
    /// - Tier 4: >= 12 GB VRAM
    /// - Tier 3: >= 8 GB VRAM
    /// - Tier 2: >= 4 GB VRAM
    ///
    /// The final tier is the maximum of RAM and GPU tiers.
    ///
    /// # Example
    /// ```ignore
    /// // 32 GB RAM + RTX 4090 (24 GB VRAM) = Tier 5
    /// // 16 GB RAM + RTX 3060 (12 GB VRAM) = Tier 4
    /// // 8 GB RAM + no GPU = Tier 1
    /// ```
    pub fn tier(&self) -> u8 {
        let mut tier = 1u8;

        // RAM tier
        let ram_gb = self.ram_bytes / (1024 * 1024 * 1024);
        if ram_gb >= TIER_RAM_HIGH {
            tier = tier.max(4);
        } else if ram_gb >= TIER_RAM_MID {
            tier = tier.max(3);
        } else if ram_gb >= TIER_RAM_LOW {
            tier = tier.max(2);
        }

        // GPU tier (can upgrade RAM tier)
        if let Some(ref gpu) = self.gpu {
            let vram_gb = gpu.vram_bytes / (1024 * 1024 * 1024);
            if vram_gb >= TIER_VRAM_HIGH {
                tier = tier.max(5);
            } else if vram_gb >= TIER_VRAM_MID {
                tier = tier.max(4);
            } else if vram_gb >= TIER_VRAM_LOW {
                tier = tier.max(3);
            } else if vram_gb >= TIER_VRAM_MIN {
                tier = tier.max(2);
            }
        }

        tier
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        let gpu_str = self
            .gpu
            .as_ref()
            .map(|g| format!("{} ({})", g.name, format_bytes(g.vram_bytes)))
            .unwrap_or_else(|| "None".to_string());

        format!(
            "CPU: {} ({} threads), RAM: {}, GPU: {}",
            self.cpu.name,
            self.cpu.threads,
            format_bytes(self.ram_bytes),
            gpu_str
        )
    }
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_detection() {
        let hw = HardwareDetector::detect().unwrap();
        assert!(hw.cpu.cores > 0);
        assert!(hw.cpu.threads > 0);
    }

    #[test]
    fn test_hardware_tier() {
        let hw = HardwareInfo {
            cpu: CpuInfo {
                name: "Test".to_string(),
                cores: 8,
                threads: 16,
                arch: "x86_64".to_string(),
                features: vec![],
            },
            ram_bytes: 32 * 1024 * 1024 * 1024,
            ram_available_bytes: 24 * 1024 * 1024 * 1024,
            gpu: Some(GpuInfo {
                name: "RTX 4090".to_string(),
                vendor: GpuVendor::Nvidia,
                vram_bytes: 24 * 1024 * 1024 * 1024,
                vram_available_bytes: 20 * 1024 * 1024 * 1024,
                cuda_version: Some("12.0".to_string()),
                supported: true,
            }),
            disk: DiskInfo {
                total_bytes: 1000 * 1024 * 1024 * 1024,
                available_bytes: 500 * 1024 * 1024 * 1024,
                data_path: "/home/test".to_string(),
            },
            platform: PlatformInfo {
                os: "linux".to_string(),
                os_version: "6.0".to_string(),
                arch: "x86_64".to_string(),
            },
        };

        assert_eq!(hw.tier(), 5);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }
}
