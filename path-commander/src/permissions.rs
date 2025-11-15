use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Check if the current process is running with administrator privileges
pub fn is_admin() -> bool {
    unsafe {
        let mut token = Default::default();

        // Get the access token for the current process
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut return_length = 0u32;

        // Query the token elevation information
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );

        if result.is_err() {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

/// Get a message about admin privileges
pub fn get_privilege_message() -> String {
    if is_admin() {
        "Running with Administrator privileges - Can modify both USER and MACHINE paths".to_string()
    } else {
        "Running without Administrator privileges - MACHINE paths are read-only".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin() {
        // This will return the actual admin status
        let admin = is_admin();
        println!("Is admin: {}", admin);
    }
}
