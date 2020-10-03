extern crate clap;
extern crate fall_gen;
extern crate fall_tree;
extern crate lang_fall;

use std::path::PathBuf;

use fall_gen::{process, Result, Task};

use clap::{App, Arg};

fn main() -> Result<()> {
    let matches = App::new("Fall parser generator")
        .arg(Arg::with_name("grammar").index(1).required(true))
        .arg(Arg::with_name("tests").long("tests"))
        .get_matches();
    let grammar = PathBuf::from(matches.value_of("grammar").unwrap());
    let command = if matches.is_present("tests") {
        Task::Examples(grammar)
    } else {
        Task::Generate(grammar)
    };
    process(command)
}
