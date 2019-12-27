use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;

use rand::distributions::{Distribution, Uniform};

use rand::rngs::ThreadRng;

use serenity::{
    model::channel::Message,
    prelude::*,
};

use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;
use std::fs;
use crate::library::{Library, LibraryObject, elements::Elements, search_lib_obj};
use unicode_normalization::UnicodeNormalization;
use std::fmt::Formatter;
use simple_error::SimpleError;
use std::str::FromStr;
use std::ops::Deref;

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
    #[inline]
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
            static ref VIRUS_REGEX : Regex = Regex::new(r"^((.+)\s\((\w+)\))$").expect("could not compile virus regex");
            static ref CR_REGEX : Regex = Regex::new(r"^CR\s+(\d+)$").expect("could not compile CR regex");
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
            viruses.sort_unstable_by(|a, b| a.CR.cmp(&b.CR).then_with(|| a.Name.cmp(&b.Name)));

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

    pub fn get_cr(&self, cr_to_get: u8) -> Option<Vec<&str>> {
        if cr_to_get > self.highest_cr {
            return None;
        }
        return self.search_any(
            cr_to_get,
                |a,b|
                a.CR == b
        );
    }

    pub fn search_element(&self, elem: &str) -> Option<Vec<&str>> {

        let elem_to_get = Elements::from_str(elem).ok()?;

        return self.search_any(
            elem_to_get,
            |a,b|
            a.Element == b
        );
    }

    pub fn get_highest_cr(&self) -> u8 {
        return self.highest_cr;
    }

    pub fn single_cr_random_encounter(&self, cr_to_get: u8, num_viruses: usize) -> Option<Vec<&str>> {

        let viruses = self.get_cr(cr_to_get)?;

        return Some(VirusLibrary::build_encounter(&viruses, num_viruses));
    }

    pub fn multi_cr_random_encounter(&self, low_cr: u8, high_cr: u8, num_viruses: usize) -> Option<Vec<&str>> {
        let mut viruses : Vec<&str> = vec![];
        for i in low_cr..=high_cr {
            let mut to_append = self.get_cr(i)?;
            viruses.append(&mut to_append);
        }
        if viruses.len() == 0 {return None;}
        return Some(VirusLibrary::build_encounter(&viruses, num_viruses));
    }

    #[inline]
    fn build_encounter<'a>(viruses: &Vec<&'a str>, num_viruses: usize) -> Vec<&'a str> {
        let mut rng = ThreadRng::default();
        let mut to_ret : Vec<&str>  = vec![];
        let vir_size = viruses.len();
        let distribution = Uniform::from(0..vir_size);
        for _ in 0..num_viruses {
            let index = distribution.sample(&mut rng);
            to_ret.push(viruses[index]);
        }
        return to_ret;
    }

}

impl Library for VirusLibrary {
    type LibObj = Virus;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Box<Virus>> {
        return &self.library;
    }
}

impl TypeMapKey for VirusLibrary {
    type Value = Arc<RwLock<VirusLibrary>>;
}

pub (crate) fn send_virus(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide a name");
        return;
    }
    let to_join = &args[1..];
    let to_search = to_join.join(" ");
    let data = ctx.data.read();
    let library_lock = data.get::<VirusLibrary>().expect("NCP library not found");
    let library = library_lock.read().expect("library was poisoned, panicking");
    search_lib_obj(ctx, msg, &to_search, library.deref());
}

pub (crate) fn send_virus_element(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide an element");
        return;
    }

    let data = ctx.data.read();
    let library_lock = data.get::<VirusLibrary>().expect("chip library not found");
    let library = library_lock.read().expect("chip library poisoned, panicking");
    let elem_res = library.search_element(args[1]);
    match elem_res {
        Some(elem) => long_say!(ctx, msg, elem),
        None => say!(ctx, msg, "nothing matched your search, are you sure you gave an element?"),
    }
}

pub (crate) fn send_virus_cr(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 2 {
        say!(ctx, msg, "you must provide a CR to search for");
        return;
    }
    let cr_to_get_res = args[1].trim().parse::<u8>();

    if cr_to_get_res.is_err() {
        say!(ctx, msg, "an invalid number was provided");
        return;
    }
    let cr_to_get = cr_to_get_res.unwrap();
    let data = ctx.data.read();
    let library_lock = data.get::<VirusLibrary>().expect("NCP library not found");
    let library = library_lock.read().expect("library was poisoned, panicking");
    match library.get_cr(cr_to_get) {
        Some(val) => long_say!(ctx, msg, val),
        None => say!(ctx, msg, "There are currently no viruses in that CR"),
    }
}

pub (crate) fn send_random_encounter(ctx: &Context, msg: &Message, args: &[&str]) {
    if args.len() < 3 {
        say!(ctx, msg, concat!(
        "You must send a CR and number of viruses; EX:\n",
        "```%encounter 2-3 5```",
        "This will return 5 random viruses in CR 2 & 3"
        ));
        return;
    }
    let virus_count = args[2].parse::<isize>().unwrap_or(-1);
    if virus_count <= 0 {
        say!(ctx, msg, "an invalid number of viruses were given");
        return;
    }
    let data = ctx.data.read();
    let library_lock = data.get::<VirusLibrary>().expect("NCP library not found");
    let library = library_lock.read().expect("library was poisoned, panicking");
    let single_cr_res = args[1].parse::<isize>();
    let to_send: Vec<&str>;

    //was it a single CR or a range?
    if single_cr_res.is_ok() {
        //a single CR
        let single_cr = single_cr_res.unwrap();
        if single_cr <= 0 || single_cr > library.get_highest_cr() as isize {
            say!(ctx, msg, "an invalid single CR was given");
            return;
        }
        to_send = library
            .single_cr_random_encounter(single_cr as u8, virus_count as usize)
                .expect("failed to get viruses");

    } else {
        let cr_range : Vec<&str> = args[1].trim().split('-').collect();
        if cr_range.len() != 2 {
            say!(ctx, msg, "That is an invalid CR range");
            return;
        }
        let first_cr_res = cr_range[0].parse::<u8>();
        let second_cr_res = cr_range[1].parse::<u8>();
        if first_cr_res.is_err() || second_cr_res.is_err() {
            say!(ctx, msg, "That is an invalid CR range");
            return;
        }
        let first_cr_num = first_cr_res.unwrap();
        let second_cr_num = second_cr_res.unwrap();
        if first_cr_num == second_cr_num {
            to_send = library
                .single_cr_random_encounter(
                    first_cr_num,
                    virus_count as usize
                ).expect("failed to get viruses");
        } else if first_cr_num > second_cr_num {
            to_send = library
                .multi_cr_random_encounter(
                    second_cr_num,
                    first_cr_num,
                    virus_count as usize
                ).expect("failed to get viruses");
        } else /* second_cr_num > first_cr_num */ {
            to_send = library
                .multi_cr_random_encounter(
                    first_cr_num,
                    second_cr_num,
                    virus_count as usize
                ).expect("failed to get viruses");
        }
    }
    long_say!(ctx, msg, to_send);

}