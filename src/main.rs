use {clap::Parser, html5_picture::core::Config};

fn main() {
    pretty_env_logger::init();

    // parse and check arguments for validity
    let config: Config = Config::parse();

    html5_picture::run(config);
}
