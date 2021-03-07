//! # Purpose and usage
//!
//! Supports the binary within this package.
//! Contains functions to easily generate different sizes of a picture that
//! is used on webpages. Also offers the possibility to convert them into webp
//! format and is able to create a ```<picture>``` tag for the given images.
//!
//! Currently this crate is only capable of converting ```png``` files to webp using
//! ```cwebp```.
//! So make sure that webp is installed on your computer.
//!
//! The creation of picture tag files is not yet implemented but in our pipeline.
//!
//! ## Installation
//!
//! The binary can be installed via ```cargo install html5-picture```. As stated
//! before, make sure webp is installed before using.
use {
    crate::core::{
        collect_file_names,
        copy_originals_to_output,
        create_all_output_directories,
        install_images_into,
        process_images,
        Config,
        State,
    },
    indicatif::ProgressBar,
    log::error,
    queue::Queue,
    std::path::PathBuf,
    walkdir::WalkDir,
};

/// Contains default functions and traits.
pub mod core;

/// Generic helper functions.
pub mod utils;

/// Support for webp format. Used mainly for conversion.
pub mod webp;

/// Path processing that is required for ```html5_picture```
pub mod path;

/// Functions operating on the filesystem that is required for ```html5_picture```
pub mod fs;

/// HTML5 related functions, such as creation of picture tags.
pub mod html5;

/// Determines if the given input filename contains a .png extension.
pub fn is_png(input: &PathBuf) -> bool {
    match input.extension() {
        Some(s) => match s.to_str() {
            None => false,
            Some(v) => v == "png",
        },
        None => false,
    }
}

/// Collects all png file names that are stored in the ```input_dir```.
pub fn collect_png_file_names(
    input_dir: &PathBuf,
    progressbar: Option<ProgressBar>,
) -> Vec<PathBuf> {
    let mut file_names = vec![];
    for entry in WalkDir::new(&input_dir) {
        // unwrap the entry
        let entry = if let Err(msg) = &entry {
            error!("{}", msg.to_string());
            continue;
        } else {
            entry.unwrap()
        };
        let entry = entry.into_path();

        if let Some(ref pb) = progressbar {
            pb.tick();
        }

        if !is_png(&entry) {
            continue;
        }
        file_names.push(entry);
    }
    file_names
}

/// The main function of the binary. Executes all required steps for copying,
/// conversion and installationn of the source images.
pub fn run(config: Config) {
    if !&config.input_dir.exists() {
        error!("Input directory does not exist!");
        return;
    }
    match &config.scaled_images_count {
        None | Some(0) => {
            error!("Minimum scaled images count is 1!");
            return;
        }
        _ => (),
    }

    // add all default processes
    let mut q: Queue<fn(&mut State)> = Queue::new();
    q.queue(collect_file_names).unwrap();
    q.queue(create_all_output_directories).unwrap();
    q.queue(copy_originals_to_output).unwrap();

    // finally add processing step
    q.queue(process_images).unwrap();

    if let Some(_) = &config.install_images_into {
        q.queue(install_images_into).unwrap();
    }

    let mut s = State::new(config, q.len());

    while let Some(step_function) = s.dequeue(&mut q) {
        step_function(&mut s);
    }
}
