use crate::device::{InitSystem, SystemInfo};
use crate::generator::{deploy_service_by_type, render_service_by_type, ServiceConfig};
use crate::picker::{resolve_binary_info, BinaryLocationInfo};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveStep {
    BinaryPicker,
    ServiceConfig,
    PreviewDeploy,
    StatusModal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    UnitName,
    Description,
    ExecStart,
    WorkingDirectory,
    User,
    Group,
    RestartPolicy,
    ExtraFlags,
    Environment,
}

impl ConfigField {
    pub const ALL: &'static [ConfigField] = &[
        ConfigField::UnitName,
        ConfigField::Description,
        ConfigField::ExecStart,
        ConfigField::WorkingDirectory,
        ConfigField::User,
        ConfigField::Group,
        ConfigField::RestartPolicy,
        ConfigField::ExtraFlags,
        ConfigField::Environment,
    ];

    pub fn next(&self) -> Self {
        let idx = ConfigField::ALL.iter().position(|f| f == self).unwrap_or(0);
        ConfigField::ALL[(idx + 1) % ConfigField::ALL.len()]
    }

    pub fn prev(&self) -> Self {
        let idx = ConfigField::ALL.iter().position(|f| f == self).unwrap_or(0);
        if idx == 0 {
            ConfigField::ALL[ConfigField::ALL.len() - 1]
        } else {
            ConfigField::ALL[idx - 1]
        }
    }
}

pub struct App {
    pub step: ActiveStep,
    pub binary_path_input: String,
    pub binary_info: Option<BinaryLocationInfo>,
    pub sys_info: SystemInfo,
    pub target_init_system: InitSystem,
    pub config: Option<ServiceConfig>,
    pub active_config_field: ConfigField,
    pub generated_service_content: String,
    pub status_message: String,
    pub is_error_status: bool,
    pub preview_action_index: usize, // 0 = Save Local, 1 = Deploy systemwide
}

impl App {
    pub fn new() -> Self {
        let sys_info = SystemInfo::detect();
        let target_init_system = sys_info.init_system;
        Self {
            step: ActiveStep::BinaryPicker,
            binary_path_input: String::new(),
            binary_info: None,
            sys_info,
            target_init_system,
            config: None,
            active_config_field: ConfigField::UnitName,
            generated_service_content: String::new(),
            status_message: String::new(),
            is_error_status: false,
            preview_action_index: 0,
        }
    }

    pub fn toggle_init_system(&mut self) {
        self.target_init_system = self.target_init_system.toggle();
        self.refresh_generated_content();
    }

    pub fn update_binary_info(&mut self) {
        if let Some(info) = resolve_binary_info(&self.binary_path_input) {
            let mut cfg = ServiceConfig::new(
                info.app_name.clone(),
                info.binary_name.clone(),
                info.source_path.clone(),
                self.sys_info.current_user.clone(),
            );

            // Inherit pre-calculated values
            cfg.exec_start = info.opt_binary_path.to_string_lossy().to_string();
            cfg.working_dir = info.opt_dir.to_string_lossy().to_string();

            self.binary_info = Some(info);
            self.config = Some(cfg);
        } else {
            self.binary_info = None;
            self.config = None;
        }
    }

    pub fn refresh_generated_content(&mut self) {
        if let Some(cfg) = &self.config {
            self.generated_service_content =
                render_service_by_type(self.target_init_system, cfg, &self.sys_info);
        }
    }

    pub fn handle_char_input(&mut self, c: char) {
        match self.step {
            ActiveStep::BinaryPicker => {
                self.binary_path_input.push(c);
                self.update_binary_info();
            }
            ActiveStep::ServiceConfig => {
                if let Some(cfg) = &mut self.config {
                    match self.active_config_field {
                        ConfigField::UnitName => cfg.unit_name.push(c),
                        ConfigField::Description => cfg.description.push(c),
                        ConfigField::ExecStart => cfg.exec_start.push(c),
                        ConfigField::WorkingDirectory => cfg.working_dir.push(c),
                        ConfigField::User => cfg.user.push(c),
                        ConfigField::Group => cfg.group.push(c),
                        ConfigField::RestartPolicy => cfg.restart_policy.push(c),
                        ConfigField::ExtraFlags => cfg.extra_flags.push(c),
                        ConfigField::Environment => cfg.environment.push(c),
                    }
                    self.refresh_generated_content();
                }
            }
            _ => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.step {
            ActiveStep::BinaryPicker => {
                self.binary_path_input.pop();
                self.update_binary_info();
            }
            ActiveStep::ServiceConfig => {
                if let Some(cfg) = &mut self.config {
                    match self.active_config_field {
                        ConfigField::UnitName => {
                            cfg.unit_name.pop();
                        }
                        ConfigField::Description => {
                            cfg.description.pop();
                        }
                        ConfigField::ExecStart => {
                            cfg.exec_start.pop();
                        }
                        ConfigField::WorkingDirectory => {
                            cfg.working_dir.pop();
                        }
                        ConfigField::User => {
                            cfg.user.pop();
                        }
                        ConfigField::Group => {
                            cfg.group.pop();
                        }
                        ConfigField::RestartPolicy => {
                            cfg.restart_policy.pop();
                        }
                        ConfigField::ExtraFlags => {
                            cfg.extra_flags.pop();
                        }
                        ConfigField::Environment => {
                            cfg.environment.pop();
                        }
                    }
                    self.refresh_generated_content();
                }
            }
            _ => {}
        }
    }

    pub fn next_field_or_action(&mut self) {
        if self.step == ActiveStep::ServiceConfig {
            self.active_config_field = self.active_config_field.next();
        } else if self.step == ActiveStep::PreviewDeploy {
            self.preview_action_index = (self.preview_action_index + 1) % 2;
        }
    }

    pub fn prev_field_or_action(&mut self) {
        if self.step == ActiveStep::ServiceConfig {
            self.active_config_field = self.active_config_field.prev();
        } else if self.step == ActiveStep::PreviewDeploy {
            if self.preview_action_index == 0 {
                self.preview_action_index = 1;
            } else {
                self.preview_action_index = 0;
            }
        }
    }

    pub fn proceed_step(&mut self) {
        match self.step {
            ActiveStep::BinaryPicker => {
                if self.binary_path_input.trim().is_empty() {
                    self.status_message = "Please enter a binary path first!".to_string();
                    self.is_error_status = true;
                    self.step = ActiveStep::StatusModal;
                    return;
                }
                self.update_binary_info();
                self.refresh_generated_content();
                self.step = ActiveStep::ServiceConfig;
            }
            ActiveStep::ServiceConfig => {
                self.refresh_generated_content();
                self.step = ActiveStep::PreviewDeploy;
            }
            ActiveStep::PreviewDeploy => {
                // Execute chosen action
                if self.preview_action_index == 0 {
                    // Save local
                    if let Some(cfg) = &self.config {
                        let filename = match self.target_init_system {
                            InitSystem::Systemd => {
                                if cfg.unit_name.ends_with(".service") {
                                    cfg.unit_name.clone()
                                } else {
                                    format!("{}.service", cfg.unit_name)
                                }
                            }
                            InitSystem::OpenRC => cfg.app_name.clone(),
                        };
                        let save_path = PathBuf::from(&filename);
                        match crate::generator::save_service_file(
                            &save_path,
                            &self.generated_service_content,
                        ) {
                            Ok(_) => {
                                self.status_message = format!(
                                    "Service file saved successfully to: {}",
                                    save_path.display()
                                );
                                self.is_error_status = false;
                            }
                            Err(e) => {
                                self.status_message = format!("Failed to save service file: {}", e);
                                self.is_error_status = true;
                            }
                        }
                    }
                    self.step = ActiveStep::StatusModal;
                } else {
                    // Deploy to system
                    if let Some(cfg) = &self.config {
                        match deploy_service_by_type(
                            self.target_init_system,
                            cfg,
                            &self.generated_service_content,
                        ) {
                            Ok(logs) => {
                                self.status_message = logs;
                                self.is_error_status = false;
                            }
                            Err(e) => {
                                self.status_message = format!("Deployment failed: {}", e);
                                self.is_error_status = true;
                            }
                        }
                    }
                    self.step = ActiveStep::StatusModal;
                }
            }
            ActiveStep::StatusModal => {
                // Close status modal and go back to preview
                self.step = ActiveStep::PreviewDeploy;
            }
        }
    }

    pub fn back_step(&mut self) {
        match self.step {
            ActiveStep::ServiceConfig => self.step = ActiveStep::BinaryPicker,
            ActiveStep::PreviewDeploy => self.step = ActiveStep::ServiceConfig,
            ActiveStep::StatusModal => self.step = ActiveStep::PreviewDeploy,
            ActiveStep::BinaryPicker => {}
        }
    }
}
