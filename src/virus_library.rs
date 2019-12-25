use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;

use serenity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;
use std::fs;
use crate::distance;
use crate::chip_library::{LibraryObject, Library};
use crate::battlechip::elements::Elements;
use unicode_normalization::UnicodeNormalization;
use std::fmt::Formatter;
use simple_error::SimpleError;


const VIRUS_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1PZKYP0mzzxMTmjJ8CfrUMapgQPHgi24Ev6VB3XLBUrU&exportFormat=txt";

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct Virus {
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

struct VirusLibrary {
    library: HashMap<String, Box<Virus>>,
}

impl VirusLibrary {
    pub fn new() -> VirusLibrary {
        VirusLibrary{
            library: HashMap::new(),
        }
    }

    pub fn load_viruses(&mut self) -> Result<usize, SimpleError> {

        self.library.clear();
        let virus_regex : Regex = Regex::new(r"((.+)\s\((\w+)\))").expect("could not compile virus regex");
        let cr_regex : Regex = Regex::new(r"CR\s+(\d+)").expect("could not compile CR regex");

        let virus_list : Vec<Box<Virus>> = vec![];
        let virus_text = reqwest::get(VIRUS_URL)
            .expect("no request result").text().expect("no response text")
            .replace("â€™", "'").replace("\u{FEFF}", "").replace("\r", "");
        let virus_text_arr: Vec<&str> =
            virus_text.split("\n").filter(|&i| !i.trim().is_empty()).collect();
        todo!();
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


