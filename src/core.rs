use {
    crate::{
        path,
        utils,
        webp::processor::BatchParameter,
        webp::processor::Parameter as ProcessorParameter,
        webp::WebpParameter,
    },
    clap::Clap,
    fs_extra::dir::{move_dir_with_progress, CopyOptions, TransitProcess},
    indicatif::MultiProgress,
    log::error,
    queue::Queue,
    std::{path::PathBuf, sync::Arc},
};

type Step = fn(&mut State);

/// Converts the images (currently png only) of the input folder to webp format.
/// It also has the ability to create multiple versions of the input images
/// having different sizes. See -s for further details.
/// Additionally it automatically generates HTML5 <picture> tag files for you
/// to be able to integrate them in a webpage easily.
///
/// Depends on cwebp, so make sure webp is installed on your pc!
#[derive(Clap, Debug)]
#[clap(
    version = "0.0.5-alpha",
    author = "Lewin Probst <info@emirror.de>, Michael Flau <michael@flau.net>"
)]
pub struct Config {
    /// The directory containing all images that should be processed.
    pub input_dir: PathBuf,
    /// The source image width is divided by this option (value + 1). Afterwards
    /// the source image is scaled (keeping the aspect ratio) to these widths
    /// before convertion.
    /// Useful if you want to have multiple sizes of the image on the webpage
    /// for different breakpoints.
    /// Example:
    /// Input image dimensions: 6000x962
    /// Scaled images count: 3
    /// Resulting converted images:
    ///
    ///     [filename]               [dimensions]
    ///     original_filename        6000x962
    ///     original_filename-w4500  4500x751
    ///     original_filename-w3000  3000x501
    ///     original_filename-w1500  1500x250
    #[clap(short)]
    pub scaled_images_count: Option<u8>,
    /// Installs the converted and sized pictures into the given folder.
    #[clap(short)]
    pub install_images_into: Option<PathBuf>,
    /// If true, the generation of HTML5 picture tag files is skipped.
    #[clap(short)]
    pub skip_html5_picture_tags: bool,
    /// If true, existing files are overwritten if install_images_into is set.
    #[clap(short)]
    pub force_overwrite: bool,
    /// Defines the quality of cwebp conversion.
    #[clap(short)]
    pub quality_webp: Option<u8>,
}

/// Contains the application state and config.
pub struct State {
    pub config: Config,
    pub file_names_to_convert: Vec<PathBuf>,
    pub current_step: usize,
    pub max_progress_steps: usize,
}

impl State {
    /// Creates a new instance of the application state.
    pub fn new(config: Config, max_progress_steps: usize) -> Self {
        Self {
            config,
            file_names_to_convert: vec![],
            current_step: 0,
            max_progress_steps,
        }
    }

    /// Small wrapper around the original dequeue function that automatically
    /// calculates the current application step.
    pub fn dequeue(&mut self, queue: &mut Queue<Step>) -> Option<Step> {
        self.current_step = self.max_progress_steps + 1 - queue.len();
        queue.dequeue()
    }

    /// Returns the prefix that is used in the ProgressBars.
    pub fn get_prefix(&self) -> String {
        format!("{}/{}", self.current_step, self.max_progress_steps)
    }
}

/// Collects all png files in the given input folder.
pub fn collect_file_names(state: &mut State) {
    let pb = utils::create_spinner();
    pb.set_prefix(&state.get_prefix());
    pb.set_message("Collecting files to convert...");
    state.file_names_to_convert = crate::collect_png_file_names(
        &state.config.input_dir, //&config.input_dir,
        Some(pb.clone()),
    );
    pb.finish_with_message(&format!(
        "Collected {} files!",
        &state.file_names_to_convert.len(), //&file_names_to_convert.len()
    ));
}

/// Recreates the folder structure of the input directory in the output directory.
pub fn create_all_output_directories(state: &mut State) {
    let pb = utils::create_spinner();
    pb.set_prefix(&state.get_prefix());
    pb.set_message("Create all output directories...");
    crate::fs::create_output_directories(
        &state.config.input_dir,      //&config.input_dir,
        &state.file_names_to_convert, //&file_names_to_convert,
        Some(pb.clone()),
    );
    pb.finish_with_message("Created all output directories!");
}

/// Resizes and converts all input images.
pub fn process_images(state: &mut State) {
    let webp_params = WebpParameter::new(state.config.quality_webp); //config.quality_webp);
    let params = ProcessorParameter {
        webp_parameter: webp_params,
        input: state.config.input_dir.clone(), //input: config.input_dir,
        output_dir: PathBuf::new(),
        scaled_images_count: state.config.scaled_images_count, //scaled_images_count: config.scaled_images_count,
    };
    let batch_params = BatchParameter {
        single_params: params,
    };
    let mp = Arc::new(MultiProgress::new());
    let batch_processor = crate::webp::processor::BatchProcessor::new(
        batch_params,
        Some(Arc::clone(&mp)),
    );
    let pb = utils::create_spinner();
    pb.set_prefix(&state.get_prefix());
    pb.set_message("Converting files...");
    batch_processor.run(&state.file_names_to_convert);
    pb.finish_with_message("Finished :-)");
}

/// Installs all images that have been converted to the given install folder.
pub fn install_images_into(state: &mut State) {
    let pb = utils::create_progressbar(0);
    match &state.config.install_images_into {
        None => return,
        Some(p) => {
            if !p.is_dir() {
                if let Err(msg) = std::fs::create_dir_all(p) {
                    pb.abandon_with_message(&format!(
                        "Could not create folder: {}",
                        msg.to_string()
                    ));
                }
            }
        }
    }
    pb.set_prefix(&state.get_prefix());
    let install_path =
        state.config.install_images_into.as_ref().unwrap().to_str();
    let install_string = match install_path {
        Some(s) => s,
        None => {
            pb.abandon_with_message("Invalid install_images_into parameter!");
            return;
        }
    };
    let force_overwrite = state.config.force_overwrite;
    let pb_clone = pb.clone();
    let progress_handler = move |process_info: TransitProcess| {
        pb_clone.set_length(process_info.total_bytes);
        pb_clone.set_position(process_info.copied_bytes);
        //println!("{}", process_info.total_bytes);
        if force_overwrite {
            return fs_extra::dir::TransitProcessResult::Overwrite;
        }
        fs_extra::dir::TransitProcessResult::ContinueOrAbort
    };
    pb.set_message(&format!("Installing files to {}...", &install_string));
    let mut copy_options = CopyOptions::new();
    copy_options.content_only = true;
    match move_dir_with_progress(
        path::get_output_working_dir(&state.config.input_dir).unwrap(),
        state.config.install_images_into.as_ref().unwrap(),
        &copy_options,
        progress_handler,
    ) {
        Ok(b) => error!("{}", b),
        Err(msg) => error!("{}", msg.to_string()),
    };
    pb.finish_with_message("Successfully installed images :-)");
}
