use std::path::PathBuf;

/// Calls ```get_output_working_dir``` for name conversion and creates the output directory on the filesystem.
pub fn create_output_working_dir(input_dir: &PathBuf) -> Result<(), String> {
    let path = crate::path::get_output_working_dir(input_dir)?;
    if path.exists() {
        return Ok(());
    }
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(msg) => Err(msg.to_string()),
    }
}
