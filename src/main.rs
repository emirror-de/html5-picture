use {
    clap::Clap,
    html5_picture::core::{
        collect_file_names,
        Config,
        create_all_output_directories,
        install_images_into,
        process_images,
        State,
    },
    queue::Queue,
};


fn main() {
    std::env::set_var("RUST_LOG", "error");
    pretty_env_logger::init();

    // parse and check arguments for validity
    let config: Config = Config::parse();

    // add all default processes
    let mut q: Queue<fn(&mut State)> = Queue::new();
    q.queue(collect_file_names).unwrap();
    q.queue(create_all_output_directories).unwrap();

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
