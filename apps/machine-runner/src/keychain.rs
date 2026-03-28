//! OS keychain integration for securely storing the API token.
//!
//! Uses the `keyring` crate which delegates to platform-native backends:
//! - macOS: Keychain Services
//! - Windows: Credential Manager
//! - Linux: Secret Service (GNOME Keyring / KWallet)

use anyhow::Result;

const SERVICE_NAME: &str = "cadre-machine-runner";

/// Store an API token in the OS keychain.
///
/// The token is stored under `SERVICE_NAME` with the `machine_name` as the
/// account identifier, allowing multiple machines per user.
///
/// # Errors
///
/// Returns an error if the keychain is unavailable or the store operation fails.
pub fn store_token(machine_name: &str, token: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, machine_name)?;
    entry.set_password(token)?;
    tracing::debug!(machine = %machine_name, "Stored API token in OS keychain");
    Ok(())
}

/// Retrieve an API token from the OS keychain.
///
/// Returns `Ok(None)` if no token is stored for the given machine name or if
/// the keychain is not available (with a logged warning).
///
/// # Errors
///
/// Returns an error only on unexpected keychain failures.
pub fn get_token(machine_name: &str) -> Result<Option<String>> {
    let entry = keyring::Entry::new(SERVICE_NAME, machine_name)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(keyring::Error::NoStorageAccess(_)) => {
            tracing::warn!("OS keychain not accessible; token not available");
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Delete an API token from the OS keychain.
///
/// Returns `Ok(())` if the token was deleted or if no entry existed.
///
/// # Errors
///
/// Returns an error on unexpected keychain failures.
pub fn delete_token(machine_name: &str) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, machine_name)?;
    match entry.delete_credential() {
        Ok(()) => {
            tracing::debug!(machine = %machine_name, "Deleted API token from OS keychain");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => Ok(()), // already gone
        Err(e) => Err(e.into()),
    }
}

/// Check if the OS keychain is available on this platform.
///
/// Performs a lightweight probe by creating a test entry. Does not store data.
pub fn is_keychain_available() -> bool {
    keyring::Entry::new(SERVICE_NAME, "__keychain_probe__").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keychain_available_does_not_panic() {
        // Just verifying the probe doesn't crash, regardless of result.
        let _available = is_keychain_available();
    }

    /// Full store / get / delete cycle against the real OS keychain.
    ///
    /// Ignored by default because it requires a live keychain and may prompt
    /// for permission on some platforms.
    #[test]
    #[ignore]
    fn test_keychain_store_get_delete_cycle() {
        let machine = "__cadre_test_machine__";
        let token = "tok_test_keychain_12345";

        // Store
        store_token(machine, token).expect("store_token failed");

        // Get
        let retrieved = get_token(machine)
            .expect("get_token failed")
            .expect("token should exist");
        assert_eq!(retrieved, token);

        // Delete
        delete_token(machine).expect("delete_token failed");

        // Verify deletion
        let after_delete = get_token(machine).expect("get_token after delete failed");
        assert!(after_delete.is_none(), "token should be gone after delete");
    }
}
