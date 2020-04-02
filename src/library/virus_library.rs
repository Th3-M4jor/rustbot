use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};

use rand::distributions::{Distribution, Uniform};

use rand::rngs::ThreadRng;

use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::{model::channel::Message, prelude::*};

use serde::Serialize;

#[cfg(not(debug_assertions))]
use serde_json;
#[cfg(not(debug_assertions))]
use tokio::fs;

use regex::Regex;

use crate::library::{elements::Elements, search_lib_obj, Library, LibraryObject};
use simple_error::SimpleError;
use std::fmt::Formatter;

use std::str::FromStr;
use unicode_normalization::UnicodeNormalization;

//const VIRUS_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1PZKYP0mzzxMTmjJ8CfrUMapgQPHgi24Ev6VB3XLBUrU&exportFormat=txt";

#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Virus {
    pub name: String,
    pub element: Elements,
    pub c_r: u8,
    pub description: String,
}

impl Virus {
    pub fn new<T: Into<String>, S: Into<u8>>(name: T, elem: Elements, cr: S, desc: T) -> Virus {
        Virus {
            name: name.into().nfc().collect::<String>(),
            element: elem,
            c_r: cr.into(),
            description: desc.into().nfc().collect::<String>(),
        }
    }
}

impl LibraryObject for Virus {
    #[inline]
    fn get_name(&self) -> &str {
        return &self.name;
    }
}

impl std::fmt::Display for Virus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(
            f,
            "```{} ({}) - CR {}\n{}```",
            self.name, self.element, self.c_r, self.description
        );
    }
}

pub struct VirusLibrary {
    library: HashMap<String, Arc<Virus>>,
    highest_cr: u8,
    duplicates: Vec<String>,
    virus_url: String,
}

impl TypeMapKey for VirusLibrary {
    type Value = RwLock<VirusLibrary>;
}

impl Library for VirusLibrary {
    type LibObj = Arc<Virus>;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Arc<Virus>> {
        return &self.library;
    }
}

impl VirusLibrary {
    pub fn new(url: &str) -> VirusLibrary {
        VirusLibrary {
            library: HashMap::new(),
            highest_cr: 0,
            duplicates: vec![],
            virus_url: String::from(url),
        }
    }

    pub async fn load_viruses(
        &mut self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        lazy_static! {
            static ref VIRUS_REGEX: Regex =
                Regex::new(r"^\s*((.+)\s+\((\w+)\))\s*$").expect("could not compile virus regex");
            static ref CR_REGEX: Regex =
                Regex::new(r"^CR\s+(\d+)$").expect("could not compile CR regex");
        }

        self.library.clear();
        //let virus_regex : Regex = Regex::new(r"((.+)\s\((\w+)\))").expect("could not compile virus regex");
        //let cr_regex : Regex = Regex::new(r"CR\s+(\d+)").expect("could not compile CR regex");

        //let mut virus_list : Vec<Box<Virus>> = vec![];
        self.duplicates.clear();
        let virus_text = reqwest::get(&self.virus_url)
            .await?
            .text()
            .await?
            .replace("â€™", "'")
            .replace("\u{FEFF}", "")
            .replace("\r", "");
        let virus_text_arr: Vec<&str> = virus_text
            .split("\n")
            .filter(|&i| !i.trim().is_empty())
            .collect();
        let mut curr_cr: u8 = 0;
        let mut current_virus_name = String::new();
        let mut current_virus_element: Elements = Elements::Null;
        let mut current_virus_full_name = String::new();
        let mut current_virus_description = String::new();
        let mut found_duplicates = false;
        for i in 0..virus_text_arr.len() {
            let cr_cap = CR_REGEX.captures(virus_text_arr[i]);
            let virus_cap = VIRUS_REGEX.captures(virus_text_arr[i]);
            if (cr_cap.is_some() || virus_cap.is_some() || (i + 1) == virus_text_arr.len())
                && !current_virus_description.is_empty()
            {
                if (i + 1) == virus_text_arr.len() && !(cr_cap.is_some() || virus_cap.is_some()) {
                    //push last line
                    current_virus_description.push_str(virus_text_arr[i]);
                }

                let add_res;
                if self.duplicates.contains(&current_virus_name.to_lowercase()) {
                    add_res = self.library.insert(
                        current_virus_full_name.to_lowercase(),
                        Arc::new(Virus::new(
                            &current_virus_full_name,
                            current_virus_element,
                            curr_cr,
                            &current_virus_description,
                        )),
                    );
                } else {
                    add_res = self.library.insert(
                        current_virus_name.to_lowercase(),
                        Arc::new(Virus::new(
                            &current_virus_name,
                            current_virus_element,
                            curr_cr,
                            &current_virus_description,
                        )),
                    );
                }
                if add_res.is_some() {
                    //println!("{} , {} , {}\n{}", current_virus_name, current_virus_full_name, current_virus_element, current_virus_description);
                    //found a duplicate, fixing
                    //let add_res_val = add_res.unwrap();
                    let old_virus = add_res.unwrap();
                    let new_virus = self
                        .library
                        .remove(&current_virus_name.to_lowercase())
                        .ok_or(SimpleError::new(format!(
                            "found an unrecoverable duplicate, {} 119 {:?}",
                            current_virus_full_name, self.duplicates
                        )))?;
                    //let new_virus = new_virus_res.unwrap();

                    if new_virus.element == old_virus.element {
                        return Err(Box::new(SimpleError::new(format!(
                            "found an unrecoverable duplicate, {}, 124",
                            current_virus_full_name
                        ))));
                    }
                    let repl_new_virus = Arc::new(Virus::new(
                        format!("{} ({})", new_virus.name, new_virus.element),
                        new_virus.element,
                        new_virus.c_r,
                        new_virus.description.clone(),
                    ));
                    let repl_old_virus = Arc::new(Virus::new(
                        format!("{} ({})", old_virus.name, old_virus.element),
                        old_virus.element,
                        old_virus.c_r,
                        old_virus.description.clone(),
                    ));
                    let new_add_res = self
                        .library
                        .insert(repl_new_virus.name.to_lowercase(), repl_new_virus);
                    let old_add_res = self
                        .library
                        .insert(repl_old_virus.name.to_lowercase(), repl_old_virus);
                    if new_add_res.is_some() || old_add_res.is_some() {
                        return Err(Box::new(SimpleError::new(format!(
                            "found an unrecoverable duplicate, {}, 131",
                            current_virus_full_name
                        ))));
                    }
                    self.duplicates.push(current_virus_name.to_lowercase());
                    found_duplicates = true;
                }
                current_virus_name.clear();
                current_virus_full_name.clear();
                current_virus_description.clear();
            }
            if cr_cap.is_some() {
                let cr_val = cr_cap.unwrap();
                curr_cr = cr_val
                    .get(1)
                    .unwrap()
                    .as_str()
                    .parse::<u8>()
                    .unwrap_or(u8::max_value());
            } else if virus_cap.is_some() {
                let virus_val = virus_cap.unwrap();
                current_virus_full_name.push_str(virus_val.get(1).unwrap().as_str());
                current_virus_name.push_str(virus_val.get(2).unwrap().as_str());
                current_virus_element =
                    Elements::from_str(virus_val.get(3).expect("Virus had no element").as_str())
                        .expect("could not parse element");
            //current_virus_full_name.push_str(virus_val.get(3).unwrap().as_str());
            } else {
                current_virus_description.push_str(virus_text_arr[i]);
                current_virus_description.push('\n');
            }
        }
        self.highest_cr = curr_cr;

        //only write json file if not debug
        #[cfg(not(debug_assertions))]
        {
            let mut viruses: Vec<&Arc<Virus>> = self.library.values().collect();
            viruses.sort_unstable_by(|a, b| a.c_r.cmp(&b.c_r).then_with(|| a.name.cmp(&b.name)));

            let j = serde_json::to_string_pretty(&viruses)
                .expect("could not serialize virus library to JSON");
            fs::write("./virusCompendium.json", j)
                .await
                .expect("could not write to virusCompendium.json");
        }

        if found_duplicates {
            let res = format!(
                "{} viruses loaded, found {} duplicates, recovered from all\nThey were: {:?}",
                self.library.len(),
                self.duplicates.len(),
                self.duplicates
            );
            return Ok(res);
        } else {
            return Ok(format!("{} viruses were loaded\n", self.library.len()));
        }
    }

    pub fn get_cr(&self, cr_to_get: u8) -> Option<Vec<&str>> {
        if cr_to_get > self.highest_cr {
            return None;
        }
        return self.search_any(cr_to_get, |a, b| a.c_r == b);
    }

    pub fn search_element(&self, elem: &str) -> Option<Vec<&str>> {
        let elem_to_get = Elements::from_str(elem).ok()?;

        return self.search_any(elem_to_get, |a, b| a.element == b);
    }

    pub fn get_highest_cr(&self) -> u8 {
        return self.highest_cr;
    }

    pub fn single_cr_random_encounter(
        &self,
        cr_to_get: u8,
        num_viruses: usize,
    ) -> Option<Vec<&str>> {
        let viruses = self.get_cr(cr_to_get)?;

        return Some(VirusLibrary::build_encounter(&viruses, num_viruses));
    }

    pub fn multi_cr_random_encounter(
        &self,
        low_cr: u8,
        high_cr: u8,
        num_viruses: usize,
    ) -> Option<Vec<&str>> {
        let mut viruses: Vec<&str> = vec![];
        for i in low_cr..=high_cr {
            let mut to_append = self.get_cr(i)?;
            viruses.append(&mut to_append);
        }
        if viruses.len() == 0 {
            return None;
        }
        return Some(VirusLibrary::build_encounter(&viruses, num_viruses));
    }

    #[inline]
    fn build_encounter<'a>(viruses: &Vec<&'a str>, num_viruses: usize) -> Vec<&'a str> {
        let mut rng = ThreadRng::default();
        let mut to_ret: Vec<&str> = vec![];
        let vir_size = viruses.len();
        let distribution = Uniform::from(0..vir_size);
        for _ in 0..num_viruses {
            let index = distribution.sample(&mut rng);
            to_ret.push(viruses[index]);
        }
        return to_ret;
    }

    fn get_family(&self, name: &str) -> Option<Vec<&str>> {
        if self.get(&name.to_lowercase()).is_none()
            && !self.duplicates.contains(&name.to_lowercase())
        {
            return None;
        }
        let mut viruses = self.name_contains(name, Some(usize::max_value()))?;
        if viruses.len() == 1 {
            return Some(viruses);
        }
        viruses.sort_unstable_by(move |a, b| {
            let a_val = self.get(&a.to_lowercase()).unwrap();
            let b_val = self.get(&b.to_lowercase()).unwrap();
            return a_val
                .c_r
                .cmp(&b_val.c_r)
                .then(a_val.element.cmp(&b_val.element));
        });
        return Some(viruses);
    }
}

#[group]
#[prefixes("v", "virus")]
#[default_command(send_virus)]
#[commands(
    send_virus,
    send_virus_element,
    send_virus_cr,
    send_random_encounter,
    send_family
)]
#[description(
    "A group of commands related to viruses, see `v virus` for the get virus command help"
)]
struct BnbViruses;

#[command("virus")]
#[description("Get the description of the virus with that name, or suggestions if a virus with that name does not exist")]
#[example = "Mettaur"]
pub(crate) async fn send_virus(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    }
    let to_search = args.rest();
    let data = ctx.data.read().await;
    let library_lock = data.get::<VirusLibrary>().expect("Virus library not found");
    let library = library_lock.read().await;
    //.expect("library was poisoned, panicking");

    match search_lib_obj(to_search, &library) {
        Ok(val) => say!(ctx, msg, val),
        Err(val) => say!(ctx, msg, format!("Did you mean: {}", val.join(", "))),
    }
    //say!(ctx, msg, search_lib_obj(&to_search, library));
    return Ok(());
}

#[command("element")]
#[description("Get a list of all viruses which are of the given element")]
#[example = "Elec"]
pub(crate) async fn send_virus_element(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide an element");
        return Ok(());
    }

    let data = ctx.data.read().await;
    let library_lock: &RwLock<VirusLibrary> =
        data.get::<VirusLibrary>().expect("Virus library not found");
    let library = library_lock.read().await;
    //.expect("Virus library poisoned, panicking");
    let elem_res = library.search_element(args.current().unwrap());
    match elem_res {
        Some(elem) => long_say!(ctx, msg, elem, ", "),
        None => say!(
            ctx,
            msg,
            "nothing matched your search, are you sure you gave an element?"
        ),
    }
    return Ok(());
}

#[command("cr")]
#[description("Get a list of all viruses which are of the given CR")]
#[example = "4"]
pub(crate) async fn send_virus_cr(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide a CR to search for");
        return Ok(());
    }

    let cr_to_get_res = args.single::<u8>();

    if cr_to_get_res.is_err() {
        say!(ctx, msg, "an invalid number was provided");
        return Ok(());
    }
    let cr_to_get = cr_to_get_res.unwrap();
    let data = ctx.data.read().await;
    let library_lock: &RwLock<VirusLibrary> =
        data.get::<VirusLibrary>().expect("Virus library not found");
    let library = library_lock.read().await;
    //.expect("library was poisoned, panicking");
    match library.get_cr(cr_to_get) {
        Some(val) => long_say!(ctx, msg, val, ", "),
        None => say!(ctx, msg, "There are currently no viruses in that CR"),
    }
    return Ok(());
}

#[command("encounter")]
#[description(
    "Builds a random encounter with a given number of viruses and within a given CR or CR range"
)]
#[example = "2-3 5"]
#[example = "4 6"]
pub(crate) async fn send_random_encounter(
    ctx: &mut Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    if args.len() < 2 {
        say!(
            ctx,
            msg,
            concat!(
                "You must send a CR and number of viruses; EX:\n",
                "```%encounter 2-3 5```",
                "This will return 5 random viruses in CR 2 & 3"
            )
        );
        return Ok(());
    }
    let first_arg = args.single::<String>().unwrap();
    //args.advance();
    let second_arg = args.single::<String>().unwrap();
    let virus_count = second_arg.parse::<isize>().unwrap_or(-1);
    if virus_count <= 0 {
        say!(ctx, msg, "an invalid number of viruses were given");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let library_lock: &RwLock<VirusLibrary> =
        data.get::<VirusLibrary>().expect("Virus library not found");
    let library: RwLockReadGuard<VirusLibrary> = library_lock.read().await;
    //.expect("library was poisoned, panicking");
    let single_cr_res = first_arg.parse::<isize>();
    let to_send: Vec<&str>;

    //was it a single CR or a range?
    if single_cr_res.is_ok() {
        //a single CR
        let single_cr = single_cr_res.unwrap();
        if single_cr <= 0 || single_cr > library.get_highest_cr() as isize {
            say!(ctx, msg, "an invalid single CR was given");
            return Ok(());
        }
        to_send = library
            .single_cr_random_encounter(single_cr as u8, virus_count as usize)
            .expect("failed to get viruses");
    } else {
        let cr_range: Vec<&str> = first_arg.trim().split('-').collect();
        if cr_range.len() != 2 {
            say!(ctx, msg, "That is an invalid CR range");
            return Ok(());
        }
        let first_cr_res = cr_range[0].parse::<u8>();
        let second_cr_res = cr_range[1].parse::<u8>();
        if first_cr_res.is_err() || second_cr_res.is_err() {
            say!(ctx, msg, "That is an invalid CR range");
            return Ok(());
        }
        let first_cr_num = first_cr_res.unwrap();
        let second_cr_num = second_cr_res.unwrap();
        if first_cr_num == second_cr_num {
            to_send = library
                .single_cr_random_encounter(first_cr_num, virus_count as usize)
                .expect("failed to get viruses");
        } else if first_cr_num > second_cr_num {
            to_send = library
                .multi_cr_random_encounter(second_cr_num, first_cr_num, virus_count as usize)
                .expect("failed to get viruses");
        } else
        /* second_cr_num > first_cr_num */
        {
            to_send = library
                .multi_cr_random_encounter(first_cr_num, second_cr_num, virus_count as usize)
                .expect("failed to get viruses");
        }
    }
    long_say!(ctx, msg, to_send, ", ");
    return Ok(());
}

#[command("family")]
#[description("Lists all viruses who are determined to be of a particular family, given the name of the first virus in it\nNote: Only guaranteed to work if they follow the 2 3 EX scheme")]
#[example = "Swordy"]
pub(crate) async fn send_family(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    }
    //let to_join = &args[1..];
    let to_search = args.rest();
    let data = ctx.data.read().await;
    let library_lock: &RwLock<VirusLibrary> =
        data.get::<VirusLibrary>().expect("Virus library not found");
    let library = library_lock.read().await;
    //.expect("library was poisoned, panicking");
    match library.get_family(&to_search) {
        Some(res) => long_say!(ctx, msg, res, ", "),
        None => say!(ctx, msg, "There is no family under that name"),
    }
    return Ok(());
}
