use std::{
    convert::TryInto,
    env,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Cursor, Write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};
use bindgen::RustTarget;
use regex::Regex;

const MIN_PHP_API_VER: u32 = 20200930;
const MAX_PHP_API_VER: u32 = 20210902;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const DEFINES: &[&str] = &["ZEND_WIN32", "WINDOWS", "PHP_WIN32", "WIN32"];

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

#[derive(Debug)]
struct LinkerVersion {
    major: u32,
    minor: u32,
}

/// Retrieve the version of a MSVC linker.
fn get_linker_version(linker: &Path) -> Result<LinkerVersion> {
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
    Ok(LinkerVersion {
        major: components[0],
        minor: components[1],
    })
}

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

#[cfg(windows)]
const WHICH: &str = "where";
#[cfg(not(windows))]
const WHICH: &str = "which";

fn find_executable(name: &str) -> Option<PathBuf> {
    let cmd = Command::new(WHICH).arg(name).output().ok()?;
    if cmd.status.success() {
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        Some(stdout.trim().into())
    } else {
        None
    }
}

fn find_php() -> Result<PathBuf> {
    // If PHP path is given via env, it takes priority.
    let env = std::env::var("PHP");
    if let Ok(env) = env {
        return Ok(env.into());
    }

    find_executable("php").context("Could not find PHP path. Please ensure `php` is in your PATH or the `PHP` environment variable is set.")
}

// https://windows.php.net/downloads/releases/php-devel-pack-8.1.3-nts-Win32-vs16-x64.zip
// https://windows.php.net/downloads/releases/php-devel-pack-8.1.3-Win32-vs16-x64.zip

struct DevelPack(PathBuf);

impl DevelPack {
    pub fn includes(&self) -> PathBuf {
        self.0.join("include")
    }

    pub fn php_lib(&self) -> PathBuf {
        self.0.join("lib").join("php8.lib")
    }

    pub fn include_paths(&self) -> Vec<PathBuf> {
        let includes = self.includes();
        ["", "main", "Zend", "TSRM", "ext"]
            .iter()
            .map(|p| includes.join(p))
            .collect()
    }

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

fn download_devel_pack(version: &str, is_zts: bool, arch: &str) -> Result<DevelPack> {
    let zip_name = format!(
        "php-devel-pack-{}{}-Win32-{}-{}.zip",
        version,
        if is_zts { "" } else { "-nts" },
        "vs16", /* TODO(david): At the moment all PHPs supported by ext-php-rs use VS16 so this
                 * is constant. */
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
        let mut zip_content =
            zip::read::ZipArchive::new(&mut content).context("Failed to unzip development pack")?;
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

fn get_defines(is_debug: bool) -> Vec<(&'static str, &'static str)> {
    vec![
        ("ZEND_WIN32", "1"),
        ("PHP_WIN32", "1"),
        ("WINDOWS", "1"),
        ("WIN32", "1"),
        ("ZEND_DEBUG", if is_debug { "1" } else { "0" }),
    ]
}

fn build_wrapper(defines: &[(&str, &str)], includes: &[PathBuf]) -> Result<()> {
    let mut build = cc::Build::new();
    for (var, val) in defines {
        build.define(*var, *val);
    }
    build
        .file("src/wrapper.c")
        .includes(includes)
        .try_compile("wrapper")
        .context("Failed to compile ext-php-rs C interface")?;
    Ok(())
}

fn generate_bindings(
    defines: &[(&str, &str)],
    includes: &[PathBuf],
    php_lib_name: &str,
) -> Result<()> {
    let out_dir = env::var_os("OUT_DIR").context("Failed to get OUT_DIR")?;
    let out_path = PathBuf::from(out_dir).join("bindings.rs");
    let mut bindgen = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(
            includes
                .iter()
                .map(|inc| format!("-I{}", inc.to_string_lossy())),
        )
        .clang_args(
            defines
                .iter()
                .map(|(var, val)| format!("-D{}={}", var, val)),
        )
        // .clang_args(&["-DMSC_VER=1800", "-DZEND_FASTCALL=__vectorcall"])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .no_copy("_zval_struct")
        .no_copy("_zend_string")
        .no_copy("_zend_array")
        .no_debug("_zend_function_entry") // On Windows when the handler uses vectorcall, Debug cannot be derived so we do it in code.
        .layout_tests(env::var("EXT_PHP_RS_TEST").is_ok())
        .rust_target(RustTarget::Nightly);

    for binding in ALLOWED_BINDINGS.iter() {
        bindgen = bindgen
            .allowlist_function(binding)
            .allowlist_type(binding)
            .allowlist_var(binding);
    }

    let bindings = bindgen
        .generate()
        .map_err(|_| anyhow!("Unable to generate bindings for PHP"))?
        .to_string();

    // For some reason some symbols don't link without a `#[link(name = "php8")]`
    // attribute on each extern block. Bindgen doesn't give us the option to add
    // this so we need to add it manually.
    //
    // Also, some functions need to be vectorcall. Primarily array functions.
    let out_file = File::create(&out_path).context("Failed to open output bindings file")?;
    let mut writer = BufWriter::new(out_file);
    for line in bindings.lines() {
        match &*line {
            "extern \"C\" {" | "extern \"fastcall\" {" => {
                writeln!(&mut writer, "#[link(name = \"{}\")]", php_lib_name)?;
            }
            _ => {}
        }
        writeln!(&mut writer, "{}", line)?;
    }

    Ok(())
}

#[cfg(windows)]
fn main() -> Result<()> {
    let manifest: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    for path in [
        manifest.join("src").join("wrapper.h"),
        manifest.join("src").join("wrapper.c"),
        manifest.join("allowed_bindings.rs"),
    ] {
        println!("cargo:rerun-if-changed={}", path.to_string_lossy());
    }

    let php = find_php()?;
    let info = PHPInfo::get(&php)?;
    let arch = info.architecture()?;
    let is_zts = info.thread_safety()?;
    let version = info.version()?;
    let debug = info.debug()?;
    let devkit = download_devel_pack(version, is_zts, arch)?;
    let includes = devkit.include_paths();
    let defines = get_defines(debug);
    let php_lib = devkit.php_lib();
    let php_lib_name = php_lib
        .file_stem()
        .context("Failed to get PHP library name")?
        .to_string_lossy();

    build_wrapper(&defines, &includes)?;
    generate_bindings(&defines, &includes, &*php_lib_name)?;

    println!("cargo:rustc-cfg=php81");
    if debug {
        println!("cargo:rustc-cfg=php_debug");
    }
    println!("cargo:rustc-link-lib=dylib={}", php_lib_name);
    println!(
        "cargo:rustc-link-search={}",
        php_lib
            .parent()
            .context("Failed to get PHP library parent directory")?
            .to_string_lossy()
    );

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    // rerun if wrapper header is changed
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=src/wrapper.c");
    println!("cargo:rerun-if-changed=allowed_bindings.rs");

    let out_dir = env::var_os("OUT_DIR").expect("Failed to get OUT_DIR");
    let out_path = PathBuf::from(out_dir).join("bindings.rs");

    // check for docs.rs and use stub bindings if required
    if env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=docs.rs detected - using stub bindings");
        println!("cargo:rustc-cfg=php_debug");
        println!("cargo:rustc-cfg=php81");

        std::fs::copy("docsrs_bindings.rs", out_path)
            .expect("Unable to copy docs.rs stub bindings to output directory.");
        return;
    }

    // use php-config to fetch includes
    let includes_cmd = Command::new("php-config")
        .arg("--includes")
        .output()
        .expect("Unable to run `php-config`. Please ensure it is visible in your PATH.");

    if !includes_cmd.status.success() {
        let stderr = String::from_utf8(includes_cmd.stderr)
            .unwrap_or_else(|_| String::from("Unable to read stderr"));
        panic!("Error running `php-config`: {}", stderr);
    }

    // Ensure the PHP API version is supported.
    // We could easily use grep and sed here but eventually we want to support
    // Windows, so it's easier to just use regex.
    let php_i_cmd = Command::new("php")
        .arg("-i")
        .output()
        .expect("Unable to run `php -i`. Please ensure it is visible in your PATH.");

    if !php_i_cmd.status.success() {
        let stderr = str::from_utf8(&includes_cmd.stderr).unwrap_or("Unable to read stderr");
        panic!("Error running `php -i`: {}", stderr);
    }

    let api_ver = Regex::new(r"PHP API => ([0-9]+)")
        .unwrap()
        .captures_iter(
            str::from_utf8(&php_i_cmd.stdout).expect("Unable to parse `php -i` stdout as UTF-8"),
        )
        .next()
        .and_then(|ver| ver.get(1))
        .and_then(|ver| ver.as_str().parse::<u32>().ok())
        .expect("Unable to retrieve PHP API version from `php -i`.");

    if !(MIN_PHP_API_VER..=MAX_PHP_API_VER).contains(&api_ver) {
        panic!("The current version of PHP is not supported. Current PHP API version: {}, requires a version between {} and {}", api_ver, MIN_PHP_API_VER, MAX_PHP_API_VER);
    }

    // Infra cfg flags - use these for things that change in the Zend API that don't
    // rely on a feature and the crate user won't care about (e.g. struct field
    // changes). Use a feature flag for an actual feature (e.g. enums being
    // introduced in PHP 8.1).
    //
    // PHP 8.0 is the baseline - no feature flags will be introduced here.
    //
    // The PHP version cfg flags should also stack - if you compile on PHP 8.2 you
    // should get both the `php81` and `php82` flags.
    const PHP_81_API_VER: u32 = 20210902;

    if api_ver >= PHP_81_API_VER {
        println!("cargo:rustc-cfg=php81");
    }

    let includes =
        String::from_utf8(includes_cmd.stdout).expect("unable to parse `php-config` stdout");

    // Build `wrapper.c` and link to Rust.
    cc::Build::new()
        .file("src/wrapper.c")
        .includes(
            str::replace(includes.as_ref(), "-I", "")
                .split(' ')
                .map(Path::new),
        )
        .compile("wrapper");

    let mut bindgen = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_args(includes.split(' '))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .no_copy("_zval_struct")
        .no_copy("_zend_string")
        .no_copy("_zend_array")
        .layout_tests(env::var("EXT_PHP_RS_TEST").is_ok());

    for binding in ALLOWED_BINDINGS.iter() {
        bindgen = bindgen
            .allowlist_function(binding)
            .allowlist_type(binding)
            .allowlist_var(binding);
    }

    bindgen
        .generate()
        .expect("Unable to generate bindings for PHP")
        .write_to_file(out_path)
        .expect("Unable to write bindings file.");

    let configure = Configure::get();

    if configure.has_zts() {
        println!("cargo:rustc-cfg=php_zts");
    }

    if configure.debug() {
        println!("cargo:rustc-cfg=php_debug");
    }
}

struct PHPInfo(String);

impl PHPInfo {
    pub fn get(php: &Path) -> Result<Self> {
        let cmd = Command::new(php)
            .arg("-i")
            .output()
            .context("Failed to call `php -i`")?;
        if !cmd.status.success() {
            bail!("Failed to call `php -i` status code {}", cmd.status);
        }
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        Ok(Self(stdout.to_string()))
    }

    pub fn architecture(&self) -> Result<&str> {
        self.get_key("Architecture")
            .context("Could not find architecture of PHP")
    }

    pub fn thread_safety(&self) -> Result<bool> {
        Ok(self
            .get_key("Thread Safety")
            .context("Could not find thread safety of PHP")?
            == "enabled")
    }

    pub fn debug(&self) -> Result<bool> {
        Ok(self
            .get_key("Debug Build")
            .context("Could not find debug build of PHP")?
            == "yes")
    }

    pub fn version(&self) -> Result<&str> {
        self.get_key("PHP Version")
            .context("Failed to get PHP version")
    }

    fn get_key(&self, key: &str) -> Option<&str> {
        let split = format!("{} => ", key);
        for line in self.0.lines() {
            let components: Vec<_> = line.split(&split).collect();
            if components.len() > 1 {
                return Some(components[1]);
            }
        }
        None
    }
}

struct Configure(String);

impl Configure {
    pub fn get() -> Self {
        let cmd = Command::new("php-config")
        .arg("--configure-options")
        .output()
        .expect("Unable to run `php-config --configure-options`. Please ensure it is visible in your PATH.");

        if !cmd.status.success() {
            let stderr = String::from_utf8(cmd.stderr)
                .unwrap_or_else(|_| String::from("Unable to read stderr"));
            panic!("Error running `php -i`: {}", stderr);
        }

        // check for the ZTS feature flag in configure
        let stdout =
            String::from_utf8(cmd.stdout).expect("Unable to read stdout from `php-config`.");
        Self(stdout)
    }

    pub fn has_zts(&self) -> bool {
        self.0.contains("--enable-zts")
    }

    pub fn debug(&self) -> bool {
        self.0.contains("--enable-debug")
    }
}

// Mock macro for the `allowed_bindings.rs` script.
macro_rules! bind {
    ($($s: ident),*) => {
        &[$(
            stringify!($s),
        )*]
    }
}

/// Array of functions/types used in `ext-php-rs` - used to allowlist when
/// generating bindings, as we don't want to generate bindings for everything
/// (i.e. stdlib headers).
const ALLOWED_BINDINGS: &[&str] = include!("allowed_bindings.rs");
