use crate::device::SystemInfo;
use crate::generator::ServiceConfig;
use anyhow::{Result, anyhow};
use std::fs;
use std::process::Command;

pub fn render_openrc_file(config: &ServiceConfig, sys: &SystemInfo) -> String {
    let mut out = String::new();
    out.push_str("#!/sbin/openrc-run\n");
    out.push_str("# ========================================================\n");
    out.push_str(&format!(
        "# Auto-generated OpenRC Init Script for {}\n",
        config.app_name
    ));
    out.push_str(&format!(
        "# Target System: {} ({}/{})\n",
        sys.hostname, sys.os_name, sys.cpu_arch
    ));
    out.push_str(&format!("# Target Path: {}\n", config.exec_start));
    out.push_str("# ========================================================\n\n");

    out.push_str(&format!("name=\"{}\"\n", config.app_name));
    out.push_str(&format!("description=\"{}\"\n\n", config.description));

    out.push_str(&format!("command=\"{}\"\n", config.exec_start));
    if !config.extra_flags.trim().is_empty() {
        out.push_str(&format!("command_args=\"{}\"\n", config.extra_flags.trim()));
    }
    out.push_str("command_background=\"true\"\n");
    out.push_str("pidfile=\"/run/${RC_SVCNAME}.pid\"\n");
    out.push_str(&format!("directory=\"{}\"\n", config.working_dir));

    let group_str = if config.group.is_empty() {
        config.user.clone()
    } else {
        config.group.clone()
    };
    out.push_str(&format!("command_user=\"{}:{}\"\n", config.user, group_str));

    if !config.environment.trim().is_empty() {
        out.push_str("\n# Environment Variables\n");
        for line in config.environment.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                out.push_str(&format!("export {}\n", trimmed));
            }
        }
    }

    out.push_str("\ndepend() {\n");
    out.push_str("    need net\n");
    out.push_str("    use dns logger\n");
    out.push_str("}\n");

    out
}

pub fn deploy_openrc_systemwide(config: &ServiceConfig, content: &str) -> Result<String> {
    let mut logs = String::new();

    // 1. Create target directory /opt/<app_name>
    logs.push_str(&format!("--> Creating directory: {}\n", config.working_dir));
    let status_mkdir = Command::new("sudo")
        .args(["mkdir", "-p", &config.working_dir])
        .status()?;

    if !status_mkdir.success() {
        return Err(anyhow!(
            "Failed to execute 'sudo mkdir -p {}'",
            config.working_dir
        ));
    }

    // 2. Copy binary to /opt/<app_name>/<binary_name>
    if config.source_binary_path.exists() {
        logs.push_str(&format!(
            "--> Copying binary {} -> {}\n",
            config.source_binary_path.display(),
            config.exec_start
        ));
        let status_cp = Command::new("sudo")
            .args([
                "cp",
                config.source_binary_path.to_str().unwrap(),
                &config.exec_start,
            ])
            .status()?;

        if !status_cp.success() {
            return Err(anyhow!("Failed to copy binary to {}", config.exec_start));
        }

        let status_chmod = Command::new("sudo")
            .args(["chmod", "+x", &config.exec_start])
            .status()?;
        if !status_chmod.success() {
            return Err(anyhow!(
                "Failed to set execute permissions on {}",
                config.exec_start
            ));
        }
    } else {
        logs.push_str(&format!(
            "--> Warning: Source binary file {} not found locally. Skipping binary copy.\n",
            config.source_binary_path.display()
        ));
    }

    // 3. Save init script to temporary path, then copy to /etc/init.d/
    let tmp_script_path = format!("/tmp/{}", config.app_name);
    fs::write(&tmp_script_path, content)?;

    let target_script_path = format!("/etc/init.d/{}", config.app_name);
    logs.push_str(&format!(
        "--> Deploying OpenRC init script to {}\n",
        target_script_path
    ));

    let status_script_cp = Command::new("sudo")
        .args(["cp", &tmp_script_path, &target_script_path])
        .status()?;

    if !status_script_cp.success() {
        return Err(anyhow!(
            "Failed to copy init script to {}",
            target_script_path
        ));
    }

    let status_chmod_script = Command::new("sudo")
        .args(["chmod", "+x", &target_script_path])
        .status()?;

    if !status_chmod_script.success() {
        return Err(anyhow!(
            "Failed to set executable permissions on {}",
            target_script_path
        ));
    }

    // 4. Enable service using rc-update
    logs.push_str(&format!(
        "--> Enabling service in default runlevel (sudo rc-update add {} default)...\n",
        config.app_name
    ));
    let status_rc_update = Command::new("sudo")
        .args(["rc-update", "add", &config.app_name, "default"])
        .status()?;

    if status_rc_update.success() {
        logs.push_str("--> Service added to default runlevel successfully!\n");
    } else {
        logs.push_str("--> Note: Could not run 'rc-update add' automatically.\n");
    }

    logs.push_str(&format!(
        "\nSUCCESS! OpenRC init script deployed.\nTo start service, run:\n  sudo rc-service {} start\n  sudo rc-service {} status\n",
        config.app_name, config.app_name
    ));

    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_render_openrc_file() {
        let sys = SystemInfo::detect();
        let config = ServiceConfig::new(
            "my_app".to_string(),
            "my_app_bin".to_string(),
            PathBuf::from("/tmp/my_app_bin"),
            "myuser".to_string(),
        );

        let rendered = render_openrc_file(&config, &sys);
        assert!(rendered.starts_with("#!/sbin/openrc-run"));
        assert!(rendered.contains("name=\"my_app\""));
        assert!(rendered.contains("command=\"/opt/my_app/my_app_bin\""));
        assert!(rendered.contains("command_user=\"myuser:myuser\""));
        assert!(rendered.contains("depend() {"));
    }
}

