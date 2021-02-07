use {
    log::{info, error},
    std::path::PathBuf
};

/// Converts the given input file to webp using ```cwebp``` and saves it to disk
/// into the output_dir folder.
pub fn convert_from_png(input_file_name: &PathBuf, output_file_name: &PathBuf) {
    if !&input_file_name.is_file() {
        error!("{} is not a file!", &input_file_name.to_str().unwrap());
        return;
    }

    if output_file_name.is_dir() {
        error!("{} is a directory!", &output_file_name.to_str().unwrap());
        return;
    };
    let o = &mut output_file_name.clone();
    o.set_extension("webp");

    let process = match std::process::Command::new("cwebp")
        .args(&[input_file_name.to_str().unwrap(), "-q", "100", "-o", o.to_str().unwrap()])
        .spawn() {
            Ok(process) => process,
            Err(err)    => panic!("Running process error: {}", err),
    };

    let output = match process.wait_with_output() {
            Ok(output)  => output,
            Err(err)    => panic!("Retrieving output error: {}", err),
    };

    match std::string::String::from_utf8(output.stdout) {
            Ok(stdout)  => info!("{}", stdout),
            Err(err)    => panic!("Translating output error: {}", err),
    };
}
