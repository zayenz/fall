extern crate lang_rust;

use std::path::PathBuf;
use lang_rust::editor::SymbolIndex;


fn main() {
    let paths = vec![PathBuf::from("/home/matklad/projects/fall")];
    let index = SymbolIndex::new(paths);
    ::std::thread::sleep(::std::time::Duration::from_millis(1000));
    let results = index.query("Index");
    eprintln!("{:?}", results);
}