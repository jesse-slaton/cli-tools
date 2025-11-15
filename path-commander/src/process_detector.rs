use anyhow::Result;
use std::collections::HashSet;
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

/// List of known processes that don't respond to WM_SETTINGCHANGE
/// These processes load environment variables at startup and don't refresh them
const NON_RESPONSIVE_PROCESSES: &[&str] = &[
    "cmd.exe",             // Command Prompt
    "powershell.exe",      // Windows PowerShell
    "pwsh.exe",            // PowerShell Core
    "WindowsTerminal.exe", // Windows Terminal
    "conhost.exe",         // Console Host
    "bash.exe",            // Git Bash / MSYS2
    "mintty.exe",          // MinTTY (MSYS2/Cygwin)
    "Code.exe",            // VS Code
    "devenv.exe",          // Visual Studio
    "rider64.exe",         // JetBrains Rider
    "idea64.exe",          // JetBrains IntelliJ
    "pycharm64.exe",       // JetBrains PyCharm
    "webstorm64.exe",      // JetBrains WebStorm
    "sublime_text.exe",    // Sublime Text
    "notepad++.exe",       // Notepad++
    "atom.exe",            // Atom Editor
];

/// Detect which known non-responsive processes are currently running
pub fn detect_running_processes() -> Result<Vec<String>> {
    unsafe {
        // Create snapshot of all processes
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
        if snapshot.is_invalid() {
            return Err(anyhow::anyhow!("Failed to create process snapshot"));
        }

        // Ensure handle is closed when function exits
        let _guard = HandleGuard(snapshot);

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        // Get first process
        if Process32FirstW(snapshot, &mut entry).is_err() {
            return Err(anyhow::anyhow!("Failed to get first process"));
        }

        let mut running_processes = HashSet::new();

        // Iterate through all processes
        loop {
            // Convert process name from wide string
            let exe_len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let exe_name = String::from_utf16_lossy(&entry.szExeFile[..exe_len]);

            // Check if this is a non-responsive process
            let exe_lower = exe_name.to_lowercase();
            for &known_process in NON_RESPONSIVE_PROCESSES {
                if exe_lower == known_process.to_lowercase() {
                    running_processes.insert(exe_name.clone());
                    break;
                }
            }

            // Move to next process
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            match Process32NextW(snapshot, &mut entry) {
                Ok(_) => continue,
                Err(e) => {
                    if e.code() == ERROR_NO_MORE_FILES.to_hresult() {
                        break; // No more processes
                    }
                    return Err(anyhow::anyhow!("Failed to enumerate processes: {}", e));
                }
            }
        }

        // Convert to sorted vector for consistent display
        let mut result: Vec<String> = running_processes.into_iter().collect();
        result.sort_by_key(|s| s.to_lowercase());

        Ok(result)
    }
}

/// RAII guard to ensure handle is closed
struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_processes() {
        // This test just verifies the function runs without crashing
        // Actual running processes will vary by environment
        let result = detect_running_processes();
        assert!(result.is_ok());

        if let Ok(processes) = result {
            // Should return a list (may be empty if none of the known processes are running)
            println!("Detected processes: {:?}", processes);
        }
    }
}
