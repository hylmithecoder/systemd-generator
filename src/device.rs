use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_arch: String,
    pub current_user: String,
}

impl SystemInfo {
    pub fn detect() -> Self {
        let hostname = System::host_name().unwrap_or_else(|| "linux-node".to_string());
        let os_name = System::name().unwrap_or_else(|| "Linux".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
        let arch_raw = System::cpu_arch();
        let cpu_arch = if arch_raw.is_empty() {
            std::env::consts::ARCH.to_string()
        } else {
            arch_raw
        };
        let current_user = std::env::var("USER")
            .or_else(|_| std::env::var("LOGNAME"))
            .unwrap_or_else(|_| "root".to_string());

        Self {
            hostname,
            os_name,
            os_version,
            kernel_version,
            cpu_arch,
            current_user,
        }
    }
}
