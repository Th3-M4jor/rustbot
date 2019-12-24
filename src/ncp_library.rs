use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;

use serenity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;
use std::fs;
use crate::distance;
use unicode_normalization::UnicodeNormalization;
use std::fmt::Formatter;

const NCP_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1cPLJ2tAUebIVZU4k7SVnyABpR9jQd7jarzix7oVys9M&exportFormat=txt";

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct NCP {
    pub Name: String,
    pub EBCost: u8,
    pub Color: String,
    pub All: String,
    pub Description: String,
}

impl NCP {
    pub fn new<T: Into<String>, S: Into<u8>> (
        name: T,
        cost: S,
        color: T,
        all: T,
        desc: T,
    ) -> NCP {
        NCP {
            Name: name.into().nfc().collect::<String>(),
            EBCost: cost.into(),
            Color: color.into().nfc().collect::<String>(),
            All: all.into().nfc().collect::<String>(),
            Description: desc.into().nfc().collect::<String>(),
        }
    }
}

impl std::fmt::Display for NCP {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(
            f,
            "```{} - ({} EB) - {}\n{}```",
            self.Name, self.EBCost, self.Color, self.Description
        );
    }
}

pub struct NCPLibrary {
   library: HashMap<String, Box<NCP>>,
}


const COLORS : &[&str] = &["white", "pink", "yellow", "green", "blue", "red", "gray"];

impl NCPLibrary {

    pub fn new() -> NCPLibrary {
        NCPLibrary {
            library: HashMap::new(),
        }
    }

    pub fn load_programs(&mut self) -> usize {
        lazy_static! {
            static ref NCP_TEST : Regex = Regex::new(r"(.+)\s\((\d+)\sEB\)\s-\s(.+)").expect("Bad NCP regex");
        }
        self.library.clear();
        let mut ncp_list : Vec<Box<NCP>> = vec![];
        let ncp_text = reqwest::get(NCP_URL)
            .expect("no request result").text().expect("no response text")
            .replace("â€™", "'").replace("\u{FEFF}", "").replace("\r", "");
        let ncp_text_arr: Vec<&str> =
            ncp_text.split("\n").filter(|&i| !i.trim().is_empty()).collect();
        let mut curr_color : String = String::new();
        //let mut new_color : String;
        for ncp in ncp_text_arr {
            if COLORS.contains(&ncp.trim().to_lowercase().as_str()) {
                curr_color = String::from(ncp.trim());
                continue;
            }

            let ncp_cap_res = NCP_TEST.captures(ncp);
            let ncp_cap;
            match ncp_cap_res {
                Some(val) => ncp_cap = val,
                None => continue,
            }
            let name = ncp_cap.get(1);
            let cost = ncp_cap.get(2);
            let desc = ncp_cap.get(3);
            if name.is_none() || cost.is_none() || desc.is_none() {
                continue;
            }
            let cost_val = cost.unwrap().as_str().parse::<u8>().unwrap_or(u8::max_value());
            ncp_list.push(Box::new(NCP::new(name.unwrap().as_str(), cost_val, &curr_color, ncp, desc.unwrap().as_str())));
        }

        let j = serde_json::to_string_pretty(&ncp_list).expect("could not serialize to json");
        fs::write("naviCust.json", j).expect("could not write to naviCust.json");
        while !ncp_list.is_empty() {
            let ncp = ncp_list.pop().unwrap();
            self.library.insert(ncp.Name.to_lowercase(), ncp);
        }
        return self.library.len();
    }

    pub fn get(&self, to_get: &str) -> Option<&Box<NCP>> {
        return self.library.get(&to_get.to_lowercase());
    }

    pub fn name_contains(&self, to_get: &str) -> Option<Vec<String>> {
        let to_search = to_get.to_lowercase();
        let mut to_ret : Vec<String> = vec![];
        for key in self.library.keys() {
            if key.contains(&to_search) {
                to_ret.push(self.library.get(key).unwrap().Name.clone());
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
        for val in self.library.values() {
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

}

impl TypeMapKey for NCPLibrary {
    type Value = Arc<RwLock<NCPLibrary>>;
}