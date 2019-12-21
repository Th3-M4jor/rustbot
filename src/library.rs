use std::collections::HashMap;


use serenity::prelude::*;

use serde_json;
use simple_error::SimpleError;


use crate::battlechip::BattleChip;
use crate::battlechip::elements::Elements;
use crate::battlechip::skills::Skills;
use std::fs;
use std::str::FromStr;
use crate::distance;

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

    pub fn get(&self, to_get: &str) -> Option<&Box<BattleChip>> {
        return self.chips.get(&to_get.to_lowercase());
    }

    pub fn name_contains(&self, to_get: &str) -> Option<Vec<String>> {
        let to_search = to_get.to_lowercase();
        let mut to_ret : Vec<String> = vec![];
        for key in self.chips.keys() {
            if key.contains(&to_search) {
                to_ret.push(self.chips.get(key).unwrap().Name.clone());
                if to_ret.len() > 5 {
                    break;
                }
            }
        }
        if to_ret.is_empty() {return None;}
        return Some(to_ret);
    }

    pub fn distance(&self, to_get: &str) -> Vec<String> {
        let mut distances : Vec<(usize,String)> = vec![];
        for val in self.chips.values() {
            let dist_res = distance::get_damerau_levenshtein_distance(
                &to_get.to_lowercase(), &val.Name.to_lowercase()
            );
            match dist_res {
                Ok(d) => distances.push((d,val.Name.clone())),
                Err(_) => continue,
            }
        }
        distances.sort_unstable_by(|a,b| a.0.cmp(&b.0));
        distances.truncate(5);
        distances.shrink_to_fit();
        let mut to_ret : Vec<String> = vec![];
        for val in distances {
            to_ret.push(val.1.clone());
        }
        return to_ret;
    }

    pub fn search_element(&self, to_get: &str) -> Option<Vec<String>> {
        //let elem_res = Elements::from_str(to_get);
        let elem_to_get;
        match Elements::from_str(to_get) {
            Ok(e) => elem_to_get = e,
            Err(_) => return None,
        }
        let mut to_ret : Vec<String> = vec![];
        for val in self.chips.values() {
            if val.Elements.contains(&elem_to_get) {
                to_ret.push(val.Name.clone());
            }
        }
        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable();
        return Some(to_ret);
    }

    pub fn search_skill(&self, to_get: &str) -> Option<Vec<String>> {
        let skill_to_get;

        let skill_res = Skills::from_str(to_get);
        match skill_res {
            Ok(s) => skill_to_get = s,
            Err(_) => return None,
        }
        let mut to_ret : Vec<String> = vec![];
        for val in self.chips.values() {
            if val.Skills.contains(&skill_to_get) {
                to_ret.push(val.Name.clone());
            }
        }
        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable();
        return Some(to_ret);
    }

    pub fn search_skill_check(&self, to_get: &str) -> Option<Vec<String>> {
        let skill_to_get;
        let skill_res = Skills::from_str(to_get);
        match skill_res {
            Ok(s) => skill_to_get = s,
            Err(_) => return None,
        }
        let mut to_ret : Vec<String> = vec![];
        for val in self.chips.values() {
            if val.SkillTarget == skill_to_get {
                to_ret.push(val.Name.clone());
            }
        }
        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable();
        return Some(to_ret);
    }

    pub fn search_by_skill_user(&self, to_get: &str) -> Option<Vec<String>> {
        let skill_to_get;
        let skill_res = Skills::from_str(to_get);
        match skill_res {
            Ok(s) => skill_to_get = s,
            Err(_) => return None,
        }
        let mut to_ret : Vec<String> = vec![];
        for val in self.chips.values() {
            if val.SkillUser == skill_to_get {
                to_ret.push(val.Name.clone());
            }
        }
        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable();
        return Some(to_ret);
    }
}

impl TypeMapKey for ChipLibrary {
    type Value = ChipLibrary;
}