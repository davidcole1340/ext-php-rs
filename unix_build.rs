use std::{path::PathBuf, process::Command};

use anyhow::{bail, Context, Result};

use crate::{PHPInfo, PHPProvider};

pub struct Provider {}

impl Provider {
    /// Runs `php-config` with one argument, returning the stdout.
    fn php_config(&self, arg: &str) -> Result<String> {
        let cmd = Command::new("/home/ptondereau/Code/php-debug/bin/php-config")
            .arg(arg)
            .output()
            .context("Failed to run `php-config`")?;
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        if !cmd.status.success() {
            let stderr = String::from_utf8_lossy(&cmd.stderr);
            bail!("Failed to run `php-config`: {} {}", stdout, stderr);
        }
        Ok(stdout.to_string())
    }
}

impl<'a> PHPProvider<'a> for Provider {
    fn new(_: &'a PHPInfo) -> Result<Self> {
        Ok(Self {})
    }

    fn get_includes(&self) -> Result<Vec<PathBuf>> {
        Ok(self
            .php_config("--includes")?
            .split(' ')
            .map(|s| s.trim_start_matches("-I"))
            .map(PathBuf::from)
            .collect())
    }

    fn get_defines(&self) -> Result<Vec<(&'static str, &'static str)>> {
        Ok(vec![])
    }
}
