use {
    clap::Clap,
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    std::{path::PathBuf, sync::Arc},
    queue::Queue,
};

/// Converts the images (currently png only) of the input folder to webp format.
/// It also has the ability to create multiple versions of the input images
/// having different sizes. See -s for further details.
///
/// Depends on cwebp, so make sure webp is installed on your pc!
#[derive(Clap, Debug)]
#[clap(version = "0.0.4-alpha", author = "Lewin Probst <info@emirror.de>")]
struct Args {
    /// The directory containing all images that should be processed.
    pub input_dir: PathBuf,
    /// The source image width is divided by this option (value + 1). Afterwards
    /// the source image is scaled (keeping the aspect ratio) to these widths
    /// before convertion.
    ///
    /// Useful if you want to have multiple sizes of the image on the webpage
    /// for different breakpoints.
    ///
    /// Example:
    /// Input image dimensions: 6000x962
    /// Scaled images count: 3
    /// Resulting converted images:
    ///     [filename]               [dimensions]
    ///     original_filename        6000x962
    ///     original_filename-w4500  4500x751
    ///     original_filename-w3000  3000x501
    ///     original_filename-w1500  1500x250
    #[clap(short)]
    pub scaled_images_count: Option<u8>,
    #[clap(short)]
    pub install_images_into: Option<PathBuf>,
    /// Defines the quality of cwebp conversion.
    #[clap(short)]
    pub quality_webp: Option<u8>,
}

fn create_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{prefix}] {spinner} {wide_msg}"),
    );
    pb
}

type Step = fn(&mut State);

struct State {
    pub config: Args,
    pub file_names_to_convert: Vec<PathBuf>,
    pub current_step: usize,
    pub max_progress_steps: usize,
}

impl State {
    pub fn new(config: Args, max_progress_steps: usize) -> Self {
        Self {
            config,
            file_names_to_convert: vec![],
            current_step: 0,
            max_progress_steps,
        }
    }

    pub fn dequeue(&mut self, queue: &mut Queue<Step>) -> Option<Step> {
        self.current_step = self.max_progress_steps + 1 - queue.len();
        queue.dequeue()
    }

    pub fn get_prefix(&self) -> String {
        format!("{}/{}", self.current_step, self.max_progress_steps)
    }
}

fn collect_file_names(state: &mut State) {
    let pb = create_spinner();
    pb.set_prefix(&state.get_prefix());
    pb.set_message("Collecting files to convert...");
    state.file_names_to_convert = html5_picture::collect_png_file_names(
        &state.config.input_dir, //&config.input_dir,
        Some(pb.clone()),
    );
    pb.finish_with_message(&format!(
        "Collected {} files!",
        &state.file_names_to_convert.len(), //&file_names_to_convert.len()
    ));
}

fn create_all_output_directories(state: &mut State) {
    let pb = create_spinner();
    pb.set_prefix(&state.get_prefix());
    pb.set_message("Create all output directories...");
    html5_picture::fs::create_output_directories(
        &state.config.input_dir, //&config.input_dir,
        &state.file_names_to_convert, //&file_names_to_convert,
        Some(pb.clone()),
    );
    pb.finish_with_message("Created all output directories!");
}

fn process_images(state: &mut State) {
    let webp_params = html5_picture::webp::Parameter::new(state.config.quality_webp); //config.quality_webp);
    let params = html5_picture::webp::processor::Parameter {
        webp_parameter: webp_params,
        input: state.config.input_dir.clone(), //input: config.input_dir,
        output_dir: PathBuf::new(),
        scaled_images_count: state.config.scaled_images_count, //scaled_images_count: config.scaled_images_count,
    };
    let batch_params = html5_picture::webp::processor::BatchParameter {
        single_params: params,
    };
    let mp = Arc::new(MultiProgress::new());
    let batch_processor = html5_picture::webp::processor::BatchProcessor::new(
        batch_params,
        Some(Arc::clone(&mp)),
    );
    let pb = create_spinner();
    pb.set_prefix(&state.get_prefix());
    pb.set_message("Converting files...");
    batch_processor.run(&state.file_names_to_convert);
    pb.finish_with_message("Finished :-)");
}

fn main() {
    std::env::set_var("RUST_LOG", "error");
    pretty_env_logger::init();

    // parse and check arguments for validity
    let config: Args = Args::parse();

    // add all default processes
    let mut q: Queue<fn(&mut State)> = Queue::new();
    q.queue(collect_file_names).unwrap();
    q.queue(create_all_output_directories).unwrap();

    // finally add processing step
    q.queue(process_images).unwrap();

    let mut s = State::new(config, q.len());

    while let Some(step_function) = s.dequeue(&mut q) {
        step_function(&mut s);
    }
}
