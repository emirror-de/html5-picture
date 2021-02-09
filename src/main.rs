#[warn(missing_docs)]
use {
    clap::Clap,
    html5_picture::{webp, ImageProcessor},
    log::{error, warn},
    std::path::PathBuf,
    walkdir::WalkDir,
};

const DEFAULT_QUALITY_WEBP: u8 = 70;

/// Scales the input images (currently png only) to the given breakpoints and
/// converts them to webp format.
/// Depends on cwebp, so make sure webp is installed on your pc!
/// Currently passes -q 100 to cwebp.
#[derive(Clap, Debug)]
#[clap(version = "0.0.3-alpha", author = "Lewin Probst <info@emirror.de>")]
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

fn check_arguments(config: &mut Args) {
    // set default quality
    if let None = &config.quality_webp {
        config.quality_webp = Some(DEFAULT_QUALITY_WEBP);
    } else {
        // only a quality between 1 and 100 is available for webp
        config.quality_webp = if config.quality_webp.unwrap() > 100 {
            Some(100)
        } else if config.quality_webp.unwrap() < 1 {
            Some(1)
        } else {
            Some(DEFAULT_QUALITY_WEBP)
        };
    }
}

fn main() {
    pretty_env_logger::init();

    // parse and check arguments for validity
    let mut config: Args = Args::parse();
    check_arguments(&mut config);
    let config = config;

    // get overall output directory
    let output_base_dir =
        match html5_picture::get_output_dir_name(&config.input_dir) {
            Ok(o) => o,
            Err(msg) => {
                error!("{}", msg);
                return;
            }
        };
    // create output directory for converted files
    if let Err(msg) = html5_picture::create_output_dir(&config.input_dir) {
        error!("{}", msg);
        return;
    }

    // Instantiate converter adapter
    let webp_converter = webp::WebpConverterAdapter {
        quality: config.quality_webp.unwrap(),
    };

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
            warn!(
                "Skipping {} because it is not a .png file!",
                &entry.to_str().unwrap()
            );
            continue;
        }

        // get resulting output path name
        let f = match html5_picture::remove_base_dir(&config.input_dir, &entry)
        {
            Ok(s) => s,
            Err(msg) => {
                error!("{}", msg);
                return;
            }
        };
        let resulting_output_path = output_base_dir.join(f);

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

        if let Some(v) = &config.scaled_images_count {
            // resize and convert the png according to the given image count
            let p = ImageProcessor::new(
                entry.clone(),
                resulting_output_path.clone(),
                *v,
            );
            let resized_image_details = p.get_resized_image_details();
            match resized_image_details {
                Ok(v) => {
                    p.batch_convert(&webp_converter, v);
                }
                Err(msg) => error!("{}", msg),
            };
        };

        // convert full scale in any case
        webp_converter.from_png(&entry, &resulting_output_path, None);
        return;
    }
}
