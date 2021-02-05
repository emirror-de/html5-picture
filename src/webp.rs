use {
    log::{info, error},
    std::path::PathBuf
};

/// Converts the given input file to webp using ```cwebp``` and saves it to disk
/// into the output_dir folder.
pub fn convert_from_png(input: &PathBuf, output_dir: &PathBuf) {
    if !&input.is_file() {
        error!("{} is not a file!", &input.to_str().unwrap());
        return;
    }

    let mut output = output_dir.clone();
    if !&output.is_dir() {
        error!("{} is not a directory!", &output_dir.to_str().unwrap());
        return;
    } else {
        let i = &mut input.clone();
        i.set_extension("webp");
        output.push(&i);
    };

    let process = match std::process::Command::new("cwebp")
        .args(&[input.to_str().unwrap(), "-q", "100", "-o", output.to_str().unwrap()])
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
