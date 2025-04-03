use crate::{common::SysWrapper, renderer::core::Mouse};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct CpuStats {
    name: String,
    core_count: usize,
    thread_count: usize,
    overall_usage: f32,
    core_usage: Vec<f32>,
    tctl_temp: f32,
    tccd1_temp: f32,
    composite_temp: f32,
    min_usage: f32,
    max_usage: f32,
    min_tctl_temp: f32,
    max_tctl_temp: f32,
    min_tccd1_temp: f32,
    max_tccd1_temp: f32,
    min_composite_temp: f32,
    max_composite_temp: f32,
}

impl CpuStats {
    pub fn new() -> Self {
        CpuStats {
            name: String::from("Unknown CPU"),
            core_count: 0,
            thread_count: 0,
            overall_usage: 0.0,
            core_usage: Vec::new(),
            tctl_temp: 0.0,
            tccd1_temp: 0.0,
            composite_temp: 0.0,
            min_usage: 0.0,
            max_usage: 0.0,
            min_tctl_temp: 0.0,
            max_tctl_temp: 0.0,
            min_tccd1_temp: 0.0,
            max_tccd1_temp: 0.0,
            min_composite_temp: 0.0,
            max_composite_temp: 0.0,
        }
    }

    pub fn update(&mut self, sys_obj: &mut SysWrapper) {
        // Refresh CPU data
        sys_obj.sys.refresh_cpu_all();

        // Get CPU name
        let cpu_name = sys_obj
            .sys
            .cpus()
            .first()
            .map_or("Unknown CPU", |cpu| cpu.brand());
        self.name = cpu_name.to_string();

        // Get core count and thread count
        // Note: in sysinfo, the CPU count is actually the thread count
        self.thread_count = sys_obj.sys.cpus().len();
        // Estimate core count (may not be accurate for all CPUs)
        self.core_count = self.thread_count / 2;

        // Get overall CPU usage
        let global_usage = sys_obj.sys.global_cpu_usage();
        self.overall_usage = global_usage;

        // Update min/max for overall usage
        if self.min_usage == 0.0 || self.overall_usage < self.min_usage {
            self.min_usage = self.overall_usage;
        }
        if self.overall_usage > self.max_usage {
            self.max_usage = self.overall_usage;
        }

        // Get per-core usage
        self.core_usage = sys_obj
            .sys
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .collect();

        // Update temperature information
        self.update_temperatures();
    }

    fn update_temperatures(&mut self) {
        // Try to get temps from various sources based on CPU type
        // First, try lm_sensors style paths for AMD CPUs
        self.update_amd_temperatures();

        // If we didn't get temps from AMD, try Intel sensors
        if self.tctl_temp == 0.0 {
            self.update_intel_temperatures();
        }

        // Update min/max temperatures
        if self.tctl_temp > 0.0 {
            if self.min_tctl_temp == 0.0 || self.tctl_temp < self.min_tctl_temp {
                self.min_tctl_temp = self.tctl_temp;
            }
            if self.tctl_temp > self.max_tctl_temp {
                self.max_tctl_temp = self.tctl_temp;
            }
        }

        if self.tccd1_temp > 0.0 {
            if self.min_tccd1_temp == 0.0 || self.tccd1_temp < self.min_tccd1_temp {
                self.min_tccd1_temp = self.tccd1_temp;
            }
            if self.tccd1_temp > self.max_tccd1_temp {
                self.max_tccd1_temp = self.tccd1_temp;
            }
        }

        if self.composite_temp > 0.0 {
            if self.min_composite_temp == 0.0 || self.composite_temp < self.min_composite_temp {
                self.min_composite_temp = self.composite_temp;
            }
            if self.composite_temp > self.max_composite_temp {
                self.max_composite_temp = self.composite_temp;
            }
        }
    }

    fn update_amd_temperatures(&mut self) {
        // For AMD CPUs, check k10temp sensors
        let hwmon_path = Path::new("/sys/class/hwmon");
        if !hwmon_path.exists() {
            return;
        }

        if let Ok(hwmon_entries) = std::fs::read_dir(hwmon_path) {
            for entry in hwmon_entries.filter_map(Result::ok) {
                let hwmon_dir = entry.path();

                // Check if this is an AMD k10temp sensor
                let name_path = hwmon_dir.join("name");
                if name_path.exists() {
                    if let Ok(name) = read_file_to_string(&name_path) {
                        let name = name.trim();

                        // Look specifically for k10temp which contains AMD CPU sensors
                        if name == "k10temp" {
                            // Based on Psensor screenshot, looking for Tctl and tccd1

                            // Check for Tctl (usually temp1_input)
                            let tctl_path = hwmon_dir.join("temp1_input");
                            if tctl_path.exists() {
                                if let Ok(temp_str) = read_file_to_string(&tctl_path) {
                                    if let Ok(temp_millicelsius) = temp_str.trim().parse::<f32>() {
                                        self.tctl_temp = temp_millicelsius / 1000.0;
                                    }
                                }
                            }

                            // Check specifically for tccd1 (usually temp2_input but can be in other locations)
                            // Try multiple possible paths
                            let possible_tccd1_paths = [
                                hwmon_dir.join("temp2_input"),
                                hwmon_dir.join("temp3_input"),
                                hwmon_dir.join("temp4_input"),
                            ];

                            // Try each path
                            for path in &possible_tccd1_paths {
                                if path.exists() {
                                    // Check if this is tccd1 by looking for a label file
                                    let label_path = path.with_file_name(
                                        path.file_name()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .replace("_input", "_label"),
                                    );

                                    let is_tccd1 = if label_path.exists() {
                                        // If we have a label file, check if it contains "Tccd1"
                                        if let Ok(label) = read_file_to_string(&label_path) {
                                            label.to_lowercase().contains("ccd1")
                                                || label.to_lowercase().contains("tccd1")
                                        } else {
                                            false
                                        }
                                    } else {
                                        // If no label, assume temp2_input is tccd1 (common for k10temp)
                                        path.to_str().unwrap().ends_with("temp2_input")
                                    };

                                    if is_tccd1 {
                                        if let Ok(temp_str) = read_file_to_string(path) {
                                            if let Ok(temp_millicelsius) =
                                                temp_str.trim().parse::<f32>()
                                            {
                                                self.tccd1_temp = temp_millicelsius / 1000.0;
                                                break;
                                            }
                                        }
                                    }
                                }
                            }

                            // Use Tctl as composite if we don't have a better value
                            self.composite_temp = self.tctl_temp;
                            break;
                        }
                    }
                }
            }
        }
    }

    fn update_intel_temperatures(&mut self) {
        // For Intel CPUs, check coretemp sensors
        let hwmon_path = Path::new("/sys/class/hwmon");
        if !hwmon_path.exists() {
            return;
        }

        if let Ok(hwmon_entries) = std::fs::read_dir(hwmon_path) {
            for entry in hwmon_entries.filter_map(Result::ok) {
                let hwmon_dir = entry.path();

                // Check if this is an Intel coretemp sensor
                let name_path = hwmon_dir.join("name");
                if name_path.exists() {
                    if let Ok(name) = read_file_to_string(&name_path) {
                        let name = name.trim();

                        if name == "coretemp" {
                            // This is the Intel CPU temp sensor
                            // Package temperature (similar to composite)
                            let package_path = hwmon_dir.join("temp1_input");
                            if package_path.exists() {
                                if let Ok(temp_str) = read_file_to_string(&package_path) {
                                    if let Ok(temp_millicelsius) = temp_str.trim().parse::<f32>() {
                                        self.composite_temp = temp_millicelsius / 1000.0;
                                        // For Intel, we'll use package temp as Tctl equivalent
                                        self.tctl_temp = self.composite_temp;
                                    }
                                }
                            }

                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn get_mouse(&self) -> Mouse {
        let title = String::from("CPU");
        let mut mouse = Mouse::new(title);

        // Add CPU model name
        mouse.add(format!("Model: {}", self.name));

        // Add core and thread count
        mouse.add(format!(
            "Cores: {}, Threads: {}",
            self.core_count, self.thread_count
        ));

        // Add overall CPU usage with min/max
        mouse.add(format!(
            "Usage: {:.1}% (Min: {:.1}%, Max: {:.1}%)",
            self.overall_usage, self.min_usage, self.max_usage
        ));

        // Add temperature information if available
        if self.tctl_temp > 0.0 {
            mouse.add(format!(
                "Tctl: {:.1}°C (Min: {:.1}°C, Max: {:.1}°C)",
                self.tctl_temp, self.min_tctl_temp, self.max_tctl_temp
            ));
        }

        if self.tccd1_temp > 0.0 {
            mouse.add(format!(
                "Tccd1: {:.1}°C (Min: {:.1}°C, Max: {:.1}°C)",
                self.tccd1_temp, self.min_tccd1_temp, self.max_tccd1_temp
            ));
        }

        if self.composite_temp > 0.0 && self.composite_temp != self.tctl_temp {
            mouse.add(format!(
                "Composite: {:.1}°C (Min: {:.1}°C, Max: {:.1}°C)",
                self.composite_temp, self.min_composite_temp, self.max_composite_temp
            ));
        }

        // Add per-core usage information (limited to first 16 cores to keep UI manageable)
        if !self.core_usage.is_empty() {
            mouse.add(String::from("")); // Empty line to separate
            mouse.add(String::from("Per-core Usage:"));

            // Display cores in groups of 4 to save vertical space
            let mut core_groups = Vec::new();
            for chunk in self.core_usage.chunks(4) {
                let mut group_str = String::new();
                for (i, usage) in chunk.iter().enumerate() {
                    let core_idx = core_groups.len() * 4 + i;
                    if !group_str.is_empty() {
                        group_str.push_str(", ");
                    }
                    group_str.push_str(&format!("CPU{}: {:.1}%", core_idx, usage));
                }
                core_groups.push(group_str);
            }

            // Add each group as a line (max 4 lines for 16 cores)
            for (i, group) in core_groups.iter().take(4).enumerate() {
                mouse.add(group.clone());
            }

            // If there are more cores, indicate that
            if self.core_usage.len() > 16 {
                mouse.add(format!("... and {} more cores", self.core_usage.len() - 16));
            }
        }

        return mouse;
    }
}

// Helper function to read a file into a string (reused from gpu.rs)
fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
