//! Contains the processors that execute the single and batch conversion.

use {crate::webp::WebpParameter, std::path::PathBuf};

mod batch;
mod single;

pub use {
    batch::{BatchParameter, BatchProcessor},
    single::SingleProcessor,
};

/// The parameter required for ```SingleProcessor``` to run.
#[derive(Clone, Debug)]
pub struct Parameter {
    pub webp_parameter: WebpParameter,
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub scaled_images_count: Option<u8>,
}
