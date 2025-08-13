//! ## Purpose
//!
//! Supports the binary within this package.
//! Contains functions to easily generate different sizes of a picture that
//! can be used on webpages. Also offers the possibility to convert them into webp
//! format and is able to create `<picture>` tags for the given images.
//!
//! Currently this crate is only capable of converting `png` files to webp using
//! `cwebp`.
//! So make sure that webp is installed on your computer.
//!
//! ## Installation
//!
//! The binary can be installed via `cargo install html5-picture`. As stated
//! before, make sure webp is installed before using.
//!
//! ## Usage
//!
//! Use `html5-picture --help` for an overview of all parameters.
//!
//! ## Examples
//!
//! ### Conversion with three scales and 70% quality
//! If you want to create three different sizes of the images in `./assets`
//! with a conversion quality of 70%, enter the following command:
//!
//! ```bash
//! html5-picture ./assets 3 -q 70
//! ```
//!
//! This will convert your images and save them to `./assets-html5picture`.
//! This folder is also the working directory - make sure to not modify it while
//! the application is running.
//!
//! ### Conversion with given installation folder
//! If you pass `-i <folder_name>` as parameter, the resulting files are
//! moved from the working directory to the given `<folder_name>` after conversion
//! and scaling.
//!
//! ```bash
//! html5-picture ./assets 3 -q 100 -i ./assets-build
//! ```
//!
//! In this example the images are installted to `./assets-build`.
//!
//! ### Force overwriting
//! Using the `-f` or `--force-overwrite` flag will overwrite existing webp or
//! HTML5 picture tag files.
//!
//! ```bash
//! html5-picture ./assets 3 -q 100 -i ./dist -f
//! ```
//!
//! ### Writing picture tag files to disk
//! With the `-p` option, it is possible to save the `<picture>` tags to disk.
//! However it is recommended to use it in combination with -m, which sets the
//! mountpoint in your tag files for you.
//!
//! ```bash
//! html5-picture ./assets 4 -i ./dist -p ./html5-tags -m /some/web-server/mountpoint
//! ```
//!
//! ### Read input files by JSON
//! **Upcoming feature.**
//! The pictures can be defined using JSON format. `html5-picture` will read it
//! from stdin. This enables definition of all attributes such as `alt` per image.
#[deny(missing_docs)]
#[deny(rustdoc::missing_crate_level_docs)]
#[deny(rustdoc::broken_intra_doc_links)]
#[deny(rustdoc::private_intra_doc_links)]
use {
    crate::core::{
        Config, State, cleanup_temporary_directory, collect_file_names,
        copy_originals_to_output, create_all_output_directories,
        install_images_into, process_images, save_html_picture_tags,
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
/// conversion and installation of the source images.
pub fn run(config: Config) {
    if !&config.input_dir.exists() {
        error!("Input directory does not exist!");
        return;
    }
    match &config.scaled_images_count {
        0 => {
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

    // optional steps
    if let Some(_) = &config.install_images_into {
        q.queue(install_images_into).unwrap();
    }
    if let Some(_) = &config.picture_tags_output_folder {
        q.queue(save_html_picture_tags).unwrap();
    }

    // Always clean up temporary directory as the final step
    q.queue(cleanup_temporary_directory).unwrap();

    let mut s = State::new(config, q.len());

    while let Some(step_function) = s.dequeue(&mut q) {
        step_function(&mut s);
    }
}
