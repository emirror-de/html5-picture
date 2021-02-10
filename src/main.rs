#[warn(missing_docs)]
use {
    clap::Clap,
    html5_picture::{webp},
    log::{error, warn, debug},
    std::{path::PathBuf, sync::Arc},
};

const DEFAULT_QUALITY_WEBP: u8 = 70;

/// Scales the input images (currently png only) to the given breakpoints and
/// converts them to webp format.
/// Depends on cwebp, so make sure webp is installed on your pc!
/// Currently passes -q 100 to cwebp.
#[derive(Clap, Debug)]
#[clap(version = "0.0.4-alpha", author = "Lewin Probst <info@emirror.de>")]
struct Args {
    /// The directory containing all images that should be processed.
    pub input_dir: PathBuf,
    /// Count of scaled images to be calculated.
    #[clap(short)]
    pub scaled_images_count: Option<u8>,
    /// Defines the quality of cwebp conversion.
    #[clap(short)]
    pub quality_webp: Option<u8>,
    /*
    /// Disable conversion to webp. (Not implemented yet)
    #[clap(short)]
    pub disable_webp: bool,
    */
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    pretty_env_logger::init();

    debug!("Parsing arguments...");
    // parse and check arguments for validity
    let config: Args = Args::parse();
    debug!("...done! Arguments:\n{:#?}", &config);

    debug!("Calculate output working directory...");
    // get overall output directory
    let output_working_dir =
        match html5_picture::path::get_output_working_dir(&config.input_dir) {
            Ok(o) => o,
            Err(msg) => {
                error!("{}", msg);
                return;
            }
        };
    debug!("...done! Output working directory:\n{:#?}", &output_working_dir);

    debug!("Initializing processor...");
    let webp_params = webp::WebpParameter::new(
        config.quality_webp,
    );
    //let test_image = PathBuf::from("circles/original/education.png");
    //let test_output_dir = PathBuf::from("circles/original");
    //let params = webp::ProcessorParameter {
    //    webp_parameter: webp_params,
    //    input: config.input_dir.join(&test_image),
    //    output_dir: output_working_dir.join(test_output_dir),
    //    scaled_images_count: config.scaled_images_count,
    //};
    //let mut webp_processor = webp::SingleProcessor::new(params).unwrap();
    let params = webp::ProcessorParameter {
        webp_parameter: webp_params,
        input: config.input_dir,
        output_dir: output_working_dir,
        scaled_images_count: config.scaled_images_count,
    };
    let mut webp_processor = webp::BatchProcessor::new(params);
    webp_processor.run().await;
    debug!("...done!");
    return;
    //let webp_batch_processor = webp::BatchProcessor::new(params);

    /*
    // Instantiate converter adapter
    let webp_converter = Arc::new(webp::WebpConverterAdapter {
        quality: config.quality_webp.unwrap(),
    });

    // process all images
    for entry in WalkDir::new(&config.input_dir) {
        // unwrap the entry
        let entry = if let Err(msg) = &entry {
            error!("{}", msg.to_string());
            continue;
        } else {
            entry.unwrap()
        };

        let entry = entry.into_path();

        if !html5_picture::is_png(&entry) {
            continue;
        }

        // get resulting output path name
        let f = match html5_picture::path::remove_base_dir(&config.input_dir, &entry)
        {
            Ok(s) => s,
            Err(msg) => {
                error!("{}", msg);
                return;
            }
        };
        let resulting_output_path = output_working_dir.join(f);

        // create the output directory if the entry is one
        if entry.is_dir() && !resulting_output_path.exists() {
            match std::fs::create_dir_all(&resulting_output_path) {
                Ok(()) => (),
                Err(msg) => {
                    match resulting_output_path.to_str() {
                        Some(v) => error!(
                            "Could not create folder {}! Error: {}",
                            v, msg
                        ),
                        None => {
                            error!("Could not create folder! Error {}", msg)
                        }
                    };
                    return;
                }
            };
        }

        // resize and convert the png according to the given image count
        if let Some(v) = &config.scaled_images_count {
            let p = Arc::new(ImageProcessor::new(
                entry.clone(),
                resulting_output_path.clone(),
                *v,
            ));
            let resized_image_details = p.get_resized_image_details();
            match resized_image_details {
                Ok(v) => {
                    let c = webp_converter.clone();
                    let p_clone = p.clone();
                    tokio::spawn(async move {
                        p_clone.batch_convert(&c, v).await;
                    });
                }
                Err(msg) => error!("{}", msg),
            };
        };

        // convert full scale in any case
        match webp_converter.from_png(&entry, &resulting_output_path, None) {
            Err(msg) => error!("{}", msg),
            Ok(_) => (),
        }
        return;
    }
    */
}
