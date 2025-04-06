use {
    crate::{
        utils::create_progressbar,
        webp::processor::Parameter,
    },
    indicatif::MultiProgress,
    log::error,
    std::{path::PathBuf, sync::Arc}, tokio::task::JoinSet,
};

/// Contains all the required and optional parameter for the ```BatchProcessor```.
/// Currently it is only a wrapper around the single processor parameter.
pub struct BatchParameter {
    pub single_params: Parameter,
}

/// Processes all input files using the ```SingleProcessor``` struct.
pub struct BatchProcessor {
    params: BatchParameter,
    progressbars: Option<Arc<MultiProgress>>,
}

impl BatchProcessor {
    /// Creates a new instance.
    pub fn new(params: BatchParameter, progressbars: Option<Arc<MultiProgress>>) -> Self {
        Self { params, progressbars }
    }

    /// For each file name, a new ```SingleProcessor``` instance is created and
    /// spawned in a separate tokio thread. This function creates a new tokio
    /// runtime.
    pub fn run(&self, file_names: &Vec<PathBuf>) {
        // single threaded
        if self.params.single_params.single_threaded {
            self.run_single_threaded(file_names);
            return;
        }

        // multi threaded
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut handles = JoinSet::new();
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
                        create_progressbar((&params_single.scaled_images_count + 1) as u64);
                    let pb_clone = pb.clone();
                    m.add(pb);
                    Some(pb_clone)
                } else {
                    None
                };
                handles.spawn(async move {
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
                        pb.unwrap().abandon_with_message("Wrong color profile!");
                    }
                });
            }
            /*
            if let Some(m) = &self.progressbars {
                if let Err(msg) = handles.join_all() {
                    error!("Failed to join progressbars: {}", msg);
                }
            }
            futures::future::join_all(handles).await;
            */
            let _ = handles.join_all().await;
        });
    }

    fn run_single_threaded(&self, file_names: &Vec<PathBuf>) {
        for file_name in file_names {
            let full_file_name = file_name.clone();
            let mut params_single = self.params.single_params.clone();
            params_single.input = file_name.clone();
            let mut file_name = file_name.clone();
            file_name.pop();
            params_single.output_dir = crate::path::create_output_file_name(
                &self.params.single_params.input,
                &file_name
                ).unwrap();

            let mut webp_processor =
                crate::webp::processor::SingleProcessor::new(
                    params_single,
                    None,
                )
                .unwrap();
            if let Err(msg) = webp_processor.run() {
                error!("Error: {}, File: {}", msg, full_file_name.display());
            }
        }
    }
}
