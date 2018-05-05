#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate docopt;

use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::io::{self};
use std::path::{PathBuf};

use docopt::Docopt;
use failure::Error;

const USAGE: &'static str = "
Dropbox JPG renamer.

Usage:
    dropbox-jpg-renamer <dir-name>
    dropbox-jpg-renamer (-h | --help)

Options:
    -h, --help  Show this message.
";

const RAW_EXTS: &[&str] = &["raw", "raf"];
const JPG_EXTS: &[&str] = &["jpg", "jpeg"];

#[derive(Debug, Deserialize)]
struct Args {
    arg_dir_name: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    ::std::process::exit(match rename_files(&args.arg_dir_name) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            1
        }
    });
}

fn is_ext(exts: &[&str], entry: &DirEntry) -> bool {
    let path = entry.path();
    path.is_file() && path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| exts.contains(&ext))
        .unwrap_or(false)
}

fn dir_paths<P>(dir_name: &str, pred: P) -> io::Result<Vec<PathBuf>>
where P: Fn(&DirEntry) -> bool {
    let files = fs::read_dir(dir_name)?
        .filter(|&ref entry_res| match *entry_res {
            Ok(ref entry) => pred(&entry),
            // Keep Err entries so that they return errors when collected.
            Err(_) => true,
        })
        .collect::<io::Result<Vec<_>>>()?;

    let mut paths = files.into_iter()
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    paths.sort();
    Ok(paths)
}

fn rename_files(dir_name: &str) -> Result<(), Error> {
    let (mut raw_count, mut jpg_count) = (0, 0);
    for entry in fs::read_dir(dir_name)? {
        let entry = entry?;
        raw_count += is_ext(RAW_EXTS, &entry) as u64;
        jpg_count += is_ext(JPG_EXTS, &entry) as u64;
    }
    ensure!(raw_count == jpg_count,
            "Number of RAW files does not match number of JPG files: {} vs. {}",
            raw_count, jpg_count);

    let raw_paths = dir_paths(dir_name, |entry| is_ext(RAW_EXTS, &entry))?;
    let jpg_paths = dir_paths(dir_name, |entry| is_ext(JPG_EXTS, &entry))?;
    for (mut raw_path, jpg_path) in raw_paths.into_iter().zip(jpg_paths) {
        if raw_path.file_stem() != jpg_path.file_stem() {
            if let Some(jpg_ext) = jpg_path.extension() {
                raw_path.set_extension(jpg_ext);
                ensure!(!raw_path.is_file(),
                    "{:?} already exists; not overwriting.", raw_path);
                fs::rename(&jpg_path, &raw_path)?;
            }
        }
    }

    Ok(())
}
