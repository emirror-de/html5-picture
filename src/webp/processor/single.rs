use {
    super::Parameter,
    crate::utils::ResizedImageDetails,
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
        match ResizedImageDetails::from(
            &self.params.input,
            self.params.scaled_images_count,
        ) {
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
        if let Some(ref pb) = &self.progressbar {
            //pb.finish_and_clear();
            pb.finish_with_message("Done!");
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
}
