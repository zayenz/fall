extern crate lang_rust;

use lang_rust::editor::SymbolIndex;
use std::path::PathBuf;

fn main() {
    let paths = vec![PathBuf::from("/home/matklad/projects/fall")];
    let index = SymbolIndex::new(paths);
    std::thread::sleep(::std::time::Duration::from_millis(1000));
    let results = index.query("Index");
    eprintln!("{:?}", results);
}
