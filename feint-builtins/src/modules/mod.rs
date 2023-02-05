use ::std::borrow::Cow;
use ::std::collections::HashMap;
use ::std::io::Read;
use ::std::path::Path;

use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use tar::Archive as TarArchive;

pub mod std;
pub use self::std::STD;

/// At build time, a compressed archive is created containing the
/// std .fi module files (see `build.rs`).
///
/// At runtime, the module file data is read out and stored in a map
/// (lazily). When a std module is imported, the file data is read from
/// this map rather than reading from disk.
///
/// The utility of this is that we don't need an install process that
/// copies the std module files into some location on the file system
/// based on the location of the current executable or anything like
/// that.
pub static STD_FI_MODULES: Lazy<HashMap<String, Vec<u8>>> = Lazy::new(|| {
    let archive_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/modules.tgz"));
    let decoder = GzDecoder::new(archive_bytes);
    let mut archive = TarArchive::new(decoder);
    let mut modules = HashMap::new();
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path: Cow<'_, Path> = entry.path().unwrap();
        let path = path.to_str().unwrap().to_owned();
        let mut result = Vec::new();
        entry.read_to_end(&mut result).unwrap();
        modules.insert(path, result);
    }
    modules
});
