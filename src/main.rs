#[macro_use]
extern crate serde_derive;
extern crate docopt;

use docopt::Docopt;

const USAGE: &'static str = "
Dropbox JPG renamer.

Usage:
    dropbox-jpg-renamer <dir-name>
    dropbox-jpg-renamer (-h | --help)

Options:
    -h, --help  Show this message.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_dir_name: Vec<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    println!("{:?}", args);
}
