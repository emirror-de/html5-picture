pub mod webp;

use {
    walkdir::DirEntry,
    std::path::PathBuf,
};

pub fn get_output_dir_name(input_dir: &str) -> Result<PathBuf, String> {
    let mut path = std::path::PathBuf::from(input_dir);
    if let None = &path.file_name() {
        return Err(String::from("The last segment of the path is not valid!"));
    };
    let new_file_name = path.file_name().unwrap().to_str();
    let new_file_name = match new_file_name {
        Some(s) => String::from(s), // prevents issue #59159
        None => return Err(String::from("Failed to convert last segment of input dir to string")),
    };
    path.set_file_name(&format!("{}-html5picture", new_file_name));
    Ok(path)
}

pub fn create_output_dir(input_dir: &str) -> Result<(), String> {
    let path = get_output_dir_name(input_dir)?;
    match std::fs::create_dir(path) {
        Ok(_) => Ok(()),
        Err(msg) => Err(msg.to_string()),
    }
}

pub fn is_png(input: &DirEntry) -> bool {
    input.file_name()
        .to_str()
        .map(|s| s.ends_with(".png"))
        .unwrap_or(false)
}
