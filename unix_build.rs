use std::{path::PathBuf, process::Command};

use anyhow::{bail, Context, Result};

use crate::{find_executable, path_from_env, PHPInfo, PHPProvider};

pub struct Provider {}

impl Provider {
    /// Runs `php-config` with one argument, returning the stdout.
    fn php_config(&self, arg: &str) -> Result<String> {
        let cmd = Command::new(self.find_bin()?)
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

    fn find_bin(&self) -> Result<PathBuf> {
        // If path is given via env, it takes priority.
        if let Some(path) = path_from_env("PHP_CONFIG") {
            if !path.try_exists()? {
                // If path was explicitly given and it can't be found, this is a hard error
                bail!("php-config executable not found at {:?}", path);
            }
            return Ok(path);
        }
        find_executable("php-config").with_context(|| {
            "Could not find `php-config` executable. \
            Please ensure `php-config` is in your PATH or the \
            `PHP_CONFIG` environment variable is set."
        })
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

    fn get_sapis(&self) -> Result<Vec<String>> {
        Ok(self
            .php_config("--php-sapis")?
            .split(' ')
            .map(|s| s.trim_start_matches("-I"))
            .map(|v| v.to_string())
            .collect())
    }

    fn get_defines(&self) -> Result<Vec<(&'static str, &'static str)>> {
        Ok(vec![])
    }

    fn print_extra_link_args(&self) -> Result<()> {
        #[cfg(feature = "link-php")]
        println!("cargo:rustc-link-lib=php");

        Ok(())
    }
}
