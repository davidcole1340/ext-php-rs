use std::{
    convert::TryFrom,
    fmt::Display,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use anyhow::{bail, Context, Result};

use crate::{PHPInfo, PHPProvider};

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct Provider<'a> {
    info: &'a PHPInfo,
    devel: DevelPack,
}

impl<'a> Provider<'a> {
    /// Retrieves the PHP library name (filename without extension).
    fn get_php_lib_name(&self) -> Result<String> {
        Ok(self
            .devel
            .php_lib()
            .file_stem()
            .context("Failed to get PHP library name")?
            .to_string_lossy()
            .to_string())
    }
}

impl<'a> PHPProvider<'a> for Provider<'a> {
    fn new(info: &'a PHPInfo) -> Result<Self> {
        let version = info.version()?;
        let is_zts = info.thread_safety()?;
        let arch = info.architecture()?;
        let devel = DevelPack::new(version, is_zts, arch)?;
        if let Ok(linker) = get_rustc_linker() {
            if looks_like_msvc_linker(&linker) {
                println!("cargo:warning=It looks like you are using a MSVC linker. You may encounter issues when attempting to load your compiled extension into PHP if your MSVC linker version is not compatible with the linker used to compile your PHP. It is recommended to use `rust-lld` as your linker.");
            }
        }

        Ok(Self { info, devel })
    }

    fn get_includes(&self) -> Result<Vec<PathBuf>> {
        Ok(self.devel.include_paths())
    }

    fn get_defines(&self) -> Result<Vec<(&'static str, &'static str)>> {
        let mut defines = vec![
            ("ZEND_WIN32", "1"),
            ("PHP_WIN32", "1"),
            ("WINDOWS", "1"),
            ("WIN32", "1"),
            ("ZEND_DEBUG", if self.info.debug()? { "1" } else { "0" }),
        ];
        if self.info.thread_safety()? {
            defines.push(("ZTS", ""));
        }
        Ok(defines)
    }

    fn write_bindings(&self, bindings: String, writer: &mut impl Write) -> Result<()> {
        // For some reason some symbols don't link without a `#[link(name = "php8")]`
        // attribute on each extern block. Bindgen doesn't give us the option to add
        // this so we need to add it manually.
        let php_lib_name = self.get_php_lib_name()?;
        for line in bindings.lines() {
            match line {
                "extern \"C\" {" | "extern \"fastcall\" {" => {
                    writeln!(writer, "#[link(name = \"{}\")]", php_lib_name)?;
                }
                _ => {}
            }
            writeln!(writer, "{}", line)?;
        }
        Ok(())
    }

    fn print_extra_link_args(&self) -> Result<()> {
        let php_lib_name = self.get_php_lib_name()?;
        let php_lib_search = self
            .devel
            .php_lib()
            .parent()
            .context("Failed to get PHP library parent folder")?
            .to_string_lossy()
            .to_string();
        println!("cargo:rustc-link-lib=dylib={}", php_lib_name);
        println!("cargo:rustc-link-search={}", php_lib_search);
        Ok(())
    }
}

/// Returns the path to rustc's linker.
fn get_rustc_linker() -> Result<PathBuf> {
    // `RUSTC_LINKER` is set if the linker has been overridden anywhere.
    if let Ok(link) = std::env::var("RUSTC_LINKER") {
        return Ok(link.into());
    }

    let link = cc::windows_registry::find_tool(
        &std::env::var("TARGET").context("`TARGET` environment variable not set")?,
        "link.exe",
    )
    .context("Failed to retrieve linker tool")?;
    Ok(link.path().to_owned())
}

/// Checks if a linker looks like the MSVC link.exe linker.
fn looks_like_msvc_linker(linker: &Path) -> bool {
    let command = Command::new(linker).output();
    if let Ok(command) = command {
        let stdout = String::from_utf8_lossy(&command.stdout);
        if stdout.contains("Microsoft (R) Incremental Linker") {
            return true;
        }
    }
    false
}

#[derive(Debug, PartialEq, Eq)]
pub enum Arch {
    X86,
    X64,
    AArch64,
}

impl TryFrom<&str> for Arch {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        Ok(match value {
            "x86" => Self::X86,
            "x64" => Self::X64,
            "arm64" => Self::AArch64,
            a => bail!("Unknown architecture {}", a),
        })
    }
}

impl Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Arch::X86 => "x86",
                Arch::X64 => "x64",
                Arch::AArch64 => "arm64",
            }
        )
    }
}

struct DevelPack(PathBuf);

impl DevelPack {
    /// Downloads a new PHP development pack, unzips it in the build script
    /// temporary directory.
    fn new(version: &str, is_zts: bool, arch: Arch) -> Result<DevelPack> {
        let zip_name = format!(
            "php-devel-pack-{}{}-Win32-{}-{}.zip",
            version,
            if is_zts { "" } else { "-nts" },
            "vs16", /* TODO(david): At the moment all PHPs supported by ext-php-rs use VS16 so
                     * this is constant. */
            arch
        );

        fn download(zip_name: &str, archive: bool) -> Result<PathBuf> {
            let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
            let url = format!(
                "https://windows.php.net/downloads/releases{}/{}",
                if archive { "/archives" } else { "" },
                zip_name
            );
            let response = ureq::AgentBuilder::new()
                .tls_connector(Arc::new(native_tls::TlsConnector::new().unwrap()))
                .build()
                .get(&url)
                .set("User-Agent", USER_AGENT)
                .call()
                .context("Failed to download development pack")?;
            let mut content = vec![];
            response
                .into_reader()
                .read_to_end(&mut content)
                .context("Failed to read development pack")?;
            let mut content = Cursor::new(&mut content);
            let mut zip_content = zip::read::ZipArchive::new(&mut content)
                .context("Failed to unzip development pack")?;
            let inner_name = zip_content
                .file_names()
                .next()
                .and_then(|f| f.split('/').next())
                .context("Failed to get development pack name")?;
            let devpack_path = out_dir.join(inner_name);
            let _ = std::fs::remove_dir_all(&devpack_path);
            zip_content
                .extract(&out_dir)
                .context("Failed to extract devpack to directory")?;
            Ok(devpack_path)
        }

        download(&zip_name, false)
            .or_else(|_| download(&zip_name, true))
            .map(DevelPack)
    }

    /// Returns the path to the include folder.
    pub fn includes(&self) -> PathBuf {
        self.0.join("include")
    }

    /// Returns the path of the PHP library containing symbols for linking.
    pub fn php_lib(&self) -> PathBuf {
        let php_nts = self.0.join("lib").join("php8.lib");
        if php_nts.exists() {
            php_nts
        } else {
            self.0.join("lib").join("php8ts.lib")
        }
    }

    /// Returns a list of include paths to pass to the compiler.
    pub fn include_paths(&self) -> Vec<PathBuf> {
        let includes = self.includes();
        ["", "main", "Zend", "TSRM", "ext"]
            .iter()
            .map(|p| includes.join(p))
            .collect()
    }
}
