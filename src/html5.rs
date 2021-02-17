use {
    crate::utils::ResizedImageDetails,
    serde::{Deserialize, Serialize},
    std::path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub enum MediaWidth {
    Max(String),
    Min(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SourceAttributes {
    pub media_width: MediaWidth,
    pub srcset: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Picture {
    sources: Vec<SourceAttributes>,
    srcset_prefix: Option<String>,
    fallback_uri: String,
    alt_text: String,
}

impl Picture {
    pub fn from(
        image_file_name: &PathBuf,
        srcset_prefix: Option<String>,
        scaled_images_count: u8,
        alt_text: Option<String>,
    ) -> Result<Self, String> {
        if scaled_images_count == 0 {
            return Err("scaled_images_count must be > 0".to_string());
        }
        let resized_image_details =
            ResizedImageDetails::from(&image_file_name, scaled_images_count)?;
        let mut sources = vec![];
        for details in &resized_image_details {
            let out_file_name = match details.output_file_name.file_name() {
                Some(v) => v,
                None => {
                    return Err(String::from("Could not get output_file_name!"))
                }
            };
            let out_file_name = match out_file_name.to_str() {
                Some(v) => String::from(v),
                None => {
                    return Err(String::from(
                        "Could not convert output_file_name!",
                    ))
                }
            };
            sources.push(SourceAttributes {
                media_width: MediaWidth::Max(details.width.to_string()),
                srcset: out_file_name,
            });
        }
        // append full scale image
        let mut full_scale_image = match image_file_name.file_name() {
            Some(v) => PathBuf::from(v),
            None => {
                return Err(String::from("Could not get output_file_name!"))
            }
        };
        //let mut full_scale_image = image_file_name.file_name().clone();
        full_scale_image.set_extension("webp");
        let full_scale_image = match full_scale_image.to_str() {
            Some(v) => String::from(v),
            None => {
                return Err(String::from(
                    "Could not convert full_scale_image file name!",
                ))
            }
        };
        sources.push(SourceAttributes {
            media_width: MediaWidth::Min(
                // unwrap allowed as long as scaled_images_count > 0
                (resized_image_details.last().unwrap().width + 1).to_string(),
            ),
            srcset: full_scale_image,
        });

        Ok(Self {
            sources,
            srcset_prefix,
            fallback_uri: image_file_name.to_str().unwrap().to_string(),
            alt_text: alt_text.unwrap_or_default(),
        })
    }

    pub fn to_html_string(&self) -> String {
        let mut html = String::from("<picture>");
        let uri_prefix = match &self.srcset_prefix {
            Some(v) => format!("{}/", v),
            None => String::new(),
        };
        for src_attrs in &self.sources {
            let (min_max, value) = match &src_attrs.media_width {
                MediaWidth::Max(v) => ("max", v),
                MediaWidth::Min(v) => ("min", v),
            };
            html.push_str(&format!(
                "<source media=\"({}-width: {}px)\" srcset=\"{}{}\">",
                min_max, value, &uri_prefix, src_attrs.srcset
            ));
        }
        // add fallback image
        html.push_str(&format!(
            "<img src=\"{}{}\" alt=\"{}\" />",
            uri_prefix, self.fallback_uri, self.alt_text
        ));
        html.push_str("</picture>");
        html
    }
}
