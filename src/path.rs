use std::path::{Path, PathBuf};

/// Generates an output file name that is stored in the output working directory.
/// ## Example
///
/// ```
/// use {
///     html5_picture::path::create_output_file_name,
///     std::path::PathBuf,
/// };
///
/// let base_dir = PathBuf::from("../assets");
/// let input_file = PathBuf::from("../assets/some/picture.png");
/// let output_file = create_output_file_name(&base_dir, &input_file).unwrap();
/// assert_eq!(output_file.to_str().unwrap(), "../.assets-html5picture/some/picture.png");
/// ```
pub fn create_output_file_name(
    base_dir: &PathBuf,
    input_file: &PathBuf,
) -> Result<PathBuf, String> {
    let output_base_dir = get_output_working_dir(&base_dir)?;
    let relative_file_name = remove_base_dir(&base_dir, &input_file)?;
    Ok(output_base_dir.join(relative_file_name))
}

/// Calculates the relative filename according to `base_dir` and `input_file`
/// and joins it with the `output_dir`.
/// ## Example
///
/// ```
/// use {
///     html5_picture::path::create_output_file_name_with_output_dir,
///     std::path::PathBuf,
/// };
///
/// let output_dir = PathBuf::from("../new-assets");
/// let base_dir = PathBuf::from("../assets");
/// let input_file = PathBuf::from("../assets/some/picture.png");
/// let output_file = create_output_file_name_with_output_dir(&output_dir, &base_dir, &input_file).unwrap();
/// assert_eq!(output_file.to_str().unwrap(), "../new-assets/some/picture.png");
/// ```
pub fn create_output_file_name_with_output_dir(
    output_dir: &PathBuf,
    base_dir: &PathBuf,
    input_file: &PathBuf,
) -> Result<PathBuf, String> {
    let relative_file_name = remove_base_dir(&base_dir, &input_file)?;
    Ok(output_dir.join(relative_file_name))
}

/// Generates the output directory name by attaching a fixed string to it.
/// ## Example
///
/// ```
/// use {
///     html5_picture::path::get_output_working_dir,
///     std::path::PathBuf,
/// };
///
/// let input = PathBuf::from("../assets");
/// let input = get_output_working_dir(&input).unwrap();
/// assert_eq!(input.to_str().unwrap(), "../.assets-html5picture");
/// ```
pub fn get_output_working_dir(input_dir: &PathBuf) -> Result<PathBuf, String> {
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
    Ok(parent.join(format!(
        ".{}-html5picture",
        input_dir_name.to_str().unwrap()
    )))
}

/// Removes the given base directory from the given input file.
/// ## Example
///
/// ```
/// use {
///     html5_picture::path::remove_base_dir,
///     std::path::PathBuf,
/// };
///
/// let base_dir = PathBuf::from("../assets");
/// let input_file = PathBuf::from("../assets/some_other/directory/picture.png");
/// let output = remove_base_dir(&base_dir, &input_file).unwrap();
/// assert_eq!(output.to_str().unwrap(), "some_other/directory/picture.png");
/// ```
pub fn remove_base_dir(
    base_dir: &PathBuf,
    input_file: &PathBuf,
) -> Result<PathBuf, String> {
    match input_file.strip_prefix(base_dir) {
        Ok(relative_path) => Ok(relative_path.to_path_buf()),
        Err(_) => match input_file.to_str() {
            Some(v) => {
                Err(String::from(format!("Could not remove base dir of {}", v)))
            }
            None => Err(String::from("Could not remove base dir!")),
        },
    }
}
