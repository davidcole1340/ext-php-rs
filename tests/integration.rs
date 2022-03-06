use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};

use anyhow::{bail, Context, Result};
use log::{error, info, warn};
use simple_logger::SimpleLogger;

#[derive(Debug)]
struct Test {
    name: String,
    script: PathBuf,
    test_dir: PathBuf,
}

fn main() -> Result<()> {
    SimpleLogger::new().init()?;

    let rust = Rust::new()?;
    let tests = rust.find_tests()?;
    let mut did_pass = true;

    for test in &tests {
        let ext_path = rust.compile_test(test)?;
        let passed = rust.run_test(test, &ext_path)?;
        if !passed {
            did_pass = false;
        }
    }

    if !did_pass {
        error!("Test failed.");
        exit(1);
    }

    Ok(())
}

struct Rust {
    build_dir: PathBuf,
    deps_dir: PathBuf,
    test_dir: PathBuf,
    ext_php_rs_path: PathBuf,
    test_runner_path: PathBuf,
}

impl Rust {
    pub fn new() -> Result<Self> {
        let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR")
            .context("Could not read `CARGO_MANIFEST_DIR`.")?
            .into();
        let debug_dir = manifest_dir.join("target").join("debug");
        let build_dir = debug_dir.join("phpt");
        let test_dir = manifest_dir.join("tests").join("integration");
        let deps_dir = debug_dir.join("deps");
        let ext_php_rs_path = debug_dir.join("libext_php_rs.rlib");
        let test_runner_path = manifest_dir
            .join("tests")
            .join("integration")
            .join("run-tests.php");
        Ok(Self {
            build_dir,
            test_dir,
            deps_dir,
            ext_php_rs_path,
            test_runner_path,
        })
    }

    fn find_tests(&self) -> Result<Vec<Test>> {
        let mut tests = vec![];
        let dirs = fs::read_dir(&self.test_dir)
            .with_context(|| format!("Failed to read tests from {:?}", self.test_dir))?;
        for dir in dirs {
            let dir = match dir {
                Ok(dir) => dir,
                Err(_) => continue,
            };
            let dir_type = dir
                .file_type()
                .with_context(|| format!("Failed to get type of file for {:?}", dir))?;
            if !dir_type.is_dir() {
                continue;
            }
            let mut script = dir.file_name();
            script.push(".rs");
            let script = dir.path().join(script);
            if !script.exists() {
                continue;
            }
            tests.push(Test {
                name: dir
                    .file_name()
                    .into_string()
                    .ok()
                    .context("Could not read test name")?,
                script,
                test_dir: dir.path(),
            });
        }
        Ok(tests)
    }

    pub fn compile_test(&self, test: &Test) -> Result<PathBuf> {
        let output = self.build_dir.join(&test.name);
        let cmd = Command::new("rustc")
            .arg("--crate-name")
            .arg(&test.name)
            .arg("--crate-type")
            .arg("cdylib")
            .arg("-o")
            .arg(&output)
            .arg("-L")
            .arg(&self.deps_dir)
            .arg("--extern")
            .arg(format!(
                "ext_php_rs={}",
                self.ext_php_rs_path.to_str().unwrap()
            ))
            .arg(&test.script)
            .arg("-C")
            .arg("link-arg=-Wl,-undefined,dynamic_lookup")
            .env("CARGO_PKG_NAME", &test.name)
            .status()
            .with_context(|| format!("Failed to compile test {}", &test.name))?;
        if !cmd.success() {
            bail!("rustc exited with error code {}", cmd);
        }
        Ok(output)
    }

    pub fn run_test(&self, test: &Test, ext_path: &Path) -> Result<bool> {
        info!("Running test '{}'", test.name);
        let cmd = Command::new("php")
            .arg(&self.test_runner_path)
            .arg("-P")
            .arg("-d")
            .arg(format!("extension={}", ext_path.to_string_lossy()))
            .arg(&test.test_dir)
            .env("NO_INTERACTION", "")
            .status()?;
        if !cmd.success() {
            warn!(
                "Test '{}' failed with status code {}",
                test.name,
                cmd.code().unwrap_or(-1)
            );
        }
        Ok(cmd.success())
    }
}
