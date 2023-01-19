use std::env;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::process;

use clap_complete::{generate_to, shells};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let out_dir = match env::var_os("OUT_DIR") {
        Some(out_dir) => out_dir,
        None => {
            eprintln!("OUT_DIR env var not set");
            eprintln!("Cannot proceed");
            eprintln!("Aborting");
            process::exit(1);
        }
    };

    let stamp_path = Path::new(&out_dir).join("feint.stamp");
    if let Err(err) = File::create(&stamp_path) {
        panic!("Failed to write stamp file: {}\n{err}", stamp_path.display());
    }

    let mut cmd = build_cli();

    let path = generate_to(shells::Bash, &mut cmd, "feint", &out_dir)?;
    println!("cargo:warning=completion file generated: {:?}", path);

    let path = generate_to(shells::Fish, &mut cmd, "feint", &out_dir)?;
    println!("cargo:warning=completion file generated: {:?}", path);

    Ok(())
}
