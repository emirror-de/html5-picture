use {
    log::{debug, error, info},
    std::fs::File,
    std::io::Write,
    image::{DynamicImage, io::Reader as ImageReader},
    std::{path::{PathBuf}, process::Stdio},
    //tokio::{
    //    fs::{self, File},
    //    io::{self, AsyncWriteExt, AsyncReadExt},
    //},
    walkdir::WalkDir,
    webp::WebPMemory,
};

const DEFAULT_QUALITY_WEBP: u8 = 70;
///
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

#[derive(Clone)]
pub struct WebpParameter {
    pub quality: u8,
}

impl WebpParameter {
    pub fn new(quality: Option<u8>) -> Self {
        let quality = Self::parameter_check_quality(quality);
        Self {
            quality,
        }
    }

    fn parameter_check_quality(quality: Option<u8>) -> u8 {
        // set default quality
        if let None = quality {
            DEFAULT_QUALITY_WEBP
        } else {
            // only a quality between 1 and 100 is available for webp
            if quality.unwrap() > 100 {
                100
            } else if quality.unwrap() < 1 {
                1
            } else {
                DEFAULT_QUALITY_WEBP
            }
        }
    }
}

#[derive(Clone)]
pub struct ProcessorParameter {
    pub webp_parameter: WebpParameter,
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub scaled_images_count: Option<u8>,
}

pub struct BatchProcessor {
    params: ProcessorParameter,
}

impl BatchProcessor {
    pub fn new(params: ProcessorParameter) -> Self {
        Self { params }
    }

    pub async fn run(&self) {
        // process all images
        for entry in WalkDir::new(&self.params.input) {
            // unwrap the entry
            let entry = if let Err(msg) = &entry {
                error!("{}", msg.to_string());
                continue;
            } else {
                entry.unwrap()
            };

            let entry = entry.into_path();

            if !crate::is_png(&entry) {
                continue;
            }

            let mut params_single = self.params.clone();
            params_single.input = entry;
            tokio::spawn(async move {
                let mut webp_processor = SingleProcessor::new(params_single).unwrap();
                println!("{:#?}", webp_processor.run().await);
            });
        }
    }
}

pub struct SingleProcessor {
    params: ProcessorParameter,
    image: Option<DynamicImage>,
}

impl SingleProcessor {
    pub fn new(params: ProcessorParameter) -> Result<Self, String> {
        if !&params.input.is_file() {
            return Err(format!("Given input ({}) is not a file!", &params.input.to_str().unwrap()));
        }
        Ok(Self { params, image: None } )
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

        //let mut input_image = File::open(&self.params.input).await?;
        //let mut input_image_buffer = vec![];
        //input_image.read_to_end(&mut input_image_buffer).await?;
    pub async fn run(&mut self) -> Result<(), String> {
        debug!("Loading image {:#?} from disk...", &self.params.input);
        self.image = Some(self.load_image().unwrap());
        debug!("...done!");

        self.create_output_dir_if_required().await?;
        let output_file_name = match self.get_output_file_name() {
            Ok(v) => v,
            Err(msg) => return Err(msg.to_string()),
        };
        let output_file_name = &self.params.output_dir.join(output_file_name);
        debug!("Saving {}...", &output_file_name.to_str().unwrap());
        {
            debug!("Encoding image...");
            let encoded_img = self.encode_image(self.image.as_ref().unwrap());
            debug!("...done!");
            let mut buf = File::create(output_file_name).unwrap();
            buf.write_all(&encoded_img);
        }
        debug!("...done!");
        info!("Saved {}", &output_file_name.to_str().unwrap());
        debug!("Processing scaled images...");
        if let Some(v) = self.params.scaled_images_count {
            match self.get_resized_image_details() {
                Ok(v) => {
                    self.run_resize_images(v).await;
                }
                Err(msg) => error!("{}", msg),
            };
            info!("Saved {}", &output_file_name.to_str().unwrap());
        }
        debug!("...done!");
        Ok(())
    }

    async fn run_resize_images(&self, details: Vec<ResizedImageDetails>) {
        for detail in details.iter().rev() {
            let img = self.image.as_ref().unwrap().resize(
                detail.width,
                detail.height,
                image::imageops::FilterType::Triangle
                );
            let output_file_name = &self.params.output_dir.join(&detail.output_file_name);
            debug!("Encoding image {}...", &output_file_name.to_str().unwrap());
            let img = &self.encode_image(&img);
            debug!("...done!");
            debug!("Saving {}...", &output_file_name.to_str().unwrap());
            let mut buf = File::create(output_file_name).unwrap();
            buf.write_all(&img);
            debug!("...done!");
            info!("Saved {}", &output_file_name.to_str().unwrap());
        }
    }

    fn get_output_file_name(&self) -> Result<PathBuf, String> {
        let file_name = self.params.input.file_stem();
        if let None = &file_name {
            return Err("File name could not be extracted!".to_string());
        }
        let file_name = file_name.unwrap();
        Ok(PathBuf::from(format!("{}.webp", file_name.to_str().unwrap())))
    }

    async fn create_output_dir_if_required(&self) -> Result<(), String> {
        debug!(
            "Create output directory ({}) if not existent...",
            &self.params.output_dir.to_str().unwrap()
            );
        if !self.params.output_dir.exists() {
            match tokio::fs::create_dir_all(&self.params.output_dir).await {
                Ok(()) => (),
                Err(msg) => {
                    match self.params.output_dir.to_str() {
                        Some(v) => {
                            return Err(format!(
                                    "Could not create folder {}! Error: {}",
                                    v,
                                    msg
                                    ))
                        },
                        None => {
                            return Err(
                                format!("Could not create folder! Error {}", msg)
                                );
                        }
                    };
                }
            };
        }
        debug!("...done!");
        Ok(())
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
        let step = w as f32 / (self.params.scaled_images_count.unwrap() as f32 + 1.0);
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

/*
pub struct WebpConverterAdapter {
    pub quality: u8,
}

impl WebpConverterAdapter {
    fn parameter_check(
        input_file_name: &PathBuf,
        output_file_name: &PathBuf,
    ) -> Result<(), String> {
        if !&input_file_name.is_file() {
            return match &input_file_name.to_str() {
                Some(v) => Err(format!("{} is not a file!", v)),
                None => Err("Input is not a file!".to_string()),
            };
        };
        if output_file_name.is_dir() {
            return match &output_file_name.to_str() {
                Some(v) => Err(format!("{} is a directory!", v)),
                None => Err("Output file is a directory!".to_string()),
            };
        };
        Ok(())
    }

    /// Converts the given input file to webp using ```cwebp``` and saves it to
    /// the given output file.
    ///
    /// ## Errors and panics
    /// Returns an error if one or more parameters are invalid.
    /// Panics if the call to ```cwebp``` fails.
    ///
    /// ## Returns
    /// Returns the stdout output from cwebp.
    pub fn from_png(
        &self,
        input_file_name: &PathBuf,
        output_file_name: &PathBuf,
        additional_cwebp_arguments: Option<Vec<String>>,
    ) -> Result<String, String> {
        if let Err(msg) =
            Self::parameter_check(&input_file_name, &output_file_name)
        {
            return Err(msg.to_string());
        };
        let o = &mut output_file_name.clone();
        o.set_extension("webp");

        let mut args = vec![
            input_file_name.to_str().unwrap().to_string(),
            "-q".to_string(),
            self.quality.to_string(),
            "-o".to_string(),
            o.to_str().unwrap().to_string(),
        ];
        if let Some(v) = additional_cwebp_arguments {
            args.append(&mut v.clone());
        };
        let process = match std::process::Command::new("cwebp")
            .args(&args)
            .stdout(Stdio::null())
            .spawn()
        {
            Ok(process) => process,
            Err(err) => panic!("Running process error: {}", err),
        };

        let output = match process.wait_with_output() {
            Ok(output) => output,
            Err(err) => panic!("Retrieving output error: {}", err),
        };

        match std::string::String::from_utf8(output.stdout) {
            Ok(stdout) => Ok(stdout),
            Err(err) => panic!("Translating output error: {}", err),
        }
    }

    pub async fn convert_from_png(
        &self,
        input_file_name: &PathBuf,
        output_file_name: &PathBuf,
    ) -> Result<(), String> {
        if let Err(msg) =
            Self::parameter_check(&input_file_name, &output_file_name)
        {
            return Err(msg.to_string());
        };
        let o = &mut output_file_name.clone();
        o.set_extension("webp");

        // load the input image
        let encoded_img = {
            let img = image::io::Reader::open(
                input_file_name.to_str().unwrap().to_string(),
            )
            .unwrap()
            .decode()
            .unwrap();
            let encoder = webp::Encoder::from_image(&img);
            encoder.encode(70.0)
        };
        {
            let mut parent_dir = o.clone();
            parent_dir.pop();
        std::fs::create_dir_all(parent_dir);
        let mut buf = File::create(o).unwrap();
        buf.write_all(&encoded_img);
        }

        Ok(())
    }
}
*/
