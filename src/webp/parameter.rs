#[derive(Clone, Debug)]
pub struct Parameter {
    pub quality: u8,
}

impl Parameter {
    pub fn new(quality: Option<u8>) -> Self {
        let quality = Self::parameter_check_quality(quality);
        Self { quality }
    }

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
