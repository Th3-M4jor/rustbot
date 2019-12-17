use std::collections::HashMap;

use serde_json;
use simple_error::SimpleError;

use crate::battlechip::BattleChip;
use std::fs;

const CHIP_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1lvAKkymOplIJj6jS-N5__9aLIDXI6bETIMz01MK9MfY&exportFormat=txt";

pub struct ChipLibrary {
    chips: HashMap<String, Box<BattleChip>>,
}

impl ChipLibrary {
    pub fn new() -> ChipLibrary {
        ChipLibrary {
            chips: HashMap::new(),
        }
    }

    //returns number of chips loaded or a simple error
    pub fn load_chips(&mut self) -> Result<usize, SimpleError> {
        self.chips.clear();

        //get chip text and replace necessary characters for compatibility
        let chip_text = reqwest::get(CHIP_URL)
            .expect("no request result").text().expect("no response text")
            .replace("â€™", "'").replace("\u{FEFF}", "");
        let chip_text_arr: Vec<&str> =
            chip_text.split("\n").filter(|&i| !i.trim().is_empty()).collect();
        let mut chips: Vec<Box<BattleChip>> = vec![];
        let mut bad_chips: Vec<String> = vec![];
        for i in (0..chip_text_arr.len()).step_by(2) {
            let to_add_res = BattleChip::from_chip_string(chip_text_arr[i], chip_text_arr[i + 1]);
            match to_add_res {
                Ok(chip) => {
                    chips.push(chip);
                },
                Err(_) => {
                    bad_chips.push(String::from(chip_text_arr[i]));
                },
            }

        }

        chips.shrink_to_fit();
        chips.sort_unstable();
        let j = serde_json::to_string_pretty(&chips).expect("could not serialize to json");
        fs::write("chips.json", j).expect("could nto write to chips.json");

        while !chips.is_empty() {
            let chip = chips.pop().expect("Something went wrong popping a chip");
            self.chips.insert(chip.Name.to_lowercase(), chip);
        }

        if bad_chips.len() > 5 {
            let bad_str = format!("There were {} bad chips", bad_chips.len());
            return Err(SimpleError::new(bad_str));
        } else if bad_chips.len() > 0 {
            let mut bad_str = format!("There were {} bad chips:\n", bad_chips.len());
            for bad_chip in bad_chips {
                bad_str.push_str(&bad_chip);
                bad_str.push('\n');
            }
            return Err(SimpleError::new(bad_str));
        } else {
            return Ok(self.chips.len());
        }
    }
}