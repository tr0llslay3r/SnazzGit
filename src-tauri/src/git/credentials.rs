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
        let after_at = rest.split('@').last()?;
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

    #[test]
    fn test_extract_host_https() {
        assert_eq!(
            extract_host("https://github.com/user/repo.git"),
            Some("github.com".to_string())
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
}
