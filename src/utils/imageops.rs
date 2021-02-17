use std::path::PathBuf;

/// Resizes the image preserving the aspect ratio. Returns the new height.
pub fn calculate_height_preserve_aspect_ratio(
    image_file_name: &PathBuf,
    width: u32,
) -> Result<u32, String> {
    // get image dimensions
    let (w, h) = match image::image_dimensions(&image_file_name) {
        Err(msg) => return Err(msg.to_string()),
        Ok((w, h)) => (w, h),
    };
    let scale_factor = width as f64 / w as f64;
    Ok((scale_factor * h as f64) as u32)
}
