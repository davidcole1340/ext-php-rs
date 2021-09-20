use anyhow::{bail, Context, Result};
use diffy::{create_patch, PatchFormatter};
use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    process::Command,
    sync::{mpsc, Arc},
};
use threadpool::ThreadPool;

#[derive(Debug, Default)]
struct Test {
    rust_file: Option<String>,
    php_file: Option<String>,
    stdout_file: Option<String>,
    stderr_file: Option<String>,
}

fn main() -> Result<()> {
    let mut tests = HashMap::new();

    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .with_context(|| "Failed to read `CARGO_MANIFEST_DIR`")?;
    let test_path = format!("{}/tests/phpt", manifest);
    let files = std::fs::read_dir(&test_path)
        .with_context(|| format!("Failed to read tests from `{}`", test_path))?;

    Command::new("cargo")
        .arg("build")
        .status()
        .with_context(|| "Failed to build package")?;

    for i in files {
        let path = i.map(|i| i.path()).with_context(|| "Failed to get path")?;
        let test_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .with_context(|| "Could not extract filename from filepath")?
            .to_string();
        let file_path = path
            .to_str()
            .map(|s| s.to_string())
            .with_context(|| "Failed to get file path")?;
        match path.extension().and_then(|e| e.to_str()).unwrap() {
            "rs" => {
                let test = get(&mut tests, test_name);
                test.rust_file = Some(file_path);
            }
            "php" => {
                let test = get(&mut tests, test_name);
                test.php_file = Some(file_path);
            }
            "stdout" => {
                let test = get(&mut tests, test_name);
                test.stdout_file = Some(file_path);
            }
            "stderr" => {
                let test = get(&mut tests, test_name);
                test.stderr_file = Some(file_path);
            }
            _ => continue,
        }
    }

    let rust = Arc::new(Rust::new()?);
    let n_tests = tests.len();
    let n_workers = 5.min(n_tests);
    let workers = ThreadPool::new(n_workers);

    let (tx, rx) = mpsc::channel();

    for (test_name, test) in tests {
        let tx = tx.clone();
        let rust = rust.clone();
        workers.execute(move || {
            let fail = run_test(&test_name, &test, &rust).err();
            let result = TestResult { test_name, fail };
            let _ = tx.send(result);
        });
    }

    workers.join();
    let mut fails = 0;

    for result in rx.into_iter().take(n_tests) {
        match result.fail {
            None => {
                println!("test {} succeeded", result.test_name)
            }
            Some(fail) => {
                println!("test {} failed", result.test_name);
                println!("{}", fail);
                fails += 1;
            }
        }
    }

    if fails > 0 {
        bail!("some tests had failures")
    } else {
        Ok(())
    }
}

struct TestResult {
    test_name: String,
    fail: Option<TestFailure>,
}

fn run_test(test_name: &String, test: &Test, rust: &Rust) -> Result<(), TestFailure> {
    if !test.validate() {
        return Err(TestFailure::NoTestFile);
    }

    println!("running test `{}`", test_name);
    let ext_path = rust.compile(&test_name, &test)?;
    test.run(&ext_path)?;

    Ok(())
}

fn get(hm: &mut HashMap<String, Test>, test: String) -> &mut Test {
    match hm.entry(test) {
        std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
        std::collections::hash_map::Entry::Vacant(entry) => entry.insert(Test::default()),
    }
}

enum TestFailure {
    NoTestFile,
    InvalidExtPath,
    ExecuteFail(std::io::Error),
    ExpectedOutputReadFail(std::io::Error),
    CompileFailure { stdout: String, stderr: String },
    RunFailure { stdout: String, stderr: String },
    Diff { diff: String, file: String },
}

impl Display for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestFailure::NoTestFile => write!(f, "No test file available."),
            TestFailure::InvalidExtPath => write!(f, "Extension path was invalid."),
            TestFailure::ExecuteFail(e) => write!(f, "Failed to execute program: {}", e),
            TestFailure::ExpectedOutputReadFail(e) => {
                write!(f, "Failed to read from expected output file: {}", e)
            }
            TestFailure::CompileFailure { stdout, stderr } => write!(
                f,
                "Failed to compile test - stdout: {} stderr: {}",
                stdout, stderr
            ),
            TestFailure::RunFailure { stdout, stderr } => write!(
                f,
                "Failed to run test - stdout: {} stderr: {}",
                stdout, stderr
            ),
            TestFailure::Diff { diff, file } => {
                write!(f, "Test diff mismatch on file `{}`\n\n{}", file, diff)
            }
        }
    }
}

impl Test {
    fn validate(&self) -> bool {
        self.rust_file.is_some() && self.php_file.is_some()
    }

    fn run(&self, ext_path: &PathBuf) -> Result<(), TestFailure> {
        let php_file = self
            .php_file
            .as_ref()
            .ok_or_else(|| TestFailure::NoTestFile)?;
        let cmd = Command::new("php")
            .arg(format!(
                "-dextension={}",
                ext_path
                    .to_str()
                    .ok_or_else(|| TestFailure::InvalidExtPath)?
            ))
            .arg(&php_file)
            .output()
            .map_err(|e| TestFailure::ExecuteFail(e))?;

        let stdout = String::from_utf8_lossy(&cmd.stdout);
        let stderr = String::from_utf8_lossy(&cmd.stdout);

        if !cmd.status.success() {
            return Err(TestFailure::RunFailure {
                stdout: stdout.to_string(),
                stderr: stderr.to_string(),
            });
        }

        fn check_diff(file: Option<&str>, stdout: &str) -> Result<(), TestFailure> {
            if let Some(stdout_path) = file {
                let expected = std::fs::read_to_string(stdout_path)
                    .map_err(|e| TestFailure::ExpectedOutputReadFail(e))?;
                let patch = create_patch(&expected, &stdout);

                if patch.hunks().len() > 0 {
                    let formatter = PatchFormatter::new().with_color();
                    let diff = format!("{}", formatter.fmt_patch(&patch));
                    return Err(TestFailure::Diff {
                        diff,
                        file: stdout_path.to_string(),
                    });
                }
            }

            Ok(())
        }

        check_diff(self.stdout_file.as_deref(), &stdout)?;
        check_diff(self.stderr_file.as_deref(), &stderr)?;

        Ok(())
    }
}

struct Rust {
    build_dir: PathBuf,
}

impl Rust {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir().with_context(|| "Failed to retrive current directory")?;
        let build_dir = cwd.join("target").join("debug").join("phpt");
        let _ = std::fs::create_dir(&build_dir);

        Ok(Self { build_dir })
    }

    pub fn compile(&self, test_name: &str, test: &Test) -> Result<PathBuf, TestFailure> {
        let output = self.build_dir.join(test_name);
        let cmd = Command::new("rustc")
            .arg("--crate-name")
            .arg(test_name)
            .arg("--crate-type")
            .arg("cdylib")
            .arg("-o")
            .arg(&output)
            .arg("-L")
            .arg("target/debug/deps")
            .arg("--extern")
            .arg("ext_php_rs=target/debug/libext_php_rs.rlib")
            .arg(
                test.rust_file
                    .as_ref()
                    .ok_or_else(|| TestFailure::NoTestFile)?,
            )
            .env("CARGO_PKG_NAME", test_name)
            .output()
            .map_err(|e| TestFailure::ExecuteFail(e))?;

        if !cmd.status.success() {
            let stdout = String::from_utf8_lossy(&cmd.stdout).to_string();
            let stderr = String::from_utf8_lossy(&cmd.stderr).to_string();
            return Err(TestFailure::CompileFailure { stdout, stderr });
        }

        Ok(output)
    }
}
