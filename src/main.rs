#[warn(missing_docs)]

use {
    clap::Clap,
    log::error,
    walkdir::WalkDir,
    std::path::PathBuf,
};

/// html5-picture
/// Scales the input images to the given breakpoints and converts them to webp
/// format.
#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Lewin Probst <info@emirror.de>")]
struct Args {
    /// The directory containing all images that should be processed.
    pub input_dir: String,
    /// Disable conversion to webp. (Not implemented yet)
    #[clap(short)]
    pub disable_webp: bool,
}

fn main() {
    pretty_env_logger::init();

    let config: Args = Args::parse();

    // check if input dir is relative
    if PathBuf::from(&config.input_dir).is_absolute() {
        error!("Only relative inputs are allowed!");
        return;
    }
    // create output directory for converted files
    if let Err(msg) = html5_picture::create_output_dir(&config.input_dir) {
        error!("{}", msg);
        return;
    }

    // process all images
    for entry in WalkDir::new(&config.input_dir) {
        // unwrap the entry
        let entry = if let Err(msg) = &entry {
            error!("{}", msg.to_string());
            continue;
        } else {
            entry.unwrap()
        };

        let entry_path = entry.clone().into_path();
        // create the output directory if the entry is one
        if entry_path.is_dir() {
                let out = &format!(
                    "{}/{}",
                    &html5_picture::get_output_dir_name(&config.input_dir).unwrap().to_str().unwrap(),
                    &entry_path.to_str().unwrap()
                    );
                match std::fs::create_dir_all(out) { _ => (), };
        }
        // check if entry is a png file
        if html5_picture::is_png(&entry) {
            let input = &entry.clone().into_path();
            html5_picture::webp::from_png(
                &PathBuf::from(input),
                &html5_picture::get_output_dir_name(&config.input_dir).unwrap()
                );
        }
    }

}
