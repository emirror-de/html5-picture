use std::path::PathBuf;
/// Contains the determined image details required for conversion.
#[derive(Debug)]
pub struct ResizedImageDetails {
    pub output_file_name: PathBuf,
    pub width: u32,
    pub height: u32,
}

impl ResizedImageDetails {
    /// Creates a new instance.
    pub fn new(output_file_name: PathBuf, width: u32, height: u32) -> Self {
        Self {
            output_file_name,
            width,
            height,
        }
    }

    /// Calculates height, width and output file names for the scaled images.
    pub fn from(
        image_file_name: &PathBuf,
        scaled_images_count: u8,
    ) -> Result<Vec<ResizedImageDetails>, String> {
        // get image dimensions
        let (w, _) = match image::image_dimensions(&image_file_name) {
            Err(msg) => return Err(msg.to_string()),
            Ok((w, h)) => (w, h),
        };
        // calculate a step in pixel that is used to calculate the new width
        let step = w as f32 / (scaled_images_count as f32 + 1.0);
        let mut resized_details = vec![];
        for idx in 0..scaled_images_count {
            let new_width = (idx + 1) as f32 * step;
            let new_width = math::round::ceil(new_width.into(), 0) as u32;
            let new_height =
                crate::utils::imageops::calculate_height_preserve_aspect_ratio(
                    &image_file_name,
                    new_width,
                )?;
            let output_file_name =
                Self::get_resized_file_name(&image_file_name, new_width)?;
            resized_details.push(ResizedImageDetails::new(
                output_file_name,
                new_width,
                new_height,
            ));
        }
        Ok(resized_details)
    }

    /// Constructs the file name for a resized image.
    pub fn get_resized_file_name(
        image_file_name: &PathBuf,
        width: u32,
    ) -> Result<PathBuf, String> {
        let file_name = match &image_file_name.file_stem() {
            Some(f) => f.to_owned(),
            None => return Err("No filename given!".to_string()),
        };
        let file_name = match file_name.to_str() {
            Some(f) => f,
            None => {
                return Err(
                    "utf-8 check failed for resized filename".to_string()
                )
            }
        };
        Ok(PathBuf::from(format!("{}-w{}.webp", file_name, width)))
    }
}
