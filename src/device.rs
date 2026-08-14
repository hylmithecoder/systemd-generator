use std::path::Path;
use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitSystem {
    Systemd,
    OpenRC,
}

impl InitSystem {
    pub fn as_str(&self) -> &'static str {
        match self {
            InitSystem::Systemd => "Systemd",
            InitSystem::OpenRC => "OpenRC",
        }
    }

    pub fn detect() -> Self {
        if Path::new("/run/systemd/system").exists() {
            return InitSystem::Systemd;
        }

        if Path::new("/sbin/openrc").exists() || Path::new("/run/openrc").exists() {
            return InitSystem::OpenRC;
        }

        if let Ok(comm) = std::fs::read_to_string("/proc/1/comm") {
            let trimmed = comm.trim();
            if trimmed == "systemd" {
                return InitSystem::Systemd;
            } else if trimmed == "openrc-init" || trimmed == "openrc" {
                return InitSystem::OpenRC;
            }
        }

        InitSystem::Systemd
    }

    pub fn toggle(&self) -> Self {
        match self {
            InitSystem::Systemd => InitSystem::OpenRC,
            InitSystem::OpenRC => InitSystem::Systemd,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_arch: String,
    pub current_user: String,
    pub init_system: InitSystem,
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
        let init_system = InitSystem::detect();

        Self {
            hostname,
            os_name,
            os_version,
            kernel_version,
            cpu_arch,
            current_user,
            init_system,
        }
    }
}

