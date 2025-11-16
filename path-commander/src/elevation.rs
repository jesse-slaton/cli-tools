use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SW_NORMAL};

use crate::app::{ConnectionMode, FilterMode, Panel};

/// Serializable state for elevation persistence
#[derive(Debug, Serialize, Deserialize)]
pub struct ElevationState {
    // Connection mode
    pub connection_mode: ConnectionMode,
    pub remote_computer_name: Option<String>,

    // Paths and state
    pub machine_paths: Vec<String>,
    pub user_paths: Vec<String>,
    pub remote_machine_paths: Vec<String>,

    // Selections and marks
    pub active_panel: Panel,
    pub machine_selected: usize,
    pub user_selected: usize,
    pub remote_machine_selected: usize,
    pub machine_marked: HashSet<usize>,
    pub user_marked: HashSet<usize>,
    pub remote_machine_marked: HashSet<usize>,

    // Other state
    pub filter_mode: FilterMode,
    pub input_buffer: String,
    pub pending_directory: String,

    // Theme to restore
    pub theme_arg: Option<String>,
}

impl ElevationState {
    /// Save state to a temporary JSON file
    pub fn save(&self) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let random = rand::random::<u32>();
        let filename = format!("pc_elevation_{}_{}.json", timestamp, random);
        let path = temp_dir.join(filename);

        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize elevation state")?;

        std::fs::write(&path, json).context("Failed to write elevation state file")?;

        Ok(path)
    }

    /// Load state from a temporary JSON file
    pub fn load(path: &PathBuf) -> Result<Self> {
        // Validate file age (reject if older than 5 minutes for security)
        let metadata =
            std::fs::metadata(path).context("Failed to read elevation state file metadata")?;
        let created = metadata
            .created()
            .or_else(|_| metadata.modified())
            .context("Failed to get file timestamp")?;
        let age = std::time::SystemTime::now()
            .duration_since(created)
            .unwrap_or(std::time::Duration::from_secs(0));

        if age.as_secs() > 300 {
            anyhow::bail!("Elevation state file is too old (>5 minutes). Ignoring for security.");
        }

        // Read and parse file
        let json = std::fs::read_to_string(path).context("Failed to read elevation state file")?;

        let state: Self =
            serde_json::from_str(&json).context("Failed to deserialize elevation state")?;

        // Clean up temp file
        std::fs::remove_file(path).ok();

        Ok(state)
    }
}

/// Request UAC elevation by restarting the application with administrator privileges
pub fn request_elevation(state: &ElevationState, current_exe: &str) -> Result<()> {
    // Save state to temp file
    let state_file = state.save()?;

    // Build command line arguments
    let mut args = vec![
        "--restore-state".to_string(),
        state_file.to_string_lossy().to_string(),
    ];

    // Include theme argument if present
    if let Some(ref theme) = state.theme_arg {
        args.push("--theme".to_string());
        args.push(theme.clone());
    }

    let params = args.join(" ");

    // Convert strings to wide strings for Windows API
    let exe_wide: Vec<u16> = current_exe
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let params_wide: Vec<u16> = params.encode_utf16().chain(std::iter::once(0)).collect();
    let verb_wide: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let hwnd = GetForegroundWindow();

        let result = ShellExecuteW(
            hwnd,
            PCWSTR(verb_wide.as_ptr()),
            PCWSTR(exe_wide.as_ptr()),
            PCWSTR(params_wide.as_ptr()),
            PCWSTR::null(),
            SW_NORMAL,
        );

        // ShellExecuteW returns a value > 32 if successful
        if result.0 as isize <= 32 {
            anyhow::bail!(
                "UAC elevation was cancelled or failed (error code: {})",
                result.0 as isize
            );
        }
    }

    Ok(())
}

/// Check if there are MACHINE path changes that require elevation
pub fn needs_elevation_for_changes(
    is_admin: bool,
    machine_paths: &[String],
    machine_original: &[String],
    remote_machine_paths: &[String],
    remote_machine_original: &[String],
    connection_mode: ConnectionMode,
) -> bool {
    if is_admin {
        return false; // Already admin, no elevation needed
    }

    match connection_mode {
        ConnectionMode::Local => {
            // Check if MACHINE paths have changed
            machine_paths != machine_original
        }
        ConnectionMode::Remote => {
            // Check if either local MACHINE or remote MACHINE have changed
            machine_paths != machine_original || remote_machine_paths != remote_machine_original
        }
    }
}
