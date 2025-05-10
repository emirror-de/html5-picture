use indicatif::ProgressBar;

pub mod imageops;
mod resized_image_details;

pub use resized_image_details::ResizedImageDetails;

/// Creates a spinner that can be used to indicate progress.
pub fn create_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    /*
    pb.set_style(
        ProgressStyle::with_template("[{prefix}] {spinner} {wide_msg}")
            .unwrap(),
    );
    */
    pb
}

/// Creates a progress bar that can be used to indicate progress.
pub fn create_progressbar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    /*
    pb.set_style(
        ProgressStyle::with_template(
            "[{prefix}] {msg} {wide_bar: .cyan/blue} {pos:0}/{len}",
        )
        .unwrap(),
    );
    */
    pb
}
