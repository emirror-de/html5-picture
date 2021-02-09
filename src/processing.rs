use {crate::webp::WebpConverterAdapter, math, std::path::PathBuf};

/// Contains the determined image details required for conversion.
#[derive(Debug)]
pub struct ResizedImageDetails {
    pub output_file_name: PathBuf,
    pub width: u32,
    pub height: u32,
}

impl ResizedImageDetails {
    pub fn new(output_file_name: PathBuf, width: u32, height: u32) -> Self {
        Self {
            output_file_name,
            width,
            height,
        }
    }
}

/// Scales and converts the given image.
pub struct ImageProcessor {
    pub input: PathBuf,
    pub output: PathBuf,
    pub scaled_images_count: u8,
}

impl ImageProcessor {
    pub fn new(
        input: PathBuf,
        output: PathBuf,
        scaled_images_count: u8,
    ) -> Self {
        ImageProcessor {
            input,
            output,
            scaled_images_count,
        }
    }

    pub fn get_resized_image_details(
        &self,
    ) -> Result<Vec<ResizedImageDetails>, String> {
        // get image dimensions
        let (w, _) = match image::image_dimensions(&self.input) {
            Err(msg) => return Err(msg.to_string()),
            Ok((w, h)) => (w, h),
        };
        // calculate a step in pixel that is used to calculate the new width
        let step = w as f32 / (self.scaled_images_count as f32 + 1.0);
        let mut resized_details = vec![];
        for idx in 0..self.scaled_images_count {
            let new_width = (idx + 1) as f32 * step;
            let new_width = math::round::ceil(new_width.into(), 0) as u32;
            let new_height =
                self.calculate_height_preserve_aspect_ratio(new_width)?;
            let output_file_name = self.get_resized_file_name(new_width)?;
            resized_details.push(ResizedImageDetails::new(
                output_file_name,
                new_width,
                new_height,
            ));
        }
        Ok(resized_details)
    }

    /*
    /// Loads the image from disk.
    fn load_image(&self) -> Result<DynamicImage, String> {
        let img = ImageReader::open(&self.input);
        let img = if let Err(msg) = img {
            return Err(msg.to_string());
        } else {
            img.unwrap()
        };
        let img = match img.decode() {
            Ok(i) => i,
            Err(msg) => return Err(msg.to_string()),
        };
        Ok(img)
    }
    */

    /// Constructs the file name for a resized image.
    fn get_resized_file_name(&self, width: u32) -> Result<PathBuf, String> {
        let mut output = self.output.clone();
        let extension = match output.extension() {
            Some(e) => e.to_owned(),
            None => return Err("No extension found!".to_string()),
        };
        let file_name = match output.file_stem() {
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
        output.set_file_name(format!("{}-w{}", file_name, width));
        output.set_extension(extension);
        Ok(output)
    }

    /// Resizes the image preserving the aspect ratio. Returns the new height.
    pub fn calculate_height_preserve_aspect_ratio(
        &self,
        width: u32,
    ) -> Result<u32, String> {
        let (w, h) = match image::image_dimensions(&self.input) {
            Err(msg) => return Err(msg.to_string()),
            Ok((w, h)) => (w, h),
        };
        let scale_factor = width as f64 / w as f64;
        println!("{}", &scale_factor);
        Ok((scale_factor * h as f64) as u32)
    }

    /// Converts the source image (stored in ```self.input``` on instantiation)
    /// to the given details. Additional flags are passed using the webp_adapter.
    pub fn batch_convert(
        &self,
        webp_adapter: &WebpConverterAdapter,
        resized_image_details: Vec<ResizedImageDetails>,
    ) {
        for details in resized_image_details {
            webp_adapter.from_png(
                &self.input,
                &details.output_file_name,
                Some(vec![
                    "-resize".to_string(),
                    details.width.to_string(),
                    details.height.to_string(),
                ]),
            );
        }
    }
}
