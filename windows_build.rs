use std::{
    convert::TryFrom,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, Cursor, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
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

    /// Checks whether the rustc linker is compatible with the linker used in
    /// the PHP development kit which was downloaded.
    ///
    /// If not compatible, attempts to find a compatible linker and notifies the
    /// user if one is found.
    fn check_linker_compatibility(&self) -> Result<()> {
        let rustc_linker = get_rustc_linker()?;
        let rustc_linker_version = LinkerVersion::from_linker_path(&rustc_linker)?;
        let php_linker_version = self.devel.linker_version()?;
        let compatible = php_linker_version.is_forwards_compatible(&rustc_linker_version);
        if compatible {
            Ok(())
        } else {
            let mut error = format!("Incompatible linker versions. PHP was linked with MSVC {}, while Rust is using MSVC {}.", php_linker_version, rustc_linker_version);
            if let Some(potential_linker) = find_potential_linker(&php_linker_version)? {
                let path = potential_linker.path.to_string_lossy();
                let target_triple = std::env::var("TARGET").expect("Failed to get target triple");
                error.push_str(&format!(
                    "
A potentially compatible linker was found (MSVC version {}) located at `{}`.

Use this linker by creating a `.cargo/config.toml` file in your extension's
manifest directory with the following content:
```
[target.{}]
linker = \"{}\"
```
",
                    potential_linker.version,
                    path,
                    target_triple,
                    path.escape_default()
                ))
            } else {
                error.push_str(&format!(
                    "
You need a linker with a version earlier or equal to MSVC {}.
Download MSVC from https://visualstudio.microsoft.com/vs/features/cplusplus/.
Make sure to select C++ Development Tools in the installer.
You can correspond MSVC version with Visual Studio version
here: https://en.wikipedia.org/wiki/Microsoft_Visual_C%2B%2B#Internal_version_numbering
",
                    php_linker_version
                ));
            }
            bail!(error);
        }
    }
}

impl<'a> PHPProvider<'a> for Provider<'a> {
    fn new(info: &'a PHPInfo) -> Result<Self> {
        let version = info.version()?;
        let is_zts = info.thread_safety()?;
        let arch = info.architecture()?;
        let devel = DevelPack::new(version, is_zts, arch)?;
        Ok(Self { info, devel })
    }

    fn get_includes(&self) -> Result<Vec<PathBuf>> {
        Ok(self.devel.include_paths())
    }

    fn get_defines(&self) -> Result<Vec<(&'static str, &'static str)>> {
        Ok(vec![
            ("ZEND_WIN32", "1"),
            ("PHP_WIN32", "1"),
            ("WINDOWS", "1"),
            ("WIN32", "1"),
            ("ZEND_DEBUG", if self.info.debug()? { "1" } else { "0" }),
        ])
    }

    fn write_bindings(&self, bindings: String, writer: &mut impl Write) -> Result<()> {
        // For some reason some symbols don't link without a `#[link(name = "php8")]`
        // attribute on each extern block. Bindgen doesn't give us the option to add
        // this so we need to add it manually.
        let php_lib_name = self.get_php_lib_name()?;
        for line in bindings.lines() {
            match &*line {
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

#[derive(Debug)]
struct LinkerVersion {
    major: u32,
    minor: u32,
}

impl LinkerVersion {
    /// Retrieve the version of a MSVC linker.
    fn from_linker_path(linker: &Path) -> Result<Self> {
        let cmd = Command::new(linker)
            .output()
            .context("Failed to call linker")?;
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        let linker = stdout
            .split("\r\n")
            .next()
            .context("Linker output was empty")?;
        let version = linker
            .split(' ')
            .last()
            .context("Linker version string was empty")?;
        let components = version
            .split('.')
            .take(2)
            .map(|v| v.parse())
            .collect::<Result<Vec<_>, _>>()
            .context("Linker version component was empty")?;
        Ok(Self {
            major: components[0],
            minor: components[1],
        })
    }

    /// Checks if this linker is forwards-compatible with another linker.
    fn is_forwards_compatible(&self, other: &LinkerVersion) -> bool {
        // To be forwards compatible, the other linker must have the same major
        // version and the minor version must greater or equal to this linker.
        self.major == other.major && self.minor >= other.minor
    }
}

impl Display for LinkerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Returns the path to rustc's linker.
fn get_rustc_linker() -> Result<PathBuf> {
    // `RUSTC_LINKER` is set if the linker has been overriden anywhere.
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

/// Uses vswhere to find all the linkers installed on a system.
fn get_linkers(vswhere: &Path) -> Result<Vec<PathBuf>> {
    let cmd = Command::new(vswhere)
        .arg("-all")
        .arg("-prerelease")
        .arg("-format")
        .arg("value")
        .arg("-utf8")
        .arg("-find")
        .arg(r"VC\**\link.exe")
        .output()
        .context("Failed to call vswhere")?;
    let stdout = String::from_utf8_lossy(&cmd.stdout);
    let linkers: Vec<_> = stdout
        .split("\r\n")
        .map(PathBuf::from)
        .filter(|linker| linker.exists())
        .collect();
    Ok(linkers)
}

/// Attempts to find a potential linker that is compatible with PHP.
///
/// It must fit the following criteria:
///
/// 1. It must be forwards compatible with the PHP linker.
/// 2. The linker target architecture must match the target triple architecture.
/// 3. Optionally, the linker host architecture should match the host triple
/// architecture. On x86_64 systems, if a x64 host compiler is not found it will
/// fallback to x86.
///
/// Returns an error if there is an error. Returns None if no linker could be
/// found.
fn find_potential_linker(php_linker: &LinkerVersion) -> Result<Option<Linker>> {
    let vswhere = find_vswhere().context("Could not find `vswhere`")?;
    let linkers = get_linkers(&vswhere)?;
    let host_arch = msvc_host_arch()?;
    let target_arch = msvc_target_arch()?;
    let mut prelim_linker = None;

    for linker in &linkers {
        let linker = Linker::from_linker_path(linker)?;
        if php_linker.is_forwards_compatible(&linker.version) && linker.target_arch == target_arch {
            if linker.host_arch == host_arch {
                return Ok(Some(linker));
            } else if prelim_linker.is_none()
                && host_arch == Arch::X64
                && linker.host_arch == Arch::X86
            {
                // This linker will work - the host architectures do not match but that's OK for
                // x86_64.
                prelim_linker.replace(linker);
            }
        }
    }
    Ok(prelim_linker)
}

#[derive(Debug)]
struct Linker {
    host_arch: Arch,
    target_arch: Arch,
    version: LinkerVersion,
    path: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
enum Arch {
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

impl Linker {
    /// Retrieves information about the linker based on its path.
    fn from_linker_path(linker: &Path) -> Result<Linker> {
        let version = LinkerVersion::from_linker_path(linker)?;
        let target_arch_folder = linker
            .parent()
            .context("Could not get linker parent folder")?;
        let target_arch = Arch::try_from(
            &*target_arch_folder
                .file_stem()
                .context("Could not get linker target architecture")?
                .to_string_lossy()
                .to_lowercase(),
        )?;
        let host_arch = Arch::try_from(
            &*target_arch_folder
                .parent()
                .context("Could not get linker parent folder")?
                .file_stem()
                .context("Could not get linker host architecture")?
                .to_string_lossy()
                .replace("Host", "")
                .to_lowercase(),
        )?;
        Ok(Linker {
            host_arch,
            target_arch,
            version,
            path: linker.to_owned(),
        })
    }
}

/// Returns the architecture of a triple.
fn triple_arch(triple: &str) -> Result<Arch> {
    let arch = triple.split('-').next().context("Triple was invalid")?;
    Ok(match arch {
        "x86_64" => Arch::X64,
        "i686" => Arch::X86,
        "aarch64" => Arch::AArch64,
        a => bail!("Unknown architecture {}", a),
    })
}

/// Returns the architecture of the target the compilation is running on.
///
/// If running on an AArch64 host, X86 is returned as there are no MSVC tools
/// for AArch64 hosts.
fn msvc_host_arch() -> Result<Arch> {
    let host_triple = std::env::var("HOST").context("Failed to get host triple")?;
    Ok(match triple_arch(&host_triple)? {
        Arch::AArch64 => Arch::X86, // AArch64 does not have host tools
        a => a,
    })
}

/// Returns the architecture of the target being compiled for.
fn msvc_target_arch() -> Result<Arch> {
    let host_triple = std::env::var("TARGET").context("Failed to get host triple")?;
    triple_arch(&host_triple)
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
            let request = reqwest::blocking::ClientBuilder::new()
                .user_agent(USER_AGENT)
                .build()
                .context("Failed to create HTTP client")?
                .get(url)
                .send()
                .context("Failed to download development pack")?;
            request
                .error_for_status_ref()
                .context("Failed to download development pack")?;
            let bytes = request
                .bytes()
                .context("Failed to read content from PHP website")?;
            let mut content = Cursor::new(bytes);
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
        self.0.join("lib").join("php8.lib")
    }

    /// Returns a list of include paths to pass to the compiler.
    pub fn include_paths(&self) -> Vec<PathBuf> {
        let includes = self.includes();
        ["", "main", "Zend", "TSRM", "ext"]
            .iter()
            .map(|p| includes.join(p))
            .collect()
    }

    /// Retrieves the version of MSVC PHP was linked with.
    pub fn linker_version(&self) -> Result<LinkerVersion> {
        let config_path = self.includes().join("main").join("config.w32.h");
        let config = File::open(&config_path).context("Failed to open PHP config header")?;
        let reader = BufReader::new(config);
        let mut major = None;
        let mut minor = None;
        for line in reader.lines() {
            let line = line.context("Failed to read line from PHP config header")?;
            if major.is_none() {
                let components: Vec<_> = line.split("#define PHP_LINKER_MAJOR ").collect();
                if components.len() > 1 {
                    major.replace(
                        u32::from_str(components[1])
                            .context("Failed to convert major linker version to integer")?,
                    );
                    continue;
                }
            }
            if minor.is_none() {
                let components: Vec<_> = line.split("#define PHP_LINKER_MINOR ").collect();
                if components.len() > 1 {
                    minor.replace(
                        u32::from_str(components[1])
                            .context("Failed to convert minor linker version to integer")?,
                    );
                    continue;
                }
            }
        }
        Ok(LinkerVersion {
            major: major.context("Failed to read major linker version from config header")?,
            minor: minor.context("Failed to read minor linker version from config header")?,
        })
    }
}

/// Attempt to find a `vswhere` binary in the common locations.
pub fn find_vswhere() -> Option<PathBuf> {
    let candidates = [format!(
        r"{}\Microsoft Visual Studio\Installer\vswhere.exe",
        std::env::var("ProgramFiles(x86)").ok()?,
    )];
    for candidate in candidates {
        let candidate = PathBuf::from(candidate);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
