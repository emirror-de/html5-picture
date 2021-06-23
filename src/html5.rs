use {
    crate::{core::Config, utils::ResizedImageDetails},
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, path::PathBuf},
};

type PathBufPictureRegister = HashMap<PathBuf, Picture>;

/// Contains information about the MediaWidth property of a ```<picture>``` that
/// are required for its creation.
#[derive(Serialize, Deserialize, Debug)]
pub enum MediaWidth {
    Max(String),
    Min(String),
}

/// Information about the source attributes in a ```<picture>``` tag.
#[derive(Serialize, Deserialize, Debug)]
pub struct SourceAttributes {
    pub media_width: MediaWidth,
    pub srcset: String,
}

/// Represents the HTML5 ```<picture>``` tag.
#[derive(Serialize, Deserialize, Debug)]
pub struct Picture {
    /// Contains the <source> tags of the picture.
    pub sources: Vec<SourceAttributes>,
    /// Specifies the fallback uri of the picture.
    pub fallback_uri: String,
}

impl Picture {
    /// Collects all information about the image required for the creation of a
    /// ```<picture>``` tag.
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

    /// Creates a string that contains the full ```<picture>``` tag. It can
    /// directly be embedded into a webpage.
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

/// Contains a HashMap with all required details of the images to create an
/// according picture tag.
/// The image details are loaded into memory to be able to retrieve them as fast
/// as possible.
/// The install_images_into parameter is used to determine which images can be
/// used.
#[derive(Debug)]
pub struct PictureRegister {
    config: Config,
    register: PathBufPictureRegister,
}

impl PictureRegister {
    /// Creates a new instance from the given config.
    pub fn from(config: &Config) -> Result<Self, String> {
        match &config.install_images_into {
            None => {
                return Err(
                    "The install_images_into parameter needs to be set!"
                        .to_string(),
                )
            }
            Some(v) => {
                if !v.is_dir() {
                    return Err("The install_images_into parameter is not a valid directory".to_string());
                }
            }
        }

        Ok(Self {
            config: config.clone(),
            register: Self::create_register(&config)?,
        })
    }

    /// Creates the register from the given config.
    fn create_register(
        config: &Config,
    ) -> Result<PathBufPictureRegister, String> {
        let images_path = match &config.install_images_into {
            None => {
                return Err(
                    "The install_images_into parameter needs to be set!"
                        .to_string(),
                )
            }
            Some(v) => {
                if !v.is_dir() {
                    return Err("The install_images_into parameter is not a valid directory".to_string());
                }
                v
            }
        };

        let mut register = PathBufPictureRegister::new();
        let png_file_names = crate::collect_png_file_names(&images_path, None);
        for png in png_file_names {
            let pic = Picture::from(&png, config.scaled_images_count)?;
            register.insert(png, pic);
        }
        Ok(register)
    }

    /// Returns a reference to the ```Picture``` instance of the given image.
    /// Please use the original filename of the picture that you want to use,
    /// for example:
    /// ```ignore
    /// let p = register_instance.get(&PathBuf::from("assets/image-1.png")).unwrap();
    /// ```
    pub fn get(&self, image: &PathBuf) -> Result<&Picture, String> {
        match self.register.get(image) {
            None => Err("Image not found!".to_string()),
            Some(v) => Ok(v),
        }
    }
}
