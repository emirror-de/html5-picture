use {
    super::{Parameter, ResizedImageDetails},
    image::{io::Reader as ImageReader, DynamicImage},
    indicatif::ProgressBar,
    log::error,
    std::{fs::File, io::Write, path::PathBuf},
    webp::WebPMemory,
};

/// Resizes and converts the given input file to webp format. Every function
/// of This instance is single threaded. Multi threading support is provided
/// by the ```BatchProcessor``` struct.
pub struct SingleProcessor {
    params: Parameter,
    image: Option<DynamicImage>,
    progressbar: Option<ProgressBar>,
}

impl SingleProcessor {
    /// Creates a new instance of the struct.
    pub fn new(
        params: Parameter,
        progressbar: Option<ProgressBar>,
    ) -> Result<Self, String> {
        if !&params.input.is_file() {
            return Err(format!(
                "Given input ({}) is not a file!",
                &params.input.to_str().unwrap()
            ));
        }
        Ok(Self {
            params,
            image: None,
            progressbar,
        })
    }

    /// Loads the image from disk.
    fn load_image(&self) -> Result<DynamicImage, String> {
        let img = ImageReader::open(&self.params.input);
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

    /// Encodes the image stored internally to wepb.
    fn encode_image(&self, img: &DynamicImage) -> WebPMemory {
        let encoder = webp::Encoder::from_image(&img);
        // unwrap can safely be called here because it has been checked already
        // on instantiation of the parameter
        encoder.encode(self.params.webp_parameter.quality as f32)
    }

    /// Loads, resizes and converts the image to webp. Single threaded.
    pub fn run(&mut self) -> Result<(), String> {
        if let Some(ref pb) = &self.progressbar {
            pb.set_prefix(&self.params.input.to_str().unwrap());
            pb.set_message("Loading image...");
        }
        self.image = Some(self.load_image().unwrap());

        let output_file_name = match self.get_output_file_name() {
            Ok(v) => v,
            Err(msg) => return Err(msg.to_string()),
        };
        let output_file_name = &self.params.output_dir.join(output_file_name);
        if let Some(ref pb) = &self.progressbar {
            pb.set_message("Encoding...");
        }
        let encoded_img = self.encode_image(self.image.as_ref().unwrap());
        if let Some(ref pb) = &self.progressbar {
            pb.set_message("Saving...");
        }
        {
            let mut buf = File::create(output_file_name).unwrap();
            if let Err(msg) = buf.write_all(&encoded_img) {
                error!("{}", msg);
            };
        }
        if let Some(ref pb) = &self.progressbar {
            pb.set_message("...done!");
            pb.inc(1);
        }
        if let Some(_) = self.params.scaled_images_count {
            match self.get_resized_image_details() {
                Ok(v) => {
                    self.run_resize_images(v);
                }
                Err(msg) => {
                    if let Some(ref pb) = &self.progressbar {
                        pb.finish_with_message(&format!("Error: {}", &msg));
                    }
                    error!("{}", msg);
                    return Err(msg);
                }
            };
        }
        if let Some(ref pb) = &self.progressbar {
            //pb.finish_and_clear();
            pb.finish_with_message("Done! :-)");
        }
        Ok(())
    }

    /// Subroutine of ```run```, processes the resizing before conversion.
    fn run_resize_images(&self, details: Vec<ResizedImageDetails>) {
        for detail in details.iter().rev() {
            if let Some(ref pb) = &self.progressbar {
                pb.set_message(&format!(
                    "Resizing to {}x{} ...",
                    &detail.width, &detail.height
                ));
            }
            let img = self.image.as_ref().unwrap().resize(
                detail.width,
                detail.height,
                image::imageops::FilterType::Triangle,
            );
            let output_file_name =
                &self.params.output_dir.join(&detail.output_file_name);
            if let Some(ref pb) = &self.progressbar {
                pb.set_message("Encoding...");
            }
            let img = &self.encode_image(&img);
            if let Some(ref pb) = &self.progressbar {
                pb.set_message("Saving...");
            }
            let mut buf = File::create(output_file_name).unwrap();
            if let Err(msg) = buf.write_all(&img) {
                error!("{}", msg);
            };
            if let Some(ref pb) = &self.progressbar {
                pb.set_message("...done!");
                pb.inc(1);
            }
        }
    }

    /// Generates the output file name for resized images.
    fn get_output_file_name(&self) -> Result<PathBuf, String> {
        let file_name = self.params.input.file_stem();
        if let None = &file_name {
            return Err("File name could not be extracted!".to_string());
        }
        let file_name = file_name.unwrap();
        Ok(PathBuf::from(format!(
            "{}.webp",
            file_name.to_str().unwrap()
        )))
    }

    /// Constructs the file name for a resized image.
    fn get_resized_file_name(&self, width: u32) -> Result<PathBuf, String> {
        let file_name = match &self.params.input.file_stem() {
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

    /// Resizes the image preserving the aspect ratio. Returns the new height.
    pub fn calculate_height_preserve_aspect_ratio(
        &self,
        width: u32,
    ) -> Result<u32, String> {
        // get image dimensions
        let (w, h) = match image::image_dimensions(&self.params.input) {
            Err(msg) => return Err(msg.to_string()),
            Ok((w, h)) => (w, h),
        };
        let scale_factor = width as f64 / w as f64;
        Ok((scale_factor * h as f64) as u32)
    }

    /// Calculates height, width and output file name for all scaled images.
    pub fn get_resized_image_details(
        &self,
    ) -> Result<Vec<ResizedImageDetails>, String> {
        // get image dimensions
        let (w, _) = match image::image_dimensions(&self.params.input) {
            Err(msg) => return Err(msg.to_string()),
            Ok((w, h)) => (w, h),
        };
        // calculate a step in pixel that is used to calculate the new width
        let step =
            w as f32 / (self.params.scaled_images_count.unwrap() as f32 + 1.0);
        let mut resized_details = vec![];
        for idx in 0..self.params.scaled_images_count.unwrap() {
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
}
