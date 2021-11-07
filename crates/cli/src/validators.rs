use std::path::PathBuf;

pub fn is_file(file: &str) -> Result<(), String> {
    if PathBuf::from(file).is_file() {
        Ok(())
    } else {
        Err(format!("Given filepath `{}` is not a file.", file))
    }
}
