use anyhow::Result;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegConnectRegistryW, RegGetValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, REG_EXPAND_SZ, REG_VALUE_TYPE,
    RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ,
};

const ENVIRONMENT_KEY: &str = "Environment";
const SYSTEM_ENVIRONMENT_KEY: &str =
    "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment";
const PATH_VALUE: &str = "Path";

/// Represents whether we're working with USER or MACHINE (SYSTEM) paths
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathScope {
    User,
    Machine,
}

impl PathScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            PathScope::User => "USER",
            PathScope::Machine => "MACHINE",
        }
    }
}

/// Represents a connection to a remote computer's registry
pub struct RemoteConnection {
    computer_name: String,
    hkey_local_machine: HKEY,
    hkey_current_user: HKEY,
}

impl RemoteConnection {
    /// Connect to a remote computer's registry
    pub fn connect(computer_name: &str) -> Result<Self> {
        unsafe {
            let computer_name_wide = to_wide_string(computer_name);

            // Connect to HKEY_LOCAL_MACHINE on remote computer
            let mut hkey_local_machine = HKEY::default();
            let result = RegConnectRegistryW(
                PCWSTR(computer_name_wide.as_ptr()),
                HKEY_LOCAL_MACHINE,
                &mut hkey_local_machine,
            );

            if result != ERROR_SUCCESS {
                return Err(anyhow::anyhow!(
                    "Failed to connect to remote computer '{}': Error code {}. \
                    Ensure the computer is reachable, Remote Registry service is running, \
                    and you have administrative privileges.",
                    computer_name,
                    result.0
                ));
            }

            // Connect to HKEY_CURRENT_USER on remote computer
            let mut hkey_current_user = HKEY::default();
            let result = RegConnectRegistryW(
                PCWSTR(computer_name_wide.as_ptr()),
                HKEY_CURRENT_USER,
                &mut hkey_current_user,
            );

            if result != ERROR_SUCCESS {
                // Clean up the HKEY_LOCAL_MACHINE handle before returning error
                let _ = RegCloseKey(hkey_local_machine);
                return Err(anyhow::anyhow!(
                    "Failed to connect to remote HKEY_CURRENT_USER on '{}': Error code {}",
                    computer_name,
                    result.0
                ));
            }

            Ok(RemoteConnection {
                computer_name: computer_name.to_string(),
                hkey_local_machine,
                hkey_current_user,
            })
        }
    }

    /// Get the computer name for this connection
    pub fn computer_name(&self) -> &str {
        &self.computer_name
    }

    /// Get the HKEY handle for the specified scope
    fn get_hkey(&self, scope: PathScope) -> HKEY {
        match scope {
            PathScope::User => self.hkey_current_user,
            PathScope::Machine => self.hkey_local_machine,
        }
    }
}

impl Drop for RemoteConnection {
    fn drop(&mut self) {
        unsafe {
            // Close both registry handles when the connection is dropped
            let _ = RegCloseKey(self.hkey_local_machine);
            let _ = RegCloseKey(self.hkey_current_user);
        }
    }
}

/// Read the PATH environment variable from the registry
pub fn read_path(scope: PathScope) -> Result<String> {
    read_path_with_connection(scope, None)
}

/// Read the PATH environment variable from a remote registry
pub fn read_path_remote(scope: PathScope, connection: &RemoteConnection) -> Result<String> {
    read_path_with_connection(scope, Some(connection))
}

/// Internal function to read PATH with optional remote connection
fn read_path_with_connection(
    scope: PathScope,
    connection: Option<&RemoteConnection>,
) -> Result<String> {
    unsafe {
        let (hkey_root, subkey) = if let Some(conn) = connection {
            // Use remote connection handle
            (
                conn.get_hkey(scope),
                match scope {
                    PathScope::User => ENVIRONMENT_KEY,
                    PathScope::Machine => SYSTEM_ENVIRONMENT_KEY,
                },
            )
        } else {
            // Use local registry
            match scope {
                PathScope::User => (HKEY_CURRENT_USER, ENVIRONMENT_KEY),
                PathScope::Machine => (HKEY_LOCAL_MACHINE, SYSTEM_ENVIRONMENT_KEY),
            }
        };

        let mut hkey = HKEY::default();
        let subkey_wide = to_wide_string(subkey);

        // Open the registry key
        let result = RegOpenKeyExW(
            hkey_root,
            PCWSTR(subkey_wide.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!(
                "Failed to open registry key for {} paths",
                scope.as_str()
            ));
        }

        // Query the size needed
        let value_name_wide = to_wide_string(PATH_VALUE);
        let mut buffer_size = 0u32;

        let result = RegGetValueW(
            hkey,
            PCWSTR::null(),
            PCWSTR(value_name_wide.as_ptr()),
            RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
            None,
            None,
            Some(&mut buffer_size),
        );

        if result != ERROR_SUCCESS {
            let _ = RegCloseKey(hkey).ok();
            return Err(anyhow::anyhow!(
                "Failed to query {} PATH size",
                scope.as_str()
            ));
        }

        // Allocate buffer and read the value
        let mut buffer = vec![0u16; (buffer_size / 2) as usize];
        let mut value_type = REG_VALUE_TYPE::default();

        let result = RegGetValueW(
            hkey,
            PCWSTR::null(),
            PCWSTR(value_name_wide.as_ptr()),
            RRF_RT_REG_SZ | RRF_RT_REG_EXPAND_SZ,
            Some(&mut value_type),
            Some(buffer.as_mut_ptr() as *mut _),
            Some(&mut buffer_size),
        );

        let _ = RegCloseKey(hkey).ok();

        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!("Failed to read {} PATH", scope.as_str()));
        }

        // Convert to Rust string
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
        let path_string = OsString::from_wide(&buffer[..len])
            .into_string()
            .map_err(|_| anyhow::anyhow!("Invalid UTF-8 in PATH"))?;

        Ok(path_string)
    }
}

/// Write the PATH environment variable to the registry
pub fn write_path(scope: PathScope, value: &str) -> Result<()> {
    write_path_with_connection(scope, value, None)
}

/// Write the PATH environment variable to a remote registry
pub fn write_path_remote(
    scope: PathScope,
    value: &str,
    connection: &RemoteConnection,
) -> Result<()> {
    write_path_with_connection(scope, value, Some(connection))
}

/// Internal function to write PATH with optional remote connection
fn write_path_with_connection(
    scope: PathScope,
    value: &str,
    connection: Option<&RemoteConnection>,
) -> Result<()> {
    unsafe {
        let (hkey_root, subkey) = if let Some(conn) = connection {
            // Use remote connection handle
            (
                conn.get_hkey(scope),
                match scope {
                    PathScope::User => ENVIRONMENT_KEY,
                    PathScope::Machine => SYSTEM_ENVIRONMENT_KEY,
                },
            )
        } else {
            // Use local registry
            match scope {
                PathScope::User => (HKEY_CURRENT_USER, ENVIRONMENT_KEY),
                PathScope::Machine => (HKEY_LOCAL_MACHINE, SYSTEM_ENVIRONMENT_KEY),
            }
        };

        let mut hkey = HKEY::default();
        let subkey_wide = to_wide_string(subkey);

        // Open the registry key with write access
        let result = RegOpenKeyExW(
            hkey_root,
            PCWSTR(subkey_wide.as_ptr()),
            0,
            KEY_WRITE,
            &mut hkey,
        );

        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!(
                "Failed to open registry key for writing {} paths. Do you have admin rights?",
                scope.as_str()
            ));
        }

        // Convert value to wide string
        let value_wide = to_wide_string(value);
        let value_name_wide = to_wide_string(PATH_VALUE);

        // Convert wide string to byte slice for the new API
        let value_bytes =
            std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2);

        // Write the value
        let result = RegSetValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            0,
            REG_EXPAND_SZ,
            Some(value_bytes),
        );

        let _ = RegCloseKey(hkey).ok();

        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!("Failed to write {} PATH", scope.as_str()));
        }

        // Broadcast WM_SETTINGCHANGE to notify other applications
        broadcast_environment_change()?;

        Ok(())
    }
}

/// Parse a PATH string into individual entries
pub fn parse_path(path_string: &str) -> Vec<String> {
    path_string
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Join path entries into a PATH string
pub fn join_paths(paths: &[String]) -> String {
    paths.join(";")
}

/// Convert a Rust string to a null-terminated wide string
fn to_wide_string(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Broadcast environment change notification
fn broadcast_environment_change() -> Result<()> {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    unsafe {
        let env_wide = to_wide_string("Environment");
        let mut result = 0;

        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(env_wide.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            Some(&mut result),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let path = r"C:\Windows;C:\Windows\System32;C:\Program Files";
        let entries = parse_path(path);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0], r"C:\Windows");
    }

    #[test]
    fn test_join_paths() {
        let paths = vec![
            r"C:\Windows".to_string(),
            r"C:\Windows\System32".to_string(),
        ];
        let joined = join_paths(&paths);
        assert_eq!(joined, r"C:\Windows;C:\Windows\System32");
    }
}
