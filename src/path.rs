use std::path::{Path, PathBuf};

/// Generates the output directory name by attaching a fixed string to it.
/// ## Example
///
/// ```
/// use {
///     html5_picture::get_output_dir_name,
///     std::path::PathBuf,
/// };
///
/// let input = PathBuf::from("../assets");
/// let input = get_output_dir_name(&input).unwrap();
/// assert_eq!(input.to_str().unwrap(), "../assets-html5picture");
/// ```
pub fn get_output_working_dir(input_dir: &PathBuf) -> Result<PathBuf, String> {
    // validity checks
    if !&input_dir.is_dir() {
        match &input_dir.to_str() {
            Some(v) => return Err(format!("{} is not a valid directory!", v)),
            None => return Err(format!("Please provide a valid directory!")),
        }
    }
    if let None = &input_dir.file_name() {
        return Err(String::from(
            "The last segment of the input path is not valid!",
        ));
    };
    // get input directory name
    let input_dir_name = input_dir.file_name().unwrap();
    // get parent directiory
    let parent = match input_dir.parent() {
        Some(p) => p,
        None => Path::new(""),
    };
    // generate output directory
    Ok(parent
        .join(format!("{}-html5picture", input_dir_name.to_str().unwrap())))
}

/// Removes the given base directory from the given input file.
/// ## Example
///
/// ```
/// use {
///     html5_picture::path::get_output_working_dir,
///     std::path::PathBuf,
/// };
///
/// let input = PathBuf::from("../assets");
/// let input = html5_picture::path::get_output_working_dir(&input).unwrap();
/// assert_eq!(input.to_str().unwrap(), "../assets-html5picture");
/// ```
pub fn remove_base_dir(
    base_dir: &PathBuf,
    input_file: &PathBuf,
) -> Result<PathBuf, String> {
    match pathdiff::diff_paths(input_file, base_dir) {
        Some(p) => Ok(p),
        None => match input_file.to_str() {
            Some(v) => {
                Err(String::from(format!("Could not remove base dir of {}", v)))
            }
            None => Err(String::from("Could not remove base dir!")),
        },
    }
}
