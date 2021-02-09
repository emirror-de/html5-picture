use {
    log::{error, info},
    std::path::PathBuf,
};

pub struct WebpConverterAdapter {
    pub quality: u8,
}

impl WebpConverterAdapter {
    fn parameter_check(
        input_file_name: &PathBuf,
        output_file_name: &PathBuf,
    ) -> Result<(), String> {
        if !&input_file_name.is_file() {
            return match &input_file_name.to_str() {
                Some(v) => Err(format!("{} is not a file!", v)),
                None => Err("Input is not a file!".to_string()),
            };
        };
        if output_file_name.is_dir() {
            return match &output_file_name.to_str() {
                Some(v) => Err(format!("{} is a directory!", v)),
                None => Err("Output file is a directory!".to_string()),
            };
        };
        Ok(())
    }

    /// Converts the given input file to webp using ```cwebp``` and saves it to
    /// the given output file.
    pub fn from_png(
        &self,
        input_file_name: &PathBuf,
        output_file_name: &PathBuf,
        additional_cwebp_arguments: Option<Vec<String>>,
    ) {
        if let Err(msg) =
            Self::parameter_check(&input_file_name, &output_file_name)
        {
            error!("{}", msg);
            return;
        };
        let o = &mut output_file_name.clone();
        o.set_extension("webp");

        let mut args = vec![
            input_file_name.to_str().unwrap().to_string(),
            "-q".to_string(),
            self.quality.to_string(),
            "-o".to_string(),
            o.to_str().unwrap().to_string(),
        ];
        if let Some(v) = additional_cwebp_arguments {
            args.append(&mut v.clone());
        };
        let process =
            match std::process::Command::new("cwebp").args(&args).spawn() {
                Ok(process) => process,
                Err(err) => panic!("Running process error: {}", err),
            };

        let output = match process.wait_with_output() {
            Ok(output) => output,
            Err(err) => panic!("Retrieving output error: {}", err),
        };

        match std::string::String::from_utf8(output.stdout) {
            Ok(stdout) => info!("{}", stdout),
            Err(err) => panic!("Translating output error: {}", err),
        };
    }
}
