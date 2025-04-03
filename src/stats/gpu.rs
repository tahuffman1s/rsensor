use crate::{
    common::{get_default_byte, SysWrapper},
    renderer::core::Mouse,
};
use byte_unit::{AdjustedByte, Byte, UnitType};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone)]
pub struct GpuStats {
    gpus: Vec<GpuInfo>,
    // Track previous stats for min/max values
    previous_gpu_states: Vec<GpuMinMaxStats>,
}

#[derive(Clone)]
struct GpuMinMaxStats {
    id: String,
    min_usage_percent: f64,
    max_usage_percent: f64,
    min_memory_percent: f64,
    max_memory_percent: f64,
    min_edge_temp: f64,
    max_edge_temp: f64,
    min_junction_temp: f64,
    max_junction_temp: f64,
    min_memory_temp: f64,
    max_memory_temp: f64,
}

#[derive(Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    // Temperatures
    pub edge_temp: f64,
    pub junction_temp: f64,
    pub memory_temp: f64,
    // Usage
    pub usage_percent: f64,
    pub memory_total: AdjustedByte,
    pub memory_used: AdjustedByte,
    pub memory_percent: f64,
}

#[derive(Clone, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Unknown,
}

impl GpuVendor {
    pub fn as_string(&self) -> &str {
        match self {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Amd => "AMD",
            GpuVendor::Unknown => "Unknown",
        }
    }
}

impl GpuStats {
    pub fn new() -> Self {
        GpuStats {
            gpus: Vec::new(),
            previous_gpu_states: Vec::new(),
        }
    }

    pub fn update(&mut self, sys_obj: &mut SysWrapper) {
        // Clear existing GPU info
        let previous_gpus = self.gpus.clone();
        self.gpus.clear();

        // Detect and update NVIDIA GPUs
        self.update_nvidia_gpus();

        // Detect and update AMD GPUs
        self.update_amd_gpus();

        // Update min/max statistics for each GPU
        for gpu in &self.gpus {
            let gpu_id = format!("{} {}", gpu.vendor.as_string(), &gpu.name);

            // Find existing stats or create new ones
            let stats_index = self.previous_gpu_states.iter().position(|s| s.id == gpu_id);

            if let Some(index) = stats_index {
                // Update existing stats
                let stats = &mut self.previous_gpu_states[index];

                // Update usage percent min/max
                if gpu.usage_percent < stats.min_usage_percent || stats.min_usage_percent == 0.0 {
                    stats.min_usage_percent = gpu.usage_percent;
                }
                if gpu.usage_percent > stats.max_usage_percent {
                    stats.max_usage_percent = gpu.usage_percent;
                }

                // Update memory percent min/max
                if gpu.memory_percent < stats.min_memory_percent || stats.min_memory_percent == 0.0
                {
                    stats.min_memory_percent = gpu.memory_percent;
                }
                if gpu.memory_percent > stats.max_memory_percent {
                    stats.max_memory_percent = gpu.memory_percent;
                }

                // Update temperature min/max
                if gpu.edge_temp < stats.min_edge_temp || stats.min_edge_temp == 0.0 {
                    stats.min_edge_temp = gpu.edge_temp;
                }
                if gpu.edge_temp > stats.max_edge_temp {
                    stats.max_edge_temp = gpu.edge_temp;
                }

                if gpu.junction_temp < stats.min_junction_temp || stats.min_junction_temp == 0.0 {
                    stats.min_junction_temp = gpu.junction_temp;
                }
                if gpu.junction_temp > stats.max_junction_temp {
                    stats.max_junction_temp = gpu.junction_temp;
                }

                if gpu.memory_temp < stats.min_memory_temp || stats.min_memory_temp == 0.0 {
                    stats.min_memory_temp = gpu.memory_temp;
                }
                if gpu.memory_temp > stats.max_memory_temp {
                    stats.max_memory_temp = gpu.memory_temp;
                }
            } else {
                // Create new stats
                self.previous_gpu_states.push(GpuMinMaxStats {
                    id: gpu_id,
                    min_usage_percent: gpu.usage_percent,
                    max_usage_percent: gpu.usage_percent,
                    min_memory_percent: gpu.memory_percent,
                    max_memory_percent: gpu.memory_percent,
                    min_edge_temp: gpu.edge_temp,
                    max_edge_temp: gpu.edge_temp,
                    min_junction_temp: gpu.junction_temp,
                    max_junction_temp: gpu.junction_temp,
                    min_memory_temp: gpu.memory_temp,
                    max_memory_temp: gpu.memory_temp,
                });
            }
        }

        // Clean up stats for removed GPUs
        self.previous_gpu_states.retain(|stats| {
            self.gpus.iter().any(|gpu| {
                let gpu_id = format!("{} {}", gpu.vendor.as_string(), &gpu.name);
                stats.id == gpu_id
            })
        });
    }

    fn update_nvidia_gpus(&mut self) {
        // Check if nvidia-smi is available and get GPU information from it
        if let Ok(output) = Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 5 {
                        let name = parts[0].to_string();
                        let edge_temp = parts[1].parse::<f64>().unwrap_or_default();
                        let usage_percent = parts[2].parse::<f64>().unwrap_or_default();
                        let memory_used_mb = parts[3].parse::<u64>().unwrap_or_default();
                        let memory_total_mb = parts[4].parse::<u64>().unwrap_or_default();

                        // Convert MB to bytes for the byte-unit library
                        let memory_used = Byte::from_u64(memory_used_mb * 1024 * 1024)
                            .get_appropriate_unit(UnitType::Binary);
                        let memory_total = Byte::from_u64(memory_total_mb * 1024 * 1024)
                            .get_appropriate_unit(UnitType::Binary);
                        let memory_percent = if memory_total_mb > 0 {
                            (memory_used_mb as f64 / memory_total_mb as f64) * 100.0
                        } else {
                            0.0
                        };

                        // NVIDIA doesn't report junction and memory temps through basic SMI
                        // commands, so we use the same value for all or leave them at 0
                        let gpu_info = GpuInfo {
                            name,
                            vendor: GpuVendor::Nvidia,
                            edge_temp,
                            junction_temp: edge_temp, // Use the same value for junction temp
                            memory_temp: 0.0,         // We don't have memory temp from nvidia-smi
                            usage_percent,
                            memory_total,
                            memory_used,
                            memory_percent,
                        };

                        self.gpus.push(gpu_info);
                    }
                }
            }
        }
    }

    fn update_amd_gpus(&mut self) {
        // For AMD GPUs, we need to read from sysfs entries
        // In Linux, AMD GPU information is available in /sys/class/drm/card*/ directories

        // Find all AMD GPU directories
        let drm_path = Path::new("/sys/class/drm");
        if !drm_path.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(drm_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                let filename = path.file_name().unwrap_or_default().to_str().unwrap_or("");

                // Check if this is a card directory (e.g., card0, card1)
                if filename.starts_with("card") && path.is_dir() {
                    // Check if this is an AMD GPU
                    let vendor_path = path.join("device/vendor");
                    if let Ok(vendor) = read_file_to_string(&vendor_path) {
                        // AMD vendor ID is 0x1002
                        if vendor.trim() == "0x1002" {
                            self.add_amd_gpu_info(&path);
                        }
                    }
                }
            }
        }
    }

    fn add_amd_gpu_info(&mut self, card_path: &Path) {
        // Get GPU name using lspci which generally shows the marketing name
        let mut gpu_name = String::from("AMD GPU");

        // Try to get the PCI bus ID from the device path
        if let Ok(bus_id) = get_pci_bus_id_from_path(card_path) {
            // Use lspci to get the full device name
            if let Ok(output) = Command::new("lspci").args(["-s", &bus_id, "-vnn"]).output() {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Parse the output to extract the GPU name
                    if let Some(name) = extract_gpu_name_from_lspci(&output_str) {
                        gpu_name = name;
                    }
                }
            }
        }

        // If lspci didn't work, try the device/product file as fallback
        if gpu_name == "AMD GPU" {
            let name_path = card_path.join("device/product");
            if let Ok(name) = read_file_to_string(&name_path) {
                let name = name.trim();
                if !name.is_empty() {
                    gpu_name = name.to_string();
                }
            }
        }

        // Initialize GPU info
        let default_byte = *get_default_byte();
        let mut gpu_info = GpuInfo {
            name: gpu_name,
            vendor: GpuVendor::Amd,
            edge_temp: 0.0,
            junction_temp: 0.0,
            memory_temp: 0.0,
            usage_percent: 0.0,
            memory_total: default_byte,
            memory_used: default_byte,
            memory_percent: 0.0,
        };

        // Get temperatures
        // For AMD, temperatures are usually in millidegrees Celsius
        let hwmon_path = card_path.join("device/hwmon");
        if hwmon_path.exists() {
            if let Ok(hwmon_entries) = std::fs::read_dir(&hwmon_path) {
                for hwmon_entry in hwmon_entries.filter_map(Result::ok) {
                    let hwmon_dir = hwmon_entry.path();

                    // Edge temperature (GPU temperature)
                    let edge_temp_path = hwmon_dir.join("temp1_input");
                    if edge_temp_path.exists() {
                        if let Ok(temp_str) = read_file_to_string(&edge_temp_path) {
                            if let Ok(temp_millicelsius) = temp_str.trim().parse::<f64>() {
                                gpu_info.edge_temp = temp_millicelsius / 1000.0;
                            }
                        }
                    }

                    // Junction temperature (Hotspot)
                    let junction_temp_path = hwmon_dir.join("temp2_input");
                    if junction_temp_path.exists() {
                        if let Ok(temp_str) = read_file_to_string(&junction_temp_path) {
                            if let Ok(temp_millicelsius) = temp_str.trim().parse::<f64>() {
                                gpu_info.junction_temp = temp_millicelsius / 1000.0;
                            }
                        }
                    }

                    // Memory temperature
                    let memory_temp_path = hwmon_dir.join("temp3_input");
                    if memory_temp_path.exists() {
                        if let Ok(temp_str) = read_file_to_string(&memory_temp_path) {
                            if let Ok(temp_millicelsius) = temp_str.trim().parse::<f64>() {
                                gpu_info.memory_temp = temp_millicelsius / 1000.0;
                            }
                        }
                    }
                }
            }
        }

        // Get memory usage
        // For AMD, memory usage is available in /sys/class/drm/card*/device/mem_info_vram_*
        let vram_total_path = card_path.join("device/mem_info_vram_total");
        let vram_used_path = card_path.join("device/mem_info_vram_used");

        if vram_total_path.exists() && vram_used_path.exists() {
            if let Ok(total_str) = read_file_to_string(&vram_total_path) {
                if let Ok(total_bytes) = total_str.trim().parse::<u64>() {
                    gpu_info.memory_total =
                        Byte::from_u64(total_bytes).get_appropriate_unit(UnitType::Binary);

                    if let Ok(used_str) = read_file_to_string(&vram_used_path) {
                        if let Ok(used_bytes) = used_str.trim().parse::<u64>() {
                            gpu_info.memory_used =
                                Byte::from_u64(used_bytes).get_appropriate_unit(UnitType::Binary);
                            gpu_info.memory_percent =
                                (used_bytes as f64 / total_bytes as f64) * 100.0;
                        }
                    }
                }
            }
        }

        // Get GPU usage
        // For AMD, GPU usage is available at /sys/class/drm/card*/device/gpu_busy_percent
        let gpu_busy_path = card_path.join("device/gpu_busy_percent");
        if gpu_busy_path.exists() {
            if let Ok(busy_str) = read_file_to_string(&gpu_busy_path) {
                if let Ok(busy_percent) = busy_str.trim().parse::<f64>() {
                    gpu_info.usage_percent = busy_percent;
                }
            }
        }

        self.gpus.push(gpu_info);
    }

    pub fn get_gpus(&self) -> &Vec<GpuInfo> {
        &self.gpus
    }

    pub fn get_mouse(&self) -> Mouse {
        let title = String::from("GPU");
        let mut mouse = Mouse::new(title);

        if self.gpus.is_empty() {
            mouse.add(String::from("No GPUs detected"));
            return mouse;
        }

        for (i, gpu) in self.gpus.iter().enumerate() {
            if i > 0 {
                mouse.add(String::from("")); // Add empty line between GPUs
            }

            // Shorten the vendor name prefix to save space
            let display_name = gpu
                .name
                .replace("Advanced Micro Devices, Inc. [AMD/ATI]", "AMD");

            // Get the min/max stats for this GPU
            let gpu_id = format!("{} {}", gpu.vendor.as_string(), &gpu.name);
            let stats = self.previous_gpu_states.iter().find(|s| s.id == gpu_id);

            // Split model name into own line if it's long
            mouse.add(format!("GPU {}: {}", i + 1, display_name));

            // Add temperature information with min/max
            if let Some(stats) = stats {
                mouse.add(format!(
                    "Temp: Edge: {:.1}°C (Min: {:.1}°C, Max: {:.1}°C)",
                    gpu.edge_temp, stats.min_edge_temp, stats.max_edge_temp
                ));

                if gpu.junction_temp > 0.0 {
                    mouse.add(format!(
                        "Junct: {:.1}°C (Min: {:.1}°C, Max: {:.1}°C)",
                        gpu.junction_temp, stats.min_junction_temp, stats.max_junction_temp
                    ));
                }

                if gpu.memory_temp > 0.0 {
                    mouse.add(format!(
                        "Mem Temp: {:.1}°C (Min: {:.1}°C, Max: {:.1}°C)",
                        gpu.memory_temp, stats.min_memory_temp, stats.max_memory_temp
                    ));
                }

                // Add GPU utilization with min/max
                mouse.add(format!(
                    "GPU Usage: {:.1}% (Min: {:.1}%, Max: {:.1}%)",
                    gpu.usage_percent, stats.min_usage_percent, stats.max_usage_percent
                ));

                // Add memory information with min/max percentage
                mouse.add(format!(
                    "Memory: {:.2}{}/{:.2}{} ({:.1}%)",
                    gpu.memory_used.get_value(),
                    gpu.memory_used.get_unit(),
                    gpu.memory_total.get_value(),
                    gpu.memory_total.get_unit(),
                    gpu.memory_percent
                ));

                mouse.add(format!(
                    "Mem Usage: Min: {:.1}%, Max: {:.1}%",
                    stats.min_memory_percent, stats.max_memory_percent
                ));
            } else {
                // Fallback if we don't have min/max stats yet
                mouse.add(format!(
                    "Temp: Edge: {:.1}°C, Junction: {:.1}°C, Memory: {:.1}°C",
                    gpu.edge_temp, gpu.junction_temp, gpu.memory_temp
                ));

                mouse.add(format!("GPU Usage: {:.1}%", gpu.usage_percent));

                mouse.add(format!(
                    "Memory: {:.2}{}/{:.2}{} ({:.1}%)",
                    gpu.memory_used.get_value(),
                    gpu.memory_used.get_unit(),
                    gpu.memory_total.get_value(),
                    gpu.memory_total.get_unit(),
                    gpu.memory_percent
                ));
            }
        }

        return mouse;
    }
}

// Helper function to read a file into a string
fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Extract the PCI bus ID from a device path
fn get_pci_bus_id_from_path(card_path: &Path) -> io::Result<String> {
    // Read the device link to get the PCI path
    let device_link_path = card_path.join("device");
    let device_path = std::fs::read_link(device_link_path)?;

    // Extract the PCI bus ID from a path like:
    // "/sys/devices/pci0000:00/0000:00:03.0/0000:03:00.0"
    let path_str = device_path.to_string_lossy();

    // Find the last PCI address in the path (should be the GPU's address)
    if let Some(last_address) = path_str
        .split('/')
        .filter(|s| s.starts_with("0000:"))
        .last()
    {
        return Ok(last_address.trim_start_matches("0000:").to_string());
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "PCI bus ID not found",
    ))
}

// Extract the GPU name from lspci output
fn extract_gpu_name_from_lspci(lspci_output: &str) -> Option<String> {
    for line in lspci_output.lines() {
        // Look for VGA or Display controller lines
        if line.contains("VGA compatible controller") || line.contains("Display controller") {
            // The format is typically:
            // "XX:XX.X VGA compatible controller [XXXX:XXXX]: AMD Radeon RX XXXX (rev XX)"
            if line.contains("AMD") || line.contains("ATI") {
                // Extract just the model name part
                if let Some(colon_pos) = line.find(": ") {
                    let name_part = &line[colon_pos + 2..];

                    // Strip revision info if present
                    if let Some(rev_pos) = name_part.find(" (rev ") {
                        return Some(name_part[..rev_pos].trim().to_string());
                    } else {
                        return Some(name_part.trim().to_string());
                    }
                }
            }
        }
    }

    None
}
