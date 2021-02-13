/// The parameter that are passed to webp conversion.
#[derive(Clone, Debug)]
pub struct WebpParameter {
    pub quality: u8,
}

impl WebpParameter {
    /// Creates a new instance.
    pub fn new(quality: Option<u8>) -> Self {
        let quality = Self::parameter_check_quality(quality);
        Self { quality }
    }

    /// Checks the parameter for input mistakes.
    fn parameter_check_quality(quality: Option<u8>) -> u8 {
        // set default quality
        if let None = quality {
            super::DEFAULT_QUALITY
        } else {
            // only a quality between 1 and 100 is available for webp
            if quality.unwrap() > 100 {
                100
            } else if quality.unwrap() < 1 {
                1
            } else {
                super::DEFAULT_QUALITY
            }
        }
    }
}
