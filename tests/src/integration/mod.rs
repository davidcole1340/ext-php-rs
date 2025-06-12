pub mod array;
pub mod binary;
pub mod bool;
pub mod callable;
pub mod class;
pub mod closure;
pub mod defaults;
pub mod exception;
pub mod globals;
pub mod iterator;
pub mod magic_method;
pub mod nullable;
pub mod number;
pub mod object;
pub mod string;
pub mod types;
pub mod variadic_args;

#[cfg(test)]
mod test {
    use std::env;

    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::Once;

    static BUILD: Once = Once::new();

    fn setup() {
        BUILD.call_once(|| {
            assert!(Command::new("cargo")
                .arg("build")
                .output()
                .expect("failed to build extension")
                .status
                .success());
        });
    }

    /// Finds the location of an executable `name`.
    #[must_use]
    pub fn find_executable(name: &str) -> Option<PathBuf> {
        const WHICH: &str = if cfg!(windows) { "where" } else { "which" };
        let cmd = Command::new(WHICH).arg(name).output().ok()?;
        if cmd.status.success() {
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            stdout.trim().lines().next().map(|l| l.trim().into())
        } else {
            None
        }
    }

    /// Returns an environment variable's value as a `PathBuf`
    pub fn path_from_env(key: &str) -> Option<PathBuf> {
        std::env::var_os(key).map(PathBuf::from)
    }

    /// Finds the location of the PHP executable.
    fn find_php() -> Result<PathBuf, String> {
        // If path is given via env, it takes priority.
        if let Some(path) = path_from_env("PHP") {
            if !path
                .try_exists()
                .map_err(|e| format!("Could not check existence: {e}"))?
            {
                // If path was explicitly given and it can't be found, this is a hard error
                return Err(format!("php executable not found at {path:?}"));
            }
            return Ok(path);
        }
        Ok(find_executable("php").ok_or(
            "Could not find PHP executable. \
            Please ensure `php` is in your PATH or the `PHP` environment variable is set.",
        )?)
    }

    pub fn run_php(file: &str) -> bool {
        setup();
        let mut path = env::current_dir().expect("Could not get cwd");
        path.pop();
        path.push("target");
        path.push("debug");
        path.push(if std::env::consts::DLL_EXTENSION == "dll" {
            "tests"
        } else {
            "libtests"
        });
        path.set_extension(std::env::consts::DLL_EXTENSION);
        let output = Command::new(find_php().expect("Could not find PHP executable"))
            .arg(format!("-dextension={}", path.to_str().unwrap()))
            .arg("-dassert.active=1")
            .arg("-dassert.exception=1")
            .arg("-dzend.assertions=1")
            .arg(format!("src/integration/{file}"))
            .output()
            .expect("failed to run php file");
        if output.status.success() {
            true
        } else {
            panic!(
                "
                status: {}
                stdout: {}
                stderr: {}
                ",
                output.status,
                String::from_utf8(output.stdout).unwrap(),
                String::from_utf8(output.stderr).unwrap()
            );
        }
    }
}
