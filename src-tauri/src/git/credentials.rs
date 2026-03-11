use serde::{Deserialize, Serialize};

use super::error::GitError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// Find the first existing SSH private key in ~/.ssh/
fn find_ssh_key() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    let ssh_dir = std::path::PathBuf::from(home).join(".ssh");
    // Try common key names in preference order
    for name in &["id_ed25519", "id_ecdsa", "id_rsa"] {
        let key_path = ssh_dir.join(name);
        if key_path.exists() {
            return Some(key_path);
        }
    }
    None
}

/// Creates a credential callback closure for git2 remote operations.
///
/// For SSH remotes, tries in order:
///   1. SSH agent
///   2. SSH key files on disk (~/.ssh/id_ed25519, id_ecdsa, id_rsa)
///
/// If `creds` is provided, uses plaintext username/password for HTTPS.
/// Otherwise falls back to git credential helper.
pub fn make_credential_callback(
    creds: Option<Credentials>,
) -> impl FnMut(&str, Option<&str>, git2::CredentialType) -> Result<git2::Cred, git2::Error> {
    let mut ssh_attempt = 0u8; // 0 = agent, 1 = disk key, 2+ = exhausted
    let mut https_tried = false;
    move |url, username_from_url, allowed_types| {
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            let username = username_from_url.unwrap_or("git");
            match ssh_attempt {
                0 => {
                    ssh_attempt = 1;
                    // Try SSH agent first
                    match git2::Cred::ssh_key_from_agent(username) {
                        Ok(cred) => return Ok(cred),
                        Err(_) => {
                            // Agent failed, fall through to try disk key immediately
                            ssh_attempt = 2;
                            if let Some(key_path) = find_ssh_key() {
                                let pub_path = key_path.with_extension("pub");
                                let pub_key = if pub_path.exists() {
                                    Some(pub_path)
                                } else {
                                    None
                                };
                                return git2::Cred::ssh_key(
                                    username,
                                    pub_key.as_deref(),
                                    &key_path,
                                    None, // no passphrase
                                );
                            }
                            return Err(git2::Error::from_str(
                                "Authentication failed: SSH agent unavailable and no SSH key found on disk",
                            ));
                        }
                    }
                }
                1 => {
                    // git2 called back after agent succeeded but auth still failed — try disk key
                    ssh_attempt = 2;
                    if let Some(key_path) = find_ssh_key() {
                        let pub_path = key_path.with_extension("pub");
                        let pub_key = if pub_path.exists() {
                            Some(pub_path)
                        } else {
                            None
                        };
                        return git2::Cred::ssh_key(
                            username,
                            pub_key.as_deref(),
                            &key_path,
                            None,
                        );
                    }
                    return Err(git2::Error::from_str(
                        "Authentication failed: no valid SSH credentials available",
                    ));
                }
                _ => {
                    return Err(git2::Error::from_str(
                        "Authentication failed: all SSH methods exhausted",
                    ));
                }
            }
        }

        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            if https_tried {
                return Err(git2::Error::from_str(
                    "Authentication failed: no valid credentials available",
                ));
            }
            https_tried = true;

            if let Some(ref creds) = creds {
                return git2::Cred::userpass_plaintext(&creds.username, &creds.password);
            }
            return git2::Cred::credential_helper(
                &git2::Config::open_default()?,
                url,
                username_from_url,
            );
        }

        git2::Cred::default()
    }
}

/// Extract host from a git URL (supports both SSH and HTTPS formats).
fn extract_host(url: &str) -> Option<String> {
    // HTTPS: https://github.com/user/repo.git
    if let Some(rest) = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) {
        return rest.split('/').next().map(|s| s.to_string());
    }
    // SSH: git@github.com:user/repo.git
    if let Some(rest) = url.strip_prefix("git@") {
        return rest.split(':').next().map(|s| s.to_string());
    }
    // ssh://git@github.com/user/repo.git
    if let Some(rest) = url.strip_prefix("ssh://") {
        let after_at = rest.split('@').next_back()?;
        return after_at.split('/').next().map(|s| s.to_string());
    }
    None
}

/// Store credentials in the OS keychain.
pub fn store_credentials(remote_url: &str, creds: &Credentials) -> Result<(), GitError> {
    let host = extract_host(remote_url)
        .ok_or_else(|| GitError::General(format!("Cannot extract host from URL: {}", remote_url)))?;

    let service = format!("snazzgit:{}", host);
    let entry = keyring::Entry::new(&service, &creds.username)
        .map_err(|e| GitError::General(format!("Keyring error: {}", e)))?;
    entry
        .set_password(&creds.password)
        .map_err(|e| GitError::General(format!("Failed to store credentials: {}", e)))?;
    Ok(())
}

/// Delete credentials from the OS keychain.
pub fn delete_credentials(remote_url: &str, username: &str) -> Result<(), GitError> {
    let host = extract_host(remote_url)
        .ok_or_else(|| GitError::General(format!("Cannot extract host from URL: {}", remote_url)))?;

    let service = format!("snazzgit:{}", host);
    let entry = keyring::Entry::new(&service, username)
        .map_err(|e| GitError::General(format!("Keyring error: {}", e)))?;
    entry
        .delete_credential()
        .map_err(|e| GitError::General(format!("Failed to delete credentials: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- extract_host tests ---

    #[test]
    fn test_extract_host_https() {
        assert_eq!(
            extract_host("https://github.com/user/repo.git"),
            Some("github.com".to_string())
        );
    }

    #[test]
    fn test_extract_host_http() {
        assert_eq!(
            extract_host("http://gitlab.example.com/group/repo.git"),
            Some("gitlab.example.com".to_string())
        );
    }

    #[test]
    fn test_extract_host_ssh() {
        assert_eq!(
            extract_host("git@github.com:user/repo.git"),
            Some("github.com".to_string())
        );
    }

    #[test]
    fn test_extract_host_ssh_protocol() {
        assert_eq!(
            extract_host("ssh://git@gitlab.com/user/repo.git"),
            Some("gitlab.com".to_string())
        );
    }

    #[test]
    fn test_extract_host_with_port() {
        // https with port — host:port is returned (acceptable)
        let result = extract_host("https://git.example.com:8443/repo.git");
        assert!(result.is_some());
        assert!(result.unwrap().starts_with("git.example.com"));
    }

    #[test]
    fn test_extract_host_invalid_url() {
        assert_eq!(extract_host("not-a-url"), None);
        assert_eq!(extract_host(""), None);
        assert_eq!(extract_host("ftp://server/repo"), None);
    }

    // --- find_ssh_key tests ---

    #[test]
    fn test_find_ssh_key_returns_existing_key_or_none() {
        // This test verifies find_ssh_key doesn't panic and returns a valid path if found
        let result = find_ssh_key();
        if let Some(ref path) = result {
            assert!(path.exists());
            let name = path.file_name().unwrap().to_str().unwrap();
            assert!(
                name == "id_ed25519" || name == "id_ecdsa" || name == "id_rsa",
                "unexpected key name: {}",
                name
            );
        }
        // If None, that's fine — no SSH keys on this machine
    }

    #[test]
    fn test_find_ssh_key_preference_order() {
        // If multiple keys exist, ed25519 should be preferred over rsa
        let result = find_ssh_key();
        if let Some(ref path) = result {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap();
            let ssh_dir = std::path::PathBuf::from(home).join(".ssh");
            let name = path.file_name().unwrap().to_str().unwrap();
            // If ed25519 exists, it should be the one returned
            if ssh_dir.join("id_ed25519").exists() {
                assert_eq!(name, "id_ed25519");
            }
        }
    }

    // --- make_credential_callback tests ---

    #[test]
    fn test_callback_with_creds_returns_userpass_for_https() {
        let creds = Credentials {
            username: "testuser".into(),
            password: "testpass".into(),
        };
        let mut cb = make_credential_callback(Some(creds));
        let result = cb(
            "https://github.com/user/repo.git",
            None,
            git2::CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_callback_https_prevents_infinite_retry() {
        let creds = Credentials {
            username: "testuser".into(),
            password: "testpass".into(),
        };
        let mut cb = make_credential_callback(Some(creds));
        // First call succeeds
        let first = cb(
            "https://github.com/user/repo.git",
            None,
            git2::CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(first.is_ok());
        // Second call should fail (prevents infinite retry)
        let second = cb(
            "https://github.com/user/repo.git",
            None,
            git2::CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(second.is_err());
        let err_msg = second.err().unwrap().message().to_string();
        assert!(err_msg.contains("Authentication failed"));
    }

    #[test]
    fn test_callback_without_creds_https_tries_credential_helper() {
        // Without explicit creds, it falls through to credential helper
        // which will likely fail in test env — but should not panic
        let mut cb = make_credential_callback(None);
        let result = cb(
            "https://github.com/user/repo.git",
            Some("user"),
            git2::CredentialType::USER_PASS_PLAINTEXT,
        );
        // May succeed or fail depending on git config — just ensure no panic
        // and that a second call is blocked
        let _ = result;
        let second = cb(
            "https://github.com/user/repo.git",
            Some("user"),
            git2::CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(second.is_err());
    }

    #[test]
    fn test_callback_ssh_exhausts_after_attempts() {
        let mut cb = make_credential_callback(None);
        // First SSH call: tries agent (may fail), then disk key
        let _ = cb(
            "git@github.com:user/repo.git",
            Some("git"),
            git2::CredentialType::SSH_KEY,
        );
        // Second SSH call: may try disk key
        let _ = cb(
            "git@github.com:user/repo.git",
            Some("git"),
            git2::CredentialType::SSH_KEY,
        );
        // Third SSH call: must be exhausted
        let third = cb(
            "git@github.com:user/repo.git",
            Some("git"),
            git2::CredentialType::SSH_KEY,
        );
        assert!(third.is_err());
        assert!(third.err().unwrap().message().contains("exhausted"));
    }

    #[test]
    fn test_callback_ssh_default_username() {
        // When username_from_url is None, should default to "git"
        let mut cb = make_credential_callback(None);
        // Just ensure it doesn't panic with None username
        let _ = cb(
            "git@github.com:user/repo.git",
            None,
            git2::CredentialType::SSH_KEY,
        );
    }

    #[test]
    fn test_callback_default_cred_type() {
        // For unknown credential types, should return default
        let mut cb = make_credential_callback(None);
        let result = cb(
            "some://url",
            None,
            git2::CredentialType::DEFAULT,
        );
        // git2::Cred::default() may or may not succeed depending on platform
        let _ = result;
    }

    // --- Credentials serialization ---

    #[test]
    fn test_credentials_serialize_deserialize() {
        let creds = Credentials {
            username: "user".into(),
            password: "pass".into(),
        };
        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: Credentials = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.username, "user");
        assert_eq!(deserialized.password, "pass");
    }

    #[test]
    fn test_credentials_optional_deserialization() {
        // Ensures Option<Credentials> deserializes None from missing field
        // (important for backwards-compat on remote commands)
        let val: Option<Credentials> = serde_json::from_str("null").unwrap();
        assert!(val.is_none());
    }

    // --- store/delete credential error paths ---

    #[test]
    fn test_store_credentials_invalid_url() {
        let creds = Credentials {
            username: "user".into(),
            password: "pass".into(),
        };
        let result = store_credentials("not-a-url", &creds);
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Cannot extract host"));
    }

    #[test]
    fn test_delete_credentials_invalid_url() {
        let result = delete_credentials("not-a-url", "user");
        assert!(result.is_err());
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("Cannot extract host"));
    }
}
