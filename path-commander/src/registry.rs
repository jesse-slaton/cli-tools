use anyhow::Result;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegGetValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, REG_EXPAND_SZ, REG_VALUE_TYPE, RRF_RT_REG_EXPAND_SZ,
    RRF_RT_REG_SZ,
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

/// Read the PATH environment variable from the registry
pub fn read_path(scope: PathScope) -> Result<String> {
    unsafe {
        let (hkey_root, subkey) = match scope {
            PathScope::User => (HKEY_CURRENT_USER, ENVIRONMENT_KEY),
            PathScope::Machine => (HKEY_LOCAL_MACHINE, SYSTEM_ENVIRONMENT_KEY),
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
    unsafe {
        let (hkey_root, subkey) = match scope {
            PathScope::User => (HKEY_CURRENT_USER, ENVIRONMENT_KEY),
            PathScope::Machine => (HKEY_LOCAL_MACHINE, SYSTEM_ENVIRONMENT_KEY),
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
