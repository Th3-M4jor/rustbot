use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;

use serenity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;
use std::fs;
use crate::chip_library::{LibraryObject, Library};
use crate::battlechip::elements::Elements;
use unicode_normalization::UnicodeNormalization;
use std::fmt::Formatter;
use simple_error::SimpleError;
use std::str::FromStr;

const VIRUS_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1PZKYP0mzzxMTmjJ8CfrUMapgQPHgi24Ev6VB3XLBUrU&exportFormat=txt";

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Virus {
    pub Name: String,
    pub Element: Elements,
    pub CR: u8,
    pub Description: String,
}

impl Virus {
    pub fn new<T: Into<String>, S: Into<u8>> (
        name: T,
        elem: Elements,
        cr: S,
        desc: T,
    ) -> Virus {
        Virus {
            Name: name.into().nfc().collect::<String>(),
            Element: elem,
            CR: cr.into(),
            Description: desc.into().nfc().collect::<String>(),
        }
    }
}

impl LibraryObject for Virus {
    fn get_name(&self) -> &str {
        return &self.Name;
    }
}

impl std::fmt::Display for Virus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "```{} - CR {}\n{}```", self.Name, self.CR, self.Description);
    }
}

pub struct VirusLibrary {
    library: HashMap<String, Box<Virus>>,
    highest_cr: u8,
}

impl VirusLibrary {
    pub fn new() -> VirusLibrary {
        VirusLibrary{
            library: HashMap::new(),
            highest_cr: 0,
        }
    }

    pub fn load_viruses(&mut self) -> Result<usize, SimpleError> {

        lazy_static! {
            static ref VIRUS_REGEX : Regex = Regex::new(r"((.+)\s\((\w+)\))").expect("could not compile virus regex");
            static ref CR_REGEX : Regex = Regex::new(r"CR\s+(\d+)").expect("could not compile CR regex");
        }

        self.library.clear();
        //let virus_regex : Regex = Regex::new(r"((.+)\s\((\w+)\))").expect("could not compile virus regex");
        //let cr_regex : Regex = Regex::new(r"CR\s+(\d+)").expect("could not compile CR regex");

        //let mut virus_list : Vec<Box<Virus>> = vec![];
        let mut duplicates : Vec<String> = vec![];
        let virus_text = reqwest::get(VIRUS_URL)
            .expect("no request result").text().expect("no response text")
            .replace("â€™", "'").replace("\u{FEFF}", "").replace("\r", "");
        let virus_text_arr: Vec<&str> =
            virus_text.split("\n").filter(|&i| !i.trim().is_empty()).collect();
        let mut curr_cr : u8 = 0;
        let mut current_virus_name = String::new();
        let mut current_virus_element : Elements = Elements::Null;
        let mut current_virus_full_name = String::new();
        let mut current_virus_description = String::new();
        let mut found_duplicates = false;
        for virus_line in virus_text_arr {
            let cr_cap = CR_REGEX.captures(virus_line);
            let virus_cap = VIRUS_REGEX.captures(virus_line);
            if (cr_cap.is_some() || virus_cap.is_some()) && !current_virus_description.is_empty() {
                let add_res;
                if duplicates.contains(&current_virus_name.to_lowercase()) {
                    add_res = self.library.insert(
                        current_virus_full_name.to_lowercase(),
                        Box::new(Virus::new(
                                &current_virus_full_name, current_virus_element,
                                curr_cr, &current_virus_description
                            )));
                } else {
                    add_res = self.library.insert(
                        current_virus_name.to_lowercase(),
                        Box::new(Virus::new(
                                &current_virus_name, current_virus_element,
                                curr_cr, &current_virus_description
                            )));
                }
                if add_res.is_some() {
                    //println!("{} , {} , {}\n{}", current_virus_name, current_virus_full_name, current_virus_element, current_virus_description);
                    //found a duplicate, fixing
                    //let add_res_val = add_res.unwrap();
                    let mut old_virus = add_res.unwrap();
                    let new_virus_res = self.library.remove(&current_virus_name.to_lowercase());
                    if new_virus_res.is_none() {
                        return Err(SimpleError::new(format!("found an unrecoverable duplicate, {} 119 {:?}", current_virus_full_name, duplicates)));
                    }
                    let mut new_virus = new_virus_res.unwrap();

                    if new_virus.Element == old_virus.Element {
                        return Err(SimpleError::new(format!("found an unrecoverable duplicate, {}, 124", current_virus_full_name)));
                    }
                    new_virus.Name.push_str(&format!(" ({})", new_virus.Element));
                    old_virus.Name.push_str(&format!(" ({})", old_virus.Element));
                    let new_add_res = self.library.insert(new_virus.Name.to_lowercase(), new_virus);
                    let old_add_res = self.library.insert(old_virus.Name.to_lowercase(), old_virus);
                    if new_add_res.is_some() || old_add_res.is_some() {
                        return Err(SimpleError::new(format!("found an unrecoverable duplicate, {}, 131", current_virus_full_name)));
                    }
                    duplicates.push(current_virus_name.to_lowercase());
                    found_duplicates = true;
                }
                current_virus_name.clear();
                current_virus_full_name.clear();
                current_virus_description.clear();
            }
            if cr_cap.is_some() {
                let cr_val = cr_cap.unwrap();
                curr_cr = cr_val.get(1).unwrap().as_str().parse::<u8>().unwrap_or(u8::max_value());
            }
            else if virus_cap.is_some() {
                let virus_val = virus_cap.unwrap();
                current_virus_full_name.push_str(virus_val.get(1).unwrap().as_str());
                current_virus_name.push_str(virus_val.get(2).unwrap().as_str());
                current_virus_element = Elements::from_str(virus_val.get(3)
                    .expect("Virus had no element").as_str()).expect("could not parse element");
                //current_virus_full_name.push_str(virus_val.get(3).unwrap().as_str());
            } else {
                current_virus_description.push_str(virus_line);
                current_virus_description.push('\n');
            }
        }
        self.highest_cr = curr_cr;

        //only write json file if not debug
        #[cfg(not(debug_assertions))]
        {
            let mut viruses: Vec<&Box<Virus>> = self.library.values().collect();
            viruses.sort_unstable_by(|a, b| a.Name.cmp(&b.Name));

            let j = serde_json::to_string_pretty(&viruses).expect("could not serialize virus library to JSON");
            fs::write("./virusCompendium.json", j).expect("could not write to virusCompendium.json");
        }

        if found_duplicates {
            let res = format!(
                "{} viruses loaded, found {} duplicates, recovered from all\nThey were: {:?}",
                self.library.len(), duplicates.len(), duplicates);
            return Err(SimpleError::new(&res));
        } else {
            return Ok(self.library.len());
        }
    }

    pub fn get_cr(&self, cr_to_get: u8) -> String {
        if cr_to_get > self.highest_cr {
            return "There are no viruses in that CR yet".to_string();
        }

        let mut to_ret : Vec<&str> = vec![];
        for virus in self.library.values() {
            if virus.CR == cr_to_get {
                to_ret.push(&virus.Name);
            }
        }
        to_ret.sort_unstable();
        let viruses_in_cr : String = to_ret.join(", ");
        return viruses_in_cr;
        //return "not done yet".to_string();
    }

}

impl Library for VirusLibrary {
    type LibObj = Virus;

    fn get_collection(&self) -> &HashMap<String, Box<Virus>> {
        return &self.library;
    }
}

impl TypeMapKey for VirusLibrary {
    type Value = Arc<RwLock<VirusLibrary>>;
}


