use {
    crate::{core::Config, utils::ResizedImageDetails},
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, path::PathBuf},
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
    fallback_uri: String,
}

impl Picture {
    pub fn from(
        image_file_name: &PathBuf,
        scaled_images_count: u8,
    ) -> Result<Self, String> {
        if scaled_images_count == 0 {
            return Err("scaled_images_count must be > 0".to_string());
        }
        let resized_image_details =
            ResizedImageDetails::from(&image_file_name, scaled_images_count)?;
        let mut sources = vec![];
        let mut input_dir = image_file_name.clone();
        input_dir.pop();
        for details in &resized_image_details {
            let out_file_name =
                match input_dir.join(&details.output_file_name).to_str() {
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
        let mut full_scale_image = image_file_name.clone();
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
            fallback_uri: image_file_name.to_str().unwrap().to_string(),
        })
    }

    pub fn to_html_string(
        &self,
        srcset_prefix: Option<String>,
        alt_text: &str,
    ) -> String {
        let mut html = String::from("<picture>");
        let uri_prefix = match &srcset_prefix {
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
            uri_prefix, self.fallback_uri, alt_text
        ));
        html.push_str("</picture>");
        html
    }
}
