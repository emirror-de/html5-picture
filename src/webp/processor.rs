use {crate::webp::WebpParameter, std::path::PathBuf};

mod batch;
mod single;
pub use batch::BatchParameter;
pub use batch::BatchProcessor;
pub use single::SingleProcessor;

#[derive(Clone, Debug)]
pub struct Parameter {
    pub webp_parameter: WebpParameter,
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub scaled_images_count: Option<u8>,
}

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
