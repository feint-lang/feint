use std::env;
use std::fs::File;
use std::io::Error;
use std::path::Path;
use std::process;

use clap_complete::{self, shells};

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

    let out_dir = Path::new(&out_dir);

    stamp(out_dir)?;
    make_shell_completion_scripts(out_dir)?;

    Ok(())
}

/// Adds a stamp file to the build output directory so that the latest
/// build can be found by other tools.
fn stamp(out_dir: &Path) -> Result<(), Error> {
    let stamp_path = Path::new(out_dir).join("feint.stamp");
    File::create(stamp_path)?;
    Ok(())
}

fn make_shell_completion_scripts(out_dir: &Path) -> Result<(), Error> {
    let mut cmd = build_cli();
    clap_complete::generate_to(shells::Bash, &mut cmd, "feint", out_dir)?;
    clap_complete::generate_to(shells::Fish, &mut cmd, "feint", out_dir)?;
    Ok(())
}
