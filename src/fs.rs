//! Contains supporting functions that alter the file system.

use {indicatif::ProgressBar, log::error, std::path::PathBuf};

/// Calls ```get_output_working_dir``` for name conversion and creates the output directory on the filesystem.
pub fn create_output_working_dir(input_dir: &PathBuf) -> Result<(), String> {
    let path = crate::path::get_output_working_dir(input_dir)?;
    if path.exists() {
        return Ok(());
    }
    match std::fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(msg) => Err(msg.to_string()),
    }
}

/// Recreates the input directory structure in the output working directory.
pub fn create_output_directories(
    input_dir: &PathBuf,
    input_file_names: &Vec<PathBuf>,
    progressbar: Option<ProgressBar>,
) {
    for file_name in input_file_names {
        let mut f = match crate::path::remove_base_dir(&input_dir, file_name) {
            Ok(f) => f,
            Err(msg) => {
                error!("{}", msg);
                return;
            }
        };
        f.pop();
        let f = crate::path::get_output_working_dir(&input_dir)
            .unwrap()
            .join(f);
        if !f.is_dir() {
            match std::fs::create_dir_all(&f) {
                Ok(()) => {
                    if let Some(ref pb) = progressbar {
                        pb.inc(1);
                    }
                }
                Err(msg) => {
                    match f.to_str() {
                        Some(v) => {
                            error!(
                                "Could not create folder {}! Error: {}",
                                v, msg
                            );
                        }
                        None => {
                            error!("Could not create folder! Error {}", msg);
                        }
                    };
                }
            };
        }
    }
}
