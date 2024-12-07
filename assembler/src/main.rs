mod asm;

use std::{env, path, process};

enum ArgParseError {
    NotEnoughArgs,
    InvalidExtension,
}

fn parse_args(args: &[String]) -> Result<path::PathBuf, ArgParseError> {
    match args.get(1) {
        Some(path) => {
            let path = path::PathBuf::from(path);

            match path.extension() {
                Some(ext) if ext == "asm" => Ok(path),
                _ => Err(ArgParseError::InvalidExtension),
            }
        }
        _ => Err(ArgParseError::NotEnoughArgs),
    }
}

fn usage(args: Vec<String>, error: &str) -> ! {
    let cmd_name = args
        .first()
        .map(path::Path::new)
        .and_then(|cmd_name| cmd_name.file_name())
        .and_then(|cmd_name| cmd_name.to_str())
        .unwrap_or("assembler")
        .to_string();

    eprintln!("{error}");
    eprintln!();
    eprintln!("Usage: {cmd_name} <source_path>");
    process::exit(1);
}

fn main() {
    let args: Vec<_> = env::args().collect();

    let source_file = match parse_args(&args) {
        Ok(path) => path,
        Err(ArgParseError::NotEnoughArgs) => usage(args, "Missing <source_path> argument"),
        Err(ArgParseError::InvalidExtension) => usage(
            args,
            "Invalid source file extension, only `.asm` is supported",
        ),
    };

    match asm::compile(&source_file) {
        Ok(output_file) => println!(
            "Successfully compiled `{}` to `{}`.",
            source_file.display(),
            output_file.display()
        ),
        Err(err) => usage(
            args,
            &format!(
                "Failed to compile `{}`: {}",
                source_file.display(),
                err.cause()
            ),
        ),
    }
}
