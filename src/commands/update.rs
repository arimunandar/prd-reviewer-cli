use std::process::Command;

const REPO_URL: &str = "https://github.com/arimunandar/prd-reviewer-cli.git";

fn install_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    format!("{}/.tuntun/cli", home)
}

pub fn run() {
    println!("Checking for updates...");
    println!();

    let dir = install_dir();

    // Check if cargo is available
    if Command::new("cargo").arg("--version").output().is_err() {
        eprintln!("Error: `cargo` not found. Install Rust first:");
        eprintln!("  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh");
        std::process::exit(1);
    }

    // Get current version
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    // Fresh clone or pull
    if std::path::Path::new(&dir).join(".git").exists() {
        pull_repo(&dir);
    } else {
        if std::path::Path::new(&dir).exists() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        clone_repo(&dir);
    }

    // Read remote version from Cargo.toml
    let remote_version = read_remote_version(&dir).unwrap_or_default();

    if !remote_version.is_empty() && remote_version == current_version {
        println!("  Already up to date (v{}).", current_version);
        // Still sync skills in case templates changed
        sync_current_project();
        return;
    }

    if !remote_version.is_empty() {
        println!("  New version available: v{} → v{}", current_version, remote_version);
    }

    // Install
    println!("  Building and installing...");
    let status = Command::new("cargo")
        .args(["install", "--path", ".", "--force"])
        .current_dir(&dir)
        .status()
        .expect("Failed to run cargo install");

    if !status.success() {
        eprintln!();
        eprintln!("Update failed. Try manually:");
        eprintln!("  git clone {} {}", REPO_URL, dir);
        eprintln!("  cd {} && cargo install --path .", dir);
        std::process::exit(1);
    }

    println!();
    println!("Updated successfully: v{} → v{}", current_version, remote_version);

    // Auto-sync skills to the current working directory if it's a prd-reviewer project
    sync_current_project();
}

/// After update, sync skills/agent/CLAUDE.md in the current directory (if applicable).
fn sync_current_project() {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return,
    };

    if !super::init::has_tuntun_files(&cwd) {
        return;
    }

    println!();
    println!("Syncing skills in: {}", cwd.display());
    super::init::sync_files(&cwd, false);
    println!();
    println!("Skills and conventions updated.");
}

fn read_remote_version(dir: &str) -> Option<String> {
    let cargo_toml = std::path::Path::new(dir).join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_toml).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("version") {
            // Parse: version = "x.y.z"
            if let Some(val) = line.split('=').nth(1) {
                let version = val.trim().trim_matches('"').trim().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
    }
    None
}

fn clone_repo(dir: &str) {
    println!("  Cloning from {}...", REPO_URL);
    let status = Command::new("git")
        .args(["clone", "--depth", "1", REPO_URL, dir])
        .status()
        .expect("Failed to run git clone");

    if !status.success() {
        eprintln!("Error: Failed to clone repository from {}", REPO_URL);
        std::process::exit(1);
    }
}

fn pull_repo(dir: &str) {
    println!("  Pulling latest changes...");
    let status = Command::new("git")
        .args(["fetch", "--depth", "1", "origin", "main"])
        .current_dir(dir)
        .status();

    if let Ok(s) = status {
        if s.success() {
            let _ = Command::new("git")
                .args(["reset", "--hard", "origin/main"])
                .current_dir(dir)
                .status();
            return;
        }
    }

    // If pull fails, re-clone
    let _ = std::fs::remove_dir_all(dir);
    clone_repo(dir);
}
