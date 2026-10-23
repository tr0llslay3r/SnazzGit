use std::io::Write;
use std::process::{Command, Stdio};

use git2::{Commit, Repository, Signature, Tree};

use super::error::GitError;
use super::types::CommitSignatureInfo;

/// Sign `content` (an unsigned commit object buffer) with gpg, returning the
/// ASCII-armored detached signature. `gnupg_home` overrides `GNUPGHOME` for
/// the spawned process — tests use this to point at an isolated keyring;
/// production code passes `None` to use the user's real keyring.
fn gpg_sign(
    gnupg_home: Option<&str>,
    signing_key: Option<&str>,
    content: &str,
) -> Result<String, GitError> {
    let mut cmd = Command::new("gpg");
    cmd.args(["--batch", "--yes", "-bsa", "--output", "-"]);
    if let Some(key) = signing_key {
        cmd.args(["--local-user", key]);
    }
    if let Some(home) = gnupg_home {
        cmd.env("GNUPGHOME", home);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| GitError::General(format!("Failed to launch gpg: {}", e)))?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(content.as_bytes())?;
    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(GitError::General(format!(
            "gpg signing failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Verify a detached signature against the signed content. gpg's `--verify`
/// needs the signature as a real file (a detached signature can't be read
/// from the same stdin stream as the content being verified), so this writes
/// it to a throwaway temp file and removes it again afterward.
fn gpg_verify(
    gnupg_home: Option<&str>,
    content: &str,
    signature: &str,
) -> Result<CommitSignatureInfo, GitError> {
    let sig_path = std::env::temp_dir().join(format!(
        "snazzgit-sig-{}-{}.asc",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&sig_path, signature.as_bytes())?;

    let result = run_gpg_verify(gnupg_home, content, &sig_path);
    let _ = std::fs::remove_file(&sig_path);
    result
}

fn run_gpg_verify(
    gnupg_home: Option<&str>,
    content: &str,
    sig_path: &std::path::Path,
) -> Result<CommitSignatureInfo, GitError> {
    let mut cmd = Command::new("gpg");
    cmd.arg("--batch")
        .arg("--status-fd")
        .arg("1")
        .arg("--verify")
        .arg(sig_path)
        .arg("-");
    if let Some(home) = gnupg_home {
        cmd.env("GNUPGHOME", home);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| GitError::General(format!("Failed to launch gpg: {}", e)))?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(content.as_bytes())?;
    let output = child.wait_with_output()?;
    let status_text = String::from_utf8_lossy(&output.stdout);

    let valid = status_text.contains("[GNUPG:] GOODSIG");
    let signer = status_text.lines().find_map(|line| {
        line.strip_prefix("[GNUPG:] GOODSIG ")
            .or_else(|| line.strip_prefix("[GNUPG:] BADSIG "))
            .or_else(|| line.strip_prefix("[GNUPG:] EXPSIG "))
            .map(|rest| {
                // Format: "<long keyid> <user id...>"
                rest.split_once(' ').map_or(rest, |(_, user_id)| user_id).to_string()
            })
    });

    Ok(CommitSignatureInfo {
        signed: true,
        valid,
        signer,
    })
}

/// Create a commit, signing it with gpg when `gnupg_home`/`signing_key` are
/// provided or when the repo config enables `commit.gpgsign`. Unlike
/// `Repository::commit`, `commit_signed` doesn't update any ref itself, so
/// this also advances HEAD's current branch (or HEAD directly if detached).
pub fn create_signed_commit(
    repo: &Repository,
    sig: &Signature,
    message: &str,
    tree: &Tree,
    parents: &[&Commit],
    gnupg_home: Option<&str>,
) -> Result<String, GitError> {
    let buffer = repo.commit_create_buffer(sig, sig, message, tree, parents)?;
    let content = std::str::from_utf8(&buffer)
        .map_err(|e| GitError::General(format!("Commit buffer is not valid UTF-8: {}", e)))?;

    let signing_key = repo
        .config()
        .ok()
        .and_then(|c| c.get_string("user.signingkey").ok());

    let signature = gpg_sign(gnupg_home, signing_key.as_deref(), content)?;
    let oid = repo.commit_signed(content, &signature, None)?;

    let head_ref = repo.find_reference("HEAD")?;
    if let Some(target) = head_ref.symbolic_target() {
        let target = target.to_string();
        repo.reference(&target, oid, true, "commit (signed)")?;
    } else {
        repo.set_head_detached(oid)?;
    }

    Ok(oid.to_string())
}

pub fn get_commit_signature(path: &str, commit_id: &str) -> Result<CommitSignatureInfo, GitError> {
    get_commit_signature_with_home(path, commit_id, None)
}

fn get_commit_signature_with_home(
    path: &str,
    commit_id: &str,
    gnupg_home: Option<&str>,
) -> Result<CommitSignatureInfo, GitError> {
    let repo = Repository::open(path)?;
    let oid = git2::Oid::from_str(commit_id)
        .map_err(|e| GitError::General(format!("Invalid commit ID: {}", e)))?;
    // Ensure the commit actually exists before treating a failure from
    // extract_signature as "no signature present" rather than a real error.
    repo.find_commit(oid)?;

    match repo.extract_signature(&oid, None) {
        Ok((signature, content)) => {
            let signature = signature
                .as_str()
                .ok_or_else(|| GitError::General("Signature is not valid UTF-8".to_string()))?;
            let content = content
                .as_str()
                .ok_or_else(|| GitError::General("Commit content is not valid UTF-8".to_string()))?;
            gpg_verify(gnupg_home, content, signature)
        }
        Err(_) => Ok(CommitSignatureInfo {
            signed: false,
            valid: false,
            signer: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_repo_with_commit() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let repo = git2::Repository::init(&path).unwrap();
        let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
        (dir, path)
    }

    /// Generates an isolated, passphrase-less ed25519 test key in a fresh
    /// GNUPGHOME so tests never touch the real developer's keyring.
    fn make_test_gpg_home() -> tempfile::TempDir {
        let home = tempfile::TempDir::new().unwrap();
        let batch_file = home.path().join("keygen.batch");
        std::fs::write(
            &batch_file,
            "%no-protection\n\
             Key-Type: eddsa\n\
             Key-Curve: ed25519\n\
             Name-Real: Test User\n\
             Name-Email: test@test.com\n\
             Expire-Date: 0\n\
             %commit\n",
        )
        .unwrap();

        let output = Command::new("gpg")
            .env("GNUPGHOME", home.path())
            .args(["--batch", "--gen-key", batch_file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "key generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        home
    }

    #[test]
    fn test_gpg_sign_and_verify_round_trip() {
        let home = make_test_gpg_home();
        let home_path = home.path().to_str().unwrap();

        let signature = gpg_sign(Some(home_path), Some("test@test.com"), "hello world").unwrap();
        assert!(signature.contains("BEGIN PGP SIGNATURE"));

        let info = gpg_verify(Some(home_path), "hello world", &signature).unwrap();
        assert!(info.signed);
        assert!(info.valid);
        assert!(info.signer.unwrap().contains("test@test.com"));
    }

    #[test]
    fn test_gpg_verify_tampered_content_is_invalid() {
        let home = make_test_gpg_home();
        let home_path = home.path().to_str().unwrap();

        let signature = gpg_sign(Some(home_path), Some("test@test.com"), "original content").unwrap();
        let info = gpg_verify(Some(home_path), "tampered content", &signature).unwrap();
        assert!(!info.valid);
    }

    #[test]
    fn test_gpg_sign_nonexistent_key_errors() {
        let home = make_test_gpg_home();
        let home_path = home.path().to_str().unwrap();

        let result = gpg_sign(Some(home_path), Some("no-such-key@test.com"), "hello");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_signed_commit_and_verify_round_trip() {
        let (_dir, path) = init_repo_with_commit();
        let home = make_test_gpg_home();
        let home_path = home.path().to_str().unwrap();

        {
            let repo = git2::Repository::open(&path).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.signingkey", "test@test.com").unwrap();
        }

        let oid_str = {
            let repo = git2::Repository::open(&path).unwrap();
            let sig = git2::Signature::now("Test User", "test@test.com").unwrap();
            let head = repo.head().unwrap().peel_to_commit().unwrap();
            let tree = head.tree().unwrap();
            create_signed_commit(
                &repo,
                &sig,
                "Signed commit",
                &tree,
                &[&head],
                Some(home_path),
            )
            .unwrap()
        };

        // HEAD advanced to the new signed commit
        let repo = git2::Repository::open(&path).unwrap();
        assert_eq!(repo.head().unwrap().target().unwrap().to_string(), oid_str);

        let info = get_commit_signature_with_home(&path, &oid_str, Some(home_path)).unwrap();
        assert!(info.signed);
        assert!(info.valid);
    }

    #[test]
    fn test_get_commit_signature_unsigned_commit() {
        let (_dir, path) = init_repo_with_commit();
        let oid = {
            let repo = git2::Repository::open(&path).unwrap();
            let target = repo.head().unwrap().target().unwrap();
            target
        };
        let info = get_commit_signature(&path, &oid.to_string()).unwrap();
        assert!(!info.signed);
        assert!(!info.valid);
        assert!(info.signer.is_none());
    }

    #[test]
    fn test_get_commit_signature_invalid_commit_id_errors() {
        let (_dir, path) = init_repo_with_commit();
        let result = get_commit_signature(&path, "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef");
        assert!(result.is_err());
    }
}
