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
use failure::Fallible;

const USAGE: &'static str = "
Dropbox JPG renamer.

Usage:
    dropbox-jpg-renamer <dir-name>
    dropbox-jpg-renamer (-h | --help)

Options:
    -h, --help  Show this message.
";

const RAW_EXTS: &[&str] = &["raw", "raf", "nef"];
const JPG_EXTS: &[&str] = &["jpg", "jpeg"];

#[derive(Debug, Deserialize)]
struct Args {
    arg_dir_name: String,
}

fn main() -> Fallible<()> {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())?;

    rename_files(&args.arg_dir_name)?;
    Ok(())
}

fn is_ext(exts: &[&str], entry: &DirEntry) -> bool {
    let path = entry.path();
    path.is_file() && path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| exts.contains(&ext))
        .unwrap_or(false)
}

fn dir_paths<P>(dir_name: &str, pred: P) -> Fallible<Vec<PathBuf>>
where P: Fn(&DirEntry) -> bool {
    let files = fs::read_dir(dir_name)?
        .filter(|entry_res| match entry_res {
            Ok(entry) => pred(&entry),
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

fn rename_files(dir_name: &str) -> Fallible<()> {
    let raw_paths = dir_paths(dir_name, |entry| is_ext(RAW_EXTS, &entry))?;
    let jpg_paths = dir_paths(dir_name, |entry| is_ext(JPG_EXTS, &entry))?;

    ensure!(raw_paths.len() == jpg_paths.len(),
            "Number of RAW files does not match number of JPG files: {} vs. {}",
            raw_paths.len(), jpg_paths.len());

    for (mut raw_path, jpg_path) in raw_paths.into_iter().zip(jpg_paths) {
        if raw_path.file_stem() != jpg_path.file_stem() {
            if let Some(jpg_ext) = jpg_path.extension() {
                raw_path.set_extension(jpg_ext);
                if raw_path.is_file() {
                    eprintln!("{:?} already exists; not overwriting.", raw_path);
                } else {
                    fs::rename(&jpg_path, &raw_path)?;
                }
            }
        }
    }

    Ok(())
}
