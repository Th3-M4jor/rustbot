#[macro_use]
extern crate lazy_static;

use crate::library::ChipLibrary;

mod battlechip;
mod library;

fn main() {
    let mut chip_library = ChipLibrary::new();
    let load_res = chip_library.load_chips();
    match load_res {
        Ok(s) => {
            println!("{} chips were loaded", s);
        }
        Err(e) => {
            println!("{}", e.to_string());
        }
    }
}

