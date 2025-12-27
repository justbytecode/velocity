//! Sandboxed script execution

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::core::{VelocityResult, VelocityError};
use crate::security::permissions::{Permission, PermissionManager};

/// Script sandbox for safe execution
pub struct ScriptSandbox {
    /// Working directory
    working_dir: PathBuf,
    /// Environment variables
    env: HashMap<String, String>,
    /// Permission manager
    permissions: Option<PermissionManager>,
}

impl ScriptSandbox {
    /// Create a new sandbox
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            working_dir,
            env: HashMap::new(),
            permissions: None,
        }
    }

    /// Set environment variables
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Set permission manager
    pub fn with_permissions(mut self, permissions: PermissionManager) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Execute a script
    pub async fn execute(
        &self,
        package: &str,
        script: &str,
        args: &[String],
    ) -> VelocityResult<ScriptResult> {
        // Check script permission
        if let Some(ref perms) = self.permissions {
            let decision = perms.check(package, Permission::Scripts);
            match decision {
                crate::security::permissions::PermissionDecision::Deny => {
                    return Err(VelocityError::PermissionDenied {
                        package: package.to_string(),
                        permission: "scripts".to_string(),
                    });
                }
                crate::security::permissions::PermissionDecision::Prompt => {
                    // In a real implementation, we would prompt the user here
                    tracing::warn!(
                        "Script execution for '{}' requires approval",
                        package
                    );
                }
                crate::security::permissions::PermissionDecision::Allow => {}
            }
        }

        // Determine shell
        let (shell, shell_arg) = if cfg!(windows) {
            ("cmd.exe", "/c")
        } else {
            ("sh", "-c")
        };

        // Build command
        let full_script = if args.is_empty() {
            script.to_string()
        } else {
            format!("{} {}", script, args.join(" "))
        };

        // Add node_modules/.bin to PATH
        let node_modules_bin = self.working_dir.join("node_modules").join(".bin");
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        let path_separator = if cfg!(windows) { ";" } else { ":" };
        path_env = format!("{}{}{}", node_modules_bin.display(), path_separator, path_env);

        // Execute
        let output = Command::new(shell)
            .arg(shell_arg)
            .arg(&full_script)
            .current_dir(&self.working_dir)
            .env("PATH", &path_env)
            .envs(&self.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        Ok(ScriptResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Execute a script with inherited stdio (for interactive scripts)
    pub async fn execute_interactive(
        &self,
        package: &str,
        script: &str,
        args: &[String],
    ) -> VelocityResult<i32> {
        let (shell, shell_arg) = if cfg!(windows) {
            ("cmd.exe", "/c")
        } else {
            ("sh", "-c")
        };

        let full_script = if args.is_empty() {
            script.to_string()
        } else {
            format!("{} {}", script, args.join(" "))
        };

        let node_modules_bin = self.working_dir.join("node_modules").join(".bin");
        let mut path_env = std::env::var("PATH").unwrap_or_default();
        let path_separator = if cfg!(windows) { ";" } else { ":" };
        path_env = format!("{}{}{}", node_modules_bin.display(), path_separator, path_env);

        let status = Command::new(shell)
            .arg(shell_arg)
            .arg(&full_script)
            .current_dir(&self.working_dir)
            .env("PATH", &path_env)
            .envs(&self.env)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await?;

        Ok(status.code().unwrap_or(1))
    }
}

/// Result of script execution
#[derive(Debug)]
pub struct ScriptResult {
    /// Whether the script succeeded
    pub success: bool,
    /// Exit code if available
    pub exit_code: Option<i32>,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
}
