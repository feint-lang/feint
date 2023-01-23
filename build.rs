use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Error;
use std::path::Path;
use std::process;

use clap_complete::{self, shells};
use flate2::{Compression, GzBuilder};
use tar::Builder as TarBuilder;

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
    make_module_archive(out_dir)?;

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

fn make_module_archive(out_dir: &Path) -> Result<(), Error> {
    let archive_path = out_dir.join("modules.tgz");
    let archive_file = File::create(archive_path)?;

    let mut archive = TarBuilder::new(
        GzBuilder::new()
            .filename("modules.tar")
            .write(archive_file, Compression::best()),
    );

    let mut add_modules = |dir_name| {
        let dir_path = Path::new("src").join("modules").join(dir_name);

        let mut files = fs::read_dir(dir_path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();

        files.sort();

        for file in files {
            if file.extension() == Some(OsStr::new("fi")) {
                let file_name = file.file_name().unwrap().to_str().unwrap();
                let file_name = file_name.strip_suffix(".fi").unwrap();
                let name = format!("{dir_name}.{file_name}");
                archive.append_path_with_name(&file, name).unwrap();
                println!("cargo:rerun-if-changed={}", file.display());
            }
        }
    };

    add_modules("std");

    let encoder = archive.into_inner().unwrap();
    encoder.finish().unwrap();

    Ok(())
}
