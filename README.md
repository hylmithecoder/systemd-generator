# Systemd Service File Generator TUI (`systemdgenerator-v1.0`)

A terminal user interface (TUI) application built in **Rust** using **`ratatui`** and **`crossterm`** for generating, configuring, and deploying Linux `systemd` `.service` unit files.

It automatically inspects target system/device hardware specs and enforces directory structure consistency by mapping source binaries (e.g. `~/Project/thebinariesname/target/release/thebinariesfile`) to target path `/opt/thebinariesname/thebinariesfile`.

---

## Features

- **Binary Location Derivation**: Strips quotes and automatically resolves app name and executable filename, mapping binaries consistently to `/opt/<app_name>/<binary_name>`.
- **Target Device Environment Inspection**: Auto-detects Hostname, OS Distribution & Version, CPU Architecture, Kernel, and active User.
- **Interactive TUI Form**: Configure service properties including `ExecStart`, `WorkingDirectory`, `User`, `Group`, `Restart` policy, `Environment` variables, and custom CLI flags using `Ratatui`.
- **Live Unit Preview**: Real-time syntax-highlighted systemd `.service` file content preview.
- **Automated Privilege Deployment**:
  - Save unit file locally (`./<app_name>.service`).
  - Copy binary to `/opt/<app_name>/` and deploy unit to `/etc/systemd/system/<app_name>.service` with `sudo systemctl daemon-reload`.

---

## Installation

### Method 1: Using `install.sh` (Recommended)

Run the automated installer script:
```bash
./install.sh
```

This compiles the release binary and installs `systemdgenerator-v1.0` (with a `systemdgenerator` symlink) into `/usr/local/bin/`.

### Method 2: Manual Cargo Build

Build the binary directly with Cargo:
```bash
cargo build --release
```

The compiled binary will be placed at `target/release/systemdfilegenerator`.

---

## Usage

Start the interactive TUI application:

```bash
systemdgenerator-v1.0
# Or using Cargo:
cargo run
```

### TUI Workflow & Controls

1. **Step 1: Pick Binary**
   - Type or paste the path to your executable (e.g., `~/Project/api_siabsen/target/release/api_siabsen` or `'/path/to/bin'`).
   - The TUI dynamically derives:
     - **App Name**: `api_siabsen`
     - **Executable Path**: `/opt/api_siabsen/api_siabsen`
     - **Working Directory**: `/opt/api_siabsen`
   - Press **`Enter`** to proceed to configuration.

2. **Step 2: Configure Service**
   - Navigate fields using **`Tab`** or **`Up/Down Arrows`**.
   - Edit Unit Name, Description, User, Group, Restart Policy, Extra Flags, and Environment variables.
   - Press **`Enter`** to proceed to preview.

3. **Step 3: Preview & Deploy**
   - Inspect the auto-generated `.service` file content.
   - Switch actions using **`Tab`** or **`Up/Down Arrows`**:
     - **Option 1**: Save Unit File Locally (`./<app_name>.service`)
     - **Option 2**: Deploy Systemwide to `/etc/systemd/system/` (moves binary to `/opt/<app_name>/` and runs `systemctl daemon-reload`).
   - Press **`Enter`** to execute action.

---

## Keyboard Shortcuts

| Shortcut | Description |
| --- | --- |
| `Enter` | Confirm / Proceed to next step / Execute action |
| `Tab` / `Down Arrow` | Move focus to next form field or option |
| `Shift+Tab` / `Up Arrow` | Move focus to previous form field or option |
| `Backspace` | Delete character in focused field |
| `Esc` | Go back to previous step / Exit app |
| `Ctrl + C` | Force quit TUI |

---

## License

MIT License. Developed for Linux System Administration and Service Management.
