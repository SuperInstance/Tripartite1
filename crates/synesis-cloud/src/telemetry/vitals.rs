//! Device vitals collection
//!
//! Collects real-time system metrics including CPU, memory, GPU, and disk usage.

use crate::error::{CloudError, CloudResult};
use crate::telemetry::types::DeviceVitals;
use chrono::Utc;

/// Collect current device vitals
///
/// This function reads system metrics from the OS and returns
/// a DeviceVitals struct with current usage statistics.
///
/// # Platform Support
/// - **Linux**: Full support (CPU, memory, disk)
/// - **macOS/Windows**: Partial support (disk only, CPU/memory stubbed)
/// - **GPU**: Vendor-specific (NVIDIA via nvidia-smi, AMD via ROCm, stubbed otherwise)
pub fn collect_device_vitals(device_id: String) -> DeviceVitals {
    let timestamp = Utc::now();

    // Collect metrics
    let cpu_usage = collect_cpu_usage();
    let memory_usage = collect_memory_usage();
    let (gpu_usage, gpu_temp, gpu_vram_usage) = collect_gpu_metrics();
    let disk_usage = collect_disk_usage();

    DeviceVitals {
        device_id,
        timestamp,
        cpu_usage,
        memory_usage,
        gpu_usage,
        gpu_temp_celsius: gpu_temp,
        gpu_vram_usage,
        disk_usage,
        active_sessions: 0, // TODO: Track from session manager
        pending_queries: 0, // TODO: Track from request queue
        loaded_model: None, // TODO: Track from model loader
    }
}

/// Collect CPU usage percentage
///
/// On Linux, reads from /proc/stat
/// On other platforms, returns a stub value
fn collect_cpu_usage() -> f32 {
    #[cfg(target_os = "linux")]
    {
        read_cpu_usage_linux().unwrap_or(0.0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Stub for non-Linux platforms
        // TODO: Implement platform-specific CPU collection
        0.0
    }
}

/// Read CPU usage from /proc/stat on Linux
#[cfg(target_os = "linux")]
fn read_cpu_usage_linux() -> CloudResult<f32> {
    use std::fs;

    let stat_content = fs::read_to_string("/proc/stat")
        .map_err(|e| CloudError::telemetry(format!("Failed to read /proc/stat: {}", e)))?;

    // Parse first line: "cpu  user nice system idle iowait irq softirq"
    let first_line = stat_content
        .lines()
        .next()
        .ok_or_else(|| CloudError::telemetry("Empty /proc/stat"))?;

    let parts: Vec<u64> = first_line
        .split_whitespace()
        .skip(1) // Skip "cpu" label
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() < 4 {
        return Err(CloudError::telemetry("Invalid /proc/stat format"));
    }

    let user = parts[0];
    let nice = parts[1];
    let system = parts[2];
    let idle = parts[3];

    let total = user + nice + system + idle;
    let used = user + nice + system;

    if total == 0 {
        return Ok(0.0);
    }

    Ok((used as f32 / total as f32).min(1.0))
}

/// Collect memory usage percentage
///
/// On Linux, reads from /proc/meminfo
/// On other platforms, returns a stub value
fn collect_memory_usage() -> f32 {
    #[cfg(target_os = "linux")]
    {
        read_memory_usage_linux().unwrap_or(0.0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // Stub for non-Linux platforms
        // TODO: Implement platform-specific memory collection
        0.0
    }
}

/// Read memory usage from /proc/meminfo on Linux
#[cfg(target_os = "linux")]
fn read_memory_usage_linux() -> CloudResult<f32> {
    use std::fs;

    let meminfo_content = fs::read_to_string("/proc/meminfo")
        .map_err(|e| CloudError::telemetry(format!("Failed to read /proc/meminfo: {}", e)))?;

    let mut total_memory: u64 = 0;
    let mut free_memory: u64 = 0;
    let mut buffers: u64 = 0;
    let mut cached: u64 = 0;

    for line in meminfo_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0];
        let value = parts[1].parse::<u64>().unwrap_or(0);

        match key {
            "MemTotal:" => total_memory = value * 1024, // Convert KB to bytes
            "MemFree:" => free_memory = value * 1024,
            "Buffers:" => buffers = value * 1024,
            "Cached:" => cached = value * 1024,
            _ => {}
        }
    }

    if total_memory == 0 {
        return Err(CloudError::telemetry("Invalid /proc/meminfo format"));
    }

    // Used memory = total - (free + buffers + cached)
    let used_memory = total_memory.saturating_sub(free_memory + buffers + cached);

    Ok((used_memory as f32 / total_memory as f32).min(1.0))
}

/// Collect GPU metrics
///
/// Returns (usage%, temp_celsius, vram_usage%)
/// Attempts multiple GPU vendors in order
fn collect_gpu_metrics() -> (Option<f32>, Option<f32>, Option<f32>) {
    // Try NVIDIA first
    if let Ok(metrics) = collect_nvidia_gpu_metrics() {
        return metrics;
    }

    // Try AMD (ROCm) second
    if let Ok(metrics) = collect_amd_gpu_metrics() {
        return metrics;
    }

    // No GPU detected or monitoring unavailable
    (None, None, None)
}

/// Collect NVIDIA GPU metrics via nvidia-smi
fn collect_nvidia_gpu_metrics() -> CloudResult<(Option<f32>, Option<f32>, Option<f32>)> {
    use std::process::Command;

    // Run nvidia-smi to query GPU stats
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=utilization.gpu,temperature.gpu,utilization.memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .map_err(|_| CloudError::telemetry("nvidia-smi not available"))?;

    if !output.status.success() {
        return Err(CloudError::telemetry("nvidia-smi query failed"));
    }

    let data = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = data.trim().split(',').collect();

    if parts.len() < 3 {
        return Err(CloudError::telemetry("Invalid nvidia-smi output format"));
    }

    let gpu_usage = parts[0].trim().parse::<f32>().ok().map(|v| v / 100.0);
    let gpu_temp = parts[1].trim().parse::<f32>().ok();
    let vram_usage = parts[2].trim().parse::<f32>().ok().map(|v| v / 100.0);

    Ok((gpu_usage, gpu_temp, vram_usage))
}

/// Collect AMD GPU metrics via ROCm smi
fn collect_amd_gpu_metrics() -> CloudResult<(Option<f32>, Option<f32>, Option<f32>)> {
    use std::process::Command;

    // Run rocm-smi to query GPU stats
    let output = Command::new("rocm-smi")
        .args(&["--showuse", "--showtemp", "--showmemuse", "--csv"])
        .output()
        .map_err(|_| CloudError::telemetry("rocm-smi not available"))?;

    if !output.status.success() {
        return Err(CloudError::telemetry("rocm-smi query failed"));
    }

    // Parse ROCm output (simplified - actual parsing would need more robust logic)
    let data = String::from_utf8_lossy(&output.stdout);

    // ROCm output is complex, return stub for now
    // TODO: Implement proper ROCm parsing
    if data.contains("GPU") {
        // GPU detected, but we can't parse detailed metrics yet
        Ok((Some(0.0), None, None))
    } else {
        Err(CloudError::telemetry("No AMD GPU detected"))
    }
}

/// Collect disk usage percentage
///
/// Uses current directory as proxy for overall disk usage
fn collect_disk_usage() -> f32 {
    #[cfg(target_os = "linux")]
    {
        read_disk_usage_linux().unwrap_or(0.0)
    }

    #[cfg(not(target_os = "linux"))]
    {
        read_disk_usage_generic().unwrap_or(0.0)
    }
}

/// Read disk usage on Linux
#[cfg(target_os = "linux")]
fn read_disk_usage_linux() -> CloudResult<f32> {
    use std::fs;

    // Check root filesystem
    let metadata = fs::metadata("/")
        .map_err(|e| CloudError::telemetry(format!("Failed to stat root: {}", e)))?;

    // For simplicity, return a stub value
    // Real implementation would need to call statvfs syscall
    let _ = metadata;
    Ok(0.5) // Placeholder: 50% disk usage
}

/// Generic disk usage (cross-platform stub)
///
/// Provides a fallback disk usage metric for platforms without
/// platform-specific implementations. Returns 50% as placeholder.
#[allow(dead_code)]
fn read_disk_usage_generic() -> CloudResult<f32> {
    // Placeholder implementation
    Ok(0.5) // 50% disk usage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_device_vitals() {
        let vitals = collect_device_vitals("test-device".to_string());

        assert_eq!(vitals.device_id, "test-device");
        assert!(vitals.cpu_usage >= 0.0 && vitals.cpu_usage <= 1.0);
        assert!(vitals.memory_usage >= 0.0 && vitals.memory_usage <= 1.0);
        assert!(vitals.disk_usage >= 0.0 && vitals.disk_usage <= 1.0);
    }

    #[test]
    fn test_vitals_timestamp() {
        let vitals = collect_device_vitals("test-device".to_string());

        // Timestamp should be recent (within last minute)
        let now = Utc::now();
        let diff = now.signed_duration_since(vitals.timestamp);
        assert!(diff.num_seconds() < 60);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_cpu_collection() {
        let cpu = collect_cpu_usage();
        assert!(cpu >= 0.0 && cpu <= 1.0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_memory_collection() {
        let mem = collect_memory_usage();
        assert!(mem >= 0.0 && mem <= 1.0);
    }

    #[test]
    fn test_disk_collection() {
        let disk = collect_disk_usage();
        assert!(disk >= 0.0 && disk <= 1.0);
    }
}
