use {clap::Clap, html5_picture::core::Config};

fn main() {
    std::env::set_var("RUST_LOG", "error");
    pretty_env_logger::init();

    // parse and check arguments for validity
    let config: Config = Config::parse();

    html5_picture::run(config);
}
