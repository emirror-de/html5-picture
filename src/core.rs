use {
    crate::{
        html5::Picture, path, utils, webp::WebpParameter,
        webp::processor::BatchParameter,
        webp::processor::Parameter as ProcessorParameter,
    },
    clap::{Parser, crate_authors, crate_version},
    fs_extra::dir::{
        CopyOptions, TransitProcess, copy_with_progress, move_dir_with_progress,
    },
    indicatif::MultiProgress,
    log::error,
    queue::Queue,
    std::{path::PathBuf, sync::Arc},
};

#[cfg(debug_assertions)]
use log::debug;

type Step = fn(&mut State);

/// Converts the images (currently png only) of the input folder to webp format.
/// It also has the ability to create multiple versions of the input images
/// having different sizes. See -s for further details.
/// Additionally it automatically generates HTML5 <picture> tag files for you
/// to be able to integrate them in a webpage easily.
///
/// Depends on cwebp, so make sure webp is installed on your pc!
///
/// Example:
/// html5-picture ./assets 3;
/// Input image dimensions: 6000x962;
/// Scaled images count: 3;
/// Resulting converted images:
///     original_filename        6000x962;
///     original_filename-w4500  4500x751;
///     original_filename-w3000  3000x501;
///     original_filename-w1500  1500x250;
#[derive(Parser, Debug, Clone)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(", "),
)]
pub struct Config {
    /// The directory containing all images that should be processed.
    pub input_dir: PathBuf,
    /// The source image width is divided by this option (value + 1). Afterwards
    /// the source image is scaled (keeping the aspect ratio) to these widths
    /// before conversion.
    /// Useful if you want to have multiple sizes of the image on the webpage
    /// for different breakpoints.
    pub scaled_images_count: u8,
    /// Installs the converted and sized pictures into the given folder.
    #[clap(short)]
    pub install_images_into: Option<PathBuf>,
    /// The destination folder of HTML5 picture tag files.
    #[clap(short)]
    pub picture_tags_output_folder: Option<PathBuf>,
    /// Can be used in combination with -p, sets the mountpoint for links in
    /// the HTML tags.
    #[clap(short)]
    pub mountpoint: Option<PathBuf>,
    /// If true, existing files are overwritten if install-images-into is set.
    #[clap(short, long)]
    pub force_overwrite: bool,
    /// Defines the quality of cwebp conversion.
    #[clap(short)]
    pub quality_webp: Option<u8>,
    /// If set, the processing is done single threaded.
    #[clap(short)]
    pub single_threaded: bool,
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
    pb.set_prefix(state.get_prefix());
    pb.set_message("Collecting files to convert...");
    state.file_names_to_convert = crate::collect_png_file_names(
        &state.config.input_dir,
        Some(pb.clone()),
    );
    pb.finish_with_message(format!(
        "Collected {} files!",
        &state.file_names_to_convert.len(),
    ));
}

/// Recreates the folder structure of the input directory in the output directory.
pub fn create_all_output_directories(state: &mut State) {
    let pb = utils::create_spinner();
    pb.set_prefix(state.get_prefix());
    let message = if state.config.install_images_into.is_some() {
        "Creating all temporary output directories..."
    } else {
        "Creating all output directories..."
    };
    pb.set_message(message);
    crate::fs::create_output_directories(
        &state.config.input_dir,
        &state.file_names_to_convert,
        Some(pb.clone()),
    );
    let message = if state.config.install_images_into.is_some() {
        "Created all temporary output directories!"
    } else {
        "Created all output directories!"
    };
    pb.finish_with_message(message);
}

/// Copies the input folder to the working directory.
pub fn copy_originals_to_output(state: &mut State) {
    let pb = utils::create_progressbar(0);
    let pb_clone = pb.clone();
    let force_overwrite = state.config.force_overwrite;
    let progress_handler = move |process_info: TransitProcess| {
        pb_clone.set_length(process_info.total_bytes);
        pb_clone.set_position(process_info.copied_bytes);
        if force_overwrite {
            return fs_extra::dir::TransitProcessResult::Overwrite;
        }
        fs_extra::dir::TransitProcessResult::Skip
    };
    pb.set_prefix(state.get_prefix());
    pb.set_message("Copying original files...");
    let mut copy_options = CopyOptions::new();
    copy_options.content_only = true;
    copy_options.skip_exist = true;
    if let Err(msg) = copy_with_progress(
        &state.config.input_dir,
        path::get_output_working_dir(&state.config.input_dir).unwrap(),
        &copy_options,
        progress_handler,
    ) {
        error!("{}", msg.to_string());
    }
    pb.finish_with_message("Successfully copied original images!");
}

/// Resizes and converts all input images.
pub fn process_images(state: &mut State) {
    let webp_params = WebpParameter::new(state.config.quality_webp);
    let params = ProcessorParameter {
        webp_parameter: webp_params,
        input: state.config.input_dir.clone(),
        output_dir: PathBuf::new(),
        scaled_images_count: state.config.scaled_images_count,
        single_threaded: state.config.single_threaded,
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
    pb.set_prefix(state.get_prefix());
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
                    pb.abandon_with_message(format!(
                        "Could not create folder: {}",
                        msg.to_string()
                    ));
                }
            }
        }
    }
    pb.set_prefix(state.get_prefix());
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
        if force_overwrite {
            return fs_extra::dir::TransitProcessResult::Overwrite;
        }
        fs_extra::dir::TransitProcessResult::Skip
    };
    pb.set_message(format!("Installing files to {}...", &install_string));
    let mut copy_options = CopyOptions::new();
    copy_options.content_only = true;
    copy_options.skip_exist = true;
    if let Err(msg) = move_dir_with_progress(
        path::get_output_working_dir(&state.config.input_dir).unwrap(),
        state.config.install_images_into.as_ref().unwrap(),
        &copy_options,
        progress_handler,
    ) {
        error!("{}", msg.to_string());
    }
    pb.finish_with_message(format!(
        "Successfully installed images to {}!",
        state.config.install_images_into.as_ref().unwrap().display()
    ));
}

/// Saves the html `<picture>` tags to the folder given by the options.
pub fn save_html_picture_tags(state: &mut State) {
    let pb =
        utils::create_progressbar(state.file_names_to_convert.len() as u64);
    pb.set_prefix(state.get_prefix());
    pb.set_message("Writing HTML picture tag files...");

    if let None = &state.config.picture_tags_output_folder {
        pb.abandon_with_message(
            "Parameter picture_tags_output_folder not set!",
        );
        return;
    }

    for file_name in &state.file_names_to_convert {
        use std::io::prelude::*;
        let mut output_name = file_name.clone();
        output_name.set_extension("html");
        let output_tag_file_name =
            match crate::path::create_output_file_name_with_output_dir(
                &state.config.picture_tags_output_folder.as_ref().unwrap(),
                &state.config.input_dir,
                &output_name,
            ) {
                Ok(name) => name,
                Err(msg) => {
                    pb.abandon_with_message(format!("{}", msg.to_string()));
                    return;
                }
            };

        #[cfg(debug_assertions)]
        debug!("{:#?}", output_tag_file_name);

        if std::path::Path::new(&output_tag_file_name).exists()
            && !state.config.force_overwrite
        {
            #[cfg(debug_assertions)]
            debug!("Skipping file {:#?}", output_tag_file_name);
            continue;
        }

        let parent_folder = match output_tag_file_name.parent() {
            Some(p) => p,
            None => {
                pb.abandon_with_message(format!(
                    "No parent folder available for {}",
                    output_tag_file_name.display()
                ));
                return;
            }
        };
        let is_folder = match std::fs::metadata(parent_folder) {
            Ok(v) => v.is_dir(),
            Err(_) => false,
        };
        if !is_folder {
            if let Err(msg) = std::fs::create_dir_all(parent_folder) {
                error!(
                    "Parent folder could not be created: {}",
                    msg.to_string()
                );
                return;
            }
        }

        let mut pic =
            Picture::from(&file_name, state.config.scaled_images_count)
                .unwrap();

        if let Some(mountpoint) = &state.config.mountpoint {
            for source in &mut pic.sources {
                source.srcset =
                    match crate::path::create_output_file_name_with_output_dir(
                        &mountpoint,
                        &state.config.input_dir,
                        &PathBuf::from(&source.srcset),
                    ) {
                        Ok(name) => String::from(name.to_str().unwrap()),
                        Err(msg) => {
                            pb.abandon_with_message(format!(
                                "{}",
                                msg.to_string()
                            ));
                            return;
                        }
                    };
            }
            pic.fallback_uri =
                match crate::path::create_output_file_name_with_output_dir(
                    &mountpoint,
                    &state.config.input_dir,
                    &PathBuf::from(&pic.fallback_uri),
                ) {
                    Ok(name) => String::from(name.to_str().unwrap()),
                    Err(msg) => {
                        pb.abandon_with_message(format!("{}", msg.to_string()));
                        return;
                    }
                };
        }

        let mut html_file = match std::fs::File::create(output_tag_file_name) {
            Ok(f) => f,
            Err(msg) => {
                error!("{}", msg.to_string());
                return;
            }
        };
        if let Err(msg) =
            html_file.write_all(pic.to_html_string(None, "").as_bytes())
        {
            error!("{}", msg.to_string());
        };
        pb.inc(1);
    }
    pb.finish_with_message(format!(
        "Successfully wrote HTML picture tag files to: {}",
        &state
            .config
            .picture_tags_output_folder
            .as_ref()
            .unwrap()
            .display()
    ));
}
