use std::{path::PathBuf, process::Command};

use anyhow::{anyhow, bail, Context, Result};
use php_discovery::build::Build;

use crate::PHPProvider;

pub struct Provider<'a> {
    build: &'a Build,
}

impl<'a> Provider<'a> {
    /// Runs `php-config` with one argument, returning the stdout.
    fn php_config(&self, arg: &str) -> Result<String> {
        let config = self.build.config().ok_or_else(|| {
            anyhow!(
                "unable to locate `php-config` binary for `{}`.",
                self.build.binary.to_string_lossy()
            )
        })?;

        let cmd = Command::new(config)
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

impl<'a> PHPProvider<'a> for Provider<'a> {
    fn new(build: &'a Build) -> Result<Self> {
        Ok(Self { build })
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
