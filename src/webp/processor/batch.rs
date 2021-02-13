use {
    super::Parameter,
    indicatif::{MultiProgress, ProgressBar, ProgressStyle},
    log::error,
    std::{path::PathBuf, sync::Arc},
};

fn create_progressbar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{prefix}] {msg} {wide_bar: .cyan/blue} {pos:0}/{len}"),
    );
    pb
}

pub struct BatchParameter {
    pub single_params: Parameter,
}

pub struct BatchProcessor {
    params: BatchParameter,
    progressbars: Option<Arc<MultiProgress>>,
}

impl BatchProcessor {
    pub fn new(params: BatchParameter, progressbars: Option<Arc<MultiProgress>>) -> Self {
        Self { params, progressbars }
    }

    pub fn run(&self, file_names: &Vec<PathBuf>) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut handles = vec![];
            for file_name in file_names {
                let mut params_single = self.params.single_params.clone();
                params_single.input = file_name.clone();
                let mut file_name = file_name.clone();
                file_name.pop();
                params_single.output_dir = crate::path::create_output_file_name(
                    &self.params.single_params.input,
                    &file_name
                    ).unwrap();

                let pb = if let Some(m) = &self.progressbars {
                    let pb =
                        create_progressbar((&params_single.scaled_images_count.unwrap() + 1) as u64);
                    let pb_clone = pb.clone();
                    m.add(pb);
                    Some(pb_clone)
                } else {
                    None
                };
                let h = tokio::spawn(async move {
                    let result = std::panic::catch_unwind(|| {
                        let mut webp_processor =
                            crate::webp::processor::SingleProcessor::new(
                                params_single,
                                pb.clone(),
                            )
                            .unwrap();
                        if let Err(msg) = webp_processor.run() {
                            error!("Error: {}", msg);
                        };
                    });
                    if result.is_err() {
                        pb.unwrap().abandon_with_message("Image processing failed!");
                    }
                });
                handles.push(h);
            }
            if let Some(m) = &self.progressbars {
                if let Err(msg) = m.join_and_clear() {
                    error!("Failed to join and clear progressbars: {}", msg);
                }
            }
            futures::future::join_all(handles).await;
        });
    }
}
