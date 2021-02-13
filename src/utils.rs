use {
    indicatif::{ProgressBar, ProgressStyle},
};

pub fn create_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{prefix}] {spinner} {wide_msg}"),
    );
    pb
}

pub fn create_progressbar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{prefix}] {msg} {wide_bar: .cyan/blue} {pos:0}/{len}"),
    );
    pb
}
