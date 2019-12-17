#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use serde_json;
use crate::battlechip::BattleChip;


mod battlechip;

fn main() {
    let chip_url = "https://docs.google.com/feeds/download/documents/export/Export?id=1lvAKkymOplIJj6jS-N5__9aLIDXI6bETIMz01MK9MfY&exportFormat=txt";
    println!("Hello, world!");
    /*
    let res = reqwest::get("http://spartan364.hopto.org/chips.json").
        expect("no request result").
        text().expect("no response text");
    let chips: serde_json::Value = serde_json::from_str(&res).expect("not a json");
    if !chips.is_array() {
        panic!("not a json array");
    }
    let chip_arr = chips.as_array().expect("not a json array");
    for chip in chip_arr {
        println!("{}", chip.get("Name").unwrap());
    }
    */

    let chip_text = reqwest::get(chip_url).
        expect("no request result").text().expect("no response text");
    let chip_text_repl = chip_text.replace("â€™", "'").replace("\u{FEFF}", "");
    let chip_text_arr : Vec<&str> = chip_text_repl.split("\n").filter(|&i| !i.trim().is_empty()).collect();
    //drop(chip_text);
    //let mut chip_map: HashMap<String, Box<BattleChip>> = HashMap::new();
    let mut chips : Vec<Box<BattleChip>> = vec![];
    chips.reserve(200);
    for i in (0..chip_text_arr.len()).step_by(2) {
        let to_add_res = BattleChip::from_chip_string(chip_text_arr[i], chip_text_arr[i + 1]);
        if to_add_res.is_err() {
            println!("{}\n{}", chip_text_arr[i], to_add_res.err().unwrap());
        } else {
            let to_add = to_add_res.expect("something went very wrong");
            chips.push(to_add);
            //chip_map.insert((*to_add.name.to_ascii_lowercase()).to_owned(), to_add);

        }
    }

    chips.shrink_to_fit();

    //chips.sort_unstable_by(|a,b| a.name.cmp(&b.name));
    chips.sort_unstable();
    //let chip = chips.pop().expect("nothing popped");
    //println!("{}", chip.name);

    let j = serde_json::to_string_pretty(&chips).expect("could not serialize to json");
    println!("{}", j);

    let mut chip_map: HashMap<String, Box<BattleChip>> = HashMap::new();

    //let j: serde_json::Result<String> = serde_json::to_string_pretty(&chip_map);
    //let chip_json = j.expect("could not serialize to json");
    //println!("{}",chip_json);
}

