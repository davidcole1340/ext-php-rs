mod ext;
#[macro_use]
pub mod stub_symbols;

use anyhow::{bail, Context, Result as AResult};
use dialoguer::{Confirm, Select};

use std::{
    borrow::Cow,
    ffi::OsString,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::Command,
};

use clap::Parser;

use crate::describe::ToStub;

use self::ext::Ext;

pub type Result = anyhow::Result<()>;

pub fn run() -> Result {
    Args::parse().handle()
}

#[derive(Parser)]
#[clap(
    about = "Installs extensions and generates stub files for PHP extensions generated with `ext-php-rs`.",
    author = "David Cole <david.cole1340@gmail.com>",
    version = env!("CARGO_PKG_VERSION")
)]
enum Args {
    /// Installs the extension in the current PHP installation.
    ///
    /// This copies the extension to the PHP installation and adds the
    /// extension to a PHP configuration file.
    Install(Install),
    /// Generates stub PHP files for the extension.
    ///
    /// These stub files can be used in IDEs to provide typehinting for
    /// extension classes, functions and constants.
    Stubs(Stubs),
}

#[derive(Parser)]
struct Install {
    /// Changes the path that the extension is copied to. This will not
    /// activate the extension unless `ini_path` is also passed.
    #[clap(long)]
    install_dir: Option<PathBuf>,
    /// Path to the `php.ini` file to update with the new extension.
    #[clap(long)]
    ini_path: Option<PathBuf>,
    /// Installs the extension but doesn't enable the extension in the `php.ini`
    /// file.
    #[clap(long)]
    disable: bool,
}

#[derive(Parser)]
struct Stubs {
    /// Path to extension to generate stubs for. Defaults for searching the
    /// directory the executable is located in.
    ext: Option<PathBuf>,
    /// Path used to store generated stub file. Defaults to writing to
    /// `<ext-name>.stubs.php` in the current directory.
    #[clap(short, long)]
    out: Option<PathBuf>,
    /// Print stubs to stdout rather than write to file. Cannot be used with
    /// `out`.
    #[clap(long, conflicts_with = "out")]
    stdout: bool,
}

impl Args {
    pub fn handle(self) -> Result {
        match self {
            Args::Install(install) => install.handle(),
            Args::Stubs(stubs) => stubs.handle(),
        }
    }
}

impl Install {
    pub fn handle(self) -> Result {
        let ext_path = find_ext()?;
        let (mut ext_dir, mut php_ini) = if let Some(install_dir) = self.install_dir {
            (install_dir, None)
        } else {
            let php_config = PhpConfig::new();
            (php_config.get_ext_dir()?, Some(php_config.get_php_ini()?))
        };

        if let Some(ini_path) = self.ini_path {
            php_ini = Some(ini_path);
        }

        if !Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to install the extension `{}`?",
                ext_path
            ))
            .interact()?
        {
            bail!("Installation cancelled.");
        }

        let ext_path = PathBuf::from(ext_path);
        debug_assert!(ext_path.is_file());
        let ext_name = ext_path.file_name().expect("ext path wasn't a filepath");

        if ext_dir.is_dir() {
            ext_dir.push(ext_name);
        }

        std::fs::copy(&ext_path, &ext_dir).with_context(|| {
            "Failed to copy extension from target directory to extension directory"
        })?;

        if let Some(php_ini) = php_ini {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(php_ini)
                .with_context(|| "Failed to open `php.ini`")?;

            let mut ext_line = format!(
                "extension={}",
                ext_name
                    .to_str()
                    .with_context(|| "Failed to convert extension name to unicode string")?
            );

            let mut new_lines = vec![];
            for line in BufReader::new(&file).lines() {
                let line = line.with_context(|| "Failed to read line from `php.ini`")?;
                if !line.contains(&ext_line) {
                    new_lines.push(line);
                }
            }

            // Comment out extension if user specifies disable flag
            if self.disable {
                ext_line.insert(0, ';');
            }

            new_lines.push(ext_line);
            file.write(new_lines.join("\n").as_bytes())
                .with_context(|| "Failed to update `php.ini`")?;
        }

        Ok(())
    }
}

impl Stubs {
    pub fn handle(self) -> Result {
        let php_config = PhpConfig::new();
        let ext_path = if let Some(ext_path) = self.ext {
            ext_path
        } else {
            PathBuf::from(find_ext()?)
        };

        if !ext_path.is_file() {
            bail!("Invalid extension path given, not a file.");
        }

        let php_path = php_config.get_php_path()?;
        let ext = Ext::load(ext_path)?;
        let result = ext.describe();
        let stubs = result
            .to_stub()
            .with_context(|| "Failed to generate stubs.")?;

        if self.stdout {
            print!("{}", stubs);
        } else {
            let out_path = if let Some(out_path) = &self.out {
                Cow::Borrowed(out_path)
            } else {
                let mut cwd = std::env::current_dir()
                    .with_context(|| "Failed to get current working directory")?;
                cwd.push(format!("{}.stubs.php", result.name));
                Cow::Owned(cwd)
            };

            std::fs::write(out_path.as_ref(), &stubs)
                .with_context(|| "Failed to write stubs to file")?;
        }

        Ok(())
    }
}

struct PhpConfig {
    path: OsString,
}

impl PhpConfig {
    /// Creates a new `php-config` instance.
    pub fn new() -> Self {
        Self {
            path: if let Some(php_config) = std::env::var_os("PHP_CONFIG") {
                php_config
            } else {
                OsString::from("php-config")
            },
        }
    }

    /// Calls `php-config` and retrieves the extension directory.
    pub fn get_ext_dir(&self) -> AResult<PathBuf> {
        Ok(PathBuf::from(
            self.exec(
                |cmd| cmd.arg("--extension-dir"),
                "retrieve extension directory",
            )?
            .trim(),
        ))
    }

    /// Calls `php-config` and retrieves the `php.ini` file path.
    pub fn get_php_ini(&self) -> AResult<PathBuf> {
        let mut path = PathBuf::from(
            self.exec(|cmd| cmd.arg("--ini-path"), "retrieve `php.ini` path")?
                .trim(),
        );
        path.push("php.ini");

        if !path.exists() {
            File::create(&path).with_context(|| "Failed to create `php.ini`")?;
        }

        Ok(path)
    }

    /// Calls `php-config` and retrieves the path to the PHP binary.
    pub fn get_php_path(&self) -> AResult<PathBuf> {
        let path = PathBuf::from(
            self.exec(|cmd| cmd.arg("--php-binary"), "retrieve PHP binary path")?
                .trim(),
        );

        if !path.is_file() {
            bail!("Invalid path given for PHP executable");
        }

        Ok(path)
    }

    /// Executes the `php-config` binary. The given function `f` is used to
    /// modify the given mutable [`Command`]. If successful, a [`String`]
    /// representing stdout is returned.
    fn exec<F>(&self, f: F, ctx: &str) -> AResult<String>
    where
        F: FnOnce(&mut Command) -> &mut Command,
    {
        let mut cmd = Command::new(&self.path);
        f(&mut cmd);
        let out = cmd
            .output()
            .with_context(|| format!("Failed to {} from `php-config`", ctx))?;
        String::from_utf8(out.stdout)
            .with_context(|| "Failed to convert `php-config` output to string")
    }
}

/// Attempts to find an extension in the target directory.
fn find_ext() -> AResult<String> {
    let mut target_dir = std::env::current_exe()
        .with_context(|| "Failed to retrieve the path of the current executable")?;
    target_dir.pop();

    let mut target_files: Vec<_> = std::fs::read_dir(target_dir)
        .with_context(|| "Failed to read files from target directory")?
        .collect::<AResult<Vec<_>, std::io::Error>>()?
        .into_iter()
        .filter(|file| {
            file.path().is_file()
                && file.path().extension()
                    == Some(OsString::from(std::env::consts::DLL_EXTENSION).as_os_str())
        })
        .filter_map(|file| file.path().to_str().map(|s| s.to_string()))
        .collect();

    let ext_path = match target_files.len() {
        0 => bail!("No extensions were found in the target directory."),
        1 => target_files.remove(0),
        _ => {
            let chosen = Select::new()
                .with_prompt("There were multiple dynamic libraries detected in the target directory. Which would you like to use?")
                .items(&target_files)
                .interact()?;
            target_files.remove(chosen)
        }
    };

    Ok(ext_path)
}
