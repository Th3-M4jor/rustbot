use std::{cmp::Ordering, collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, RwLockReadGuard};

use rand::distributions::{Distribution, Uniform};

use rand::rngs::ThreadRng;

use serenity::{
    framework::standard::{macros::{command, group}, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use serde::{ser::SerializeMap, Serialize, Serializer};

#[cfg(not(debug_assertions))]
use serde_json;

use regex::Regex;

use crate::library::{battlechip::skills::Skills, elements::Elements, Library, LibraryObject};
use simple_error::SimpleError;
use std::fmt::Formatter;

use std::str::FromStr;
use unicode_normalization::UnicodeNormalization;

// const VIRUS_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1PZKYP0mzzxMTmjJ8CfrUMapgQPHgi24Ev6VB3XLBUrU&exportFormat=txt";

#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct Virus {
    pub name: String,
    pub element: Elements,
    pub skills: HashMap<Skills, u8>,
    pub h_p: usize,
    pub a_c: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abilities: Option<Vec<String>>,
    pub c_r: u8,
    pub mind: u8,
    pub body: u8,
    pub spirit: u8,
    pub drops: VirusDrops,
    pub description: String,
}

pub struct VirusDrops {
    pub table: Vec<(String, String)>,
}

impl Serialize for VirusDrops {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.table.len()))?;
        for drop in &self.table {
            map.serialize_entry(&drop.0, &drop.1)?;
        }
        map.end()
    }
}

impl VirusDrops {
    pub fn iter(&self) -> std::slice::Iter<(String, String)> {
        self.table.iter()
    }
}

impl LibraryObject for Virus {
    #[inline]
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_kind(&self) -> &str {
        "Virus"
    }

}

impl std::fmt::Display for Virus {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let first_line = format!("{} ({}) - CR {}", self.name, self.element, self.c_r);
        let second_line = format!("HP: {} | AC: {}", self.h_p, self.a_c);
        let third_line = format!(
            "Mind: {} | Body: {} | Spirit: {}",
            self.mind, self.body, self.spirit
        );

        let skill_line = self
            .skills
            .iter()
            .map(|skill| format!("{}: {}", skill.0, skill.1))
            .collect::<Vec<String>>()
            .join(" | ");
        let abilities_line = match &self.abilities {
            Some(abilities) => format!("Abilities: {}", abilities.join(", ")),
            None => String::from("Abilities: None"),
        };

        let mut drops_line = String::from("Drops: ");
        let drop_list = self
            .drops
            .table
            .iter()
            .map(|drop| format!("{}: {}", drop.0, drop.1))
            .collect::<Vec<String>>();
        drops_line.push_str(&drop_list.join(" | "));

        return write!(
            f,
            "```{}\n{}\n{}\n{}\n{}\n{}\n{}```",
            first_line,
            second_line,
            third_line,
            skill_line,
            abilities_line,
            drops_line,
            self.description
        );
    }
}

pub struct VirusLibrary {
    library: HashMap<String, Arc<Virus>>,
    highest_cr: u8,
    virus_url: String,
    virus_regex: Regex,
    cr_regex: Regex,
    hp_ac_regex: Regex,
    m_b_s_regex: Regex,
}

struct VirusSats {
    hp: usize,
    ac: usize,
    abilities: Option<Vec<String>>,
    skills: HashMap<Skills, u8>,
    mind: u8,
    body: u8,
    spirit: u8,
    drops: VirusDrops,
}

impl TypeMapKey for VirusLibrary {
    type Value = RwLock<VirusLibrary>;
}

impl Library for VirusLibrary {
    type LibObj = Arc<Virus>;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Arc<Virus>> {
        &self.library
    }
}

impl VirusLibrary {
    pub fn new(url: &str) -> VirusLibrary {
        VirusLibrary {
            library: HashMap::new(),
            highest_cr: 0,
            virus_url: String::from(url),
            virus_regex: Regex::new(r"^\s*(.+)\s+\((\w+)\)\s*$")
                .expect("could not compile virus regex"),
            cr_regex: Regex::new(r"^CR\s+(\d+)$").expect("could not compile CR regex"),
            hp_ac_regex: Regex::new(r"(?i)hp:\s+(\d+)\s+\|\s+ac:\s+(\d+)")
                .expect("could not compile HP regex"),
            m_b_s_regex: Regex::new(
                r"(?i)mind:\s+(\d+)\s+\|\s+body:\s+(\d+)\s+\|\sspirit:\s+(\d+)",
            )
            .expect("could not compile mbs regex"),
        }
    }

    pub async fn load_viruses(
        &mut self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.library.clear();
        // let virus_regex : Regex = Regex::new(r"((.+)\s\((\w+)\))").expect("could not compile virus regex");
        // let cr_regex : Regex = Regex::new(r"CR\s+(\d+)").expect("could not compile CR regex");

        // let mut virus_list : Vec<Box<Virus>> = vec![];
        let virus_text = reqwest::get(&self.virus_url)
            .await?
            .text()
            .await?
            .replace("\u{e2}\u{20ac}\u{2122}", "'")
            .replace("\u{FEFF}", "")
            .replace("\r", "");
        let virus_text_arr: Vec<&str> = virus_text
            .split('\n')
            .filter(|&i| !i.trim().is_empty())
            .collect();
        // let mut curr_cr: u8 = 0;
        let mut index: usize = 1;
        let mut premature_eof = None;
        let cr_cap = self
            .cr_regex
            .captures(virus_text_arr[0])
            .ok_or_else(|| SimpleError::new("Failed to capture first Virus CR"))?;
        let mut curr_cr = cr_cap[1]
            .parse::<u8>()
            .map_err(|_| SimpleError::new("Failed to parse first Virus CR"))?;

        while virus_text_arr.len() > index {
            if let Some(cr_cap) = self.cr_regex.captures(virus_text_arr[index]) {
                curr_cr = cr_cap[1].parse::<u8>().map_err(|_| {
                    SimpleError::new(format!(
                        "Could not parse \"{}\" into CR",
                        virus_text_arr[index]
                    ))
                })?;
                index += 1;
            }

            let name_res = self
                .virus_regex
                .captures(virus_text_arr[index])
                .ok_or_else(|| {
                    SimpleError::new(format!(
                        "Failed to parse virus name:\n{}",
                        virus_text_arr[index]
                    ))
                })?;
            let virus_name = &name_res[1];
            // println!("{}", virus_name);
            // println!("{}", &name_res[2]);
            let virus_element = name_res[2].parse::<Elements>().map_err(|_| {
                SimpleError::new(format!(
                    "Failed to parse virus element:\n{}",
                    virus_text_arr[index]
                ))
            })?;
            index += 1;
            let stat_chunk = if let Some(val) = virus_text_arr.get(index..=(index + 4)) {
                val
            } else {
                premature_eof = Some(Box::new(SimpleError::new(format!(
                    "Unexpected end of file reached while parsing {}",
                    virus_name
                ))));
                break;
            };
            let stat_res = self.parse_stats(stat_chunk).map_err(|e| {
                SimpleError::new(format!(
                    "Error at {}:\n{}\n{}",
                    virus_name,
                    e.as_str(),
                    virus_text_arr[index]
                ))
            })?;
            index += 5;
            let mut description = String::new();
            while virus_text_arr.len() > index
                && !self.cr_regex.is_match(virus_text_arr[index])
                && !self.virus_regex.is_match(virus_text_arr[index])
            {
                description.push_str(virus_text_arr[index]);
                index += 1;
            }
            let virus = Arc::new(Virus {
                name: virus_name.nfc().collect(),
                h_p: stat_res.hp,
                a_c: stat_res.ac,
                c_r: curr_cr,
                element: virus_element.to_owned(),
                mind: stat_res.mind,
                body: stat_res.body,
                spirit: stat_res.spirit,
                skills: stat_res.skills,
                abilities: stat_res.abilities,
                drops: stat_res.drops,
                description,
            });

            if self
                .library
                .insert(virus.name.to_ascii_lowercase(), virus)
                .is_some()
            {
                return Err(Box::new(SimpleError::new(format!(
                    "Duplicate virus name found: {}",
                    virus_name
                ))));
            }
            //yield to other tasks on each iteration
            tokio::task::yield_now().await;
        }

        self.highest_cr = curr_cr;

        #[cfg(not(debug_assertions))]
        {
            let mut viruses: Vec<&Arc<Virus>> = self.library.values().collect();
            viruses.sort_unstable_by(|a, b| a.c_r.cmp(&b.c_r).then_with(|| a.name.cmp(&b.name)));

            let j = tokio::task::block_in_place(|| serde_json::to_string(&viruses))
                .expect("could not serialize virus library to JSON");
            tokio::fs::write("./virusCompendium.json", j)
                .await
                .expect("could not write to virusCompendium.json");
        }

        if let Some(err) = premature_eof {
            Err(err)
        } else {
            Ok(format!("{} viruses were loaded\n", self.library.len()))
        }
    }

    fn parse_stats(&self, lines: &[&str]) -> Result<VirusSats, SimpleError> {
        if lines.len() < 4 {
            return Err(SimpleError::new("unexpected end of file"));
        }

        // let HP_AC_Regex = Regex::new(r"(?i)hp:\s+(\d+)\s+\|\s+ac:\s+(\d+)").expect("could not compile HP regex");
        // let m_b_s_Regex = Regex::new(r"(?i)mind:\s+(\d+)\s+\|\s+body:\s+(\d+)\s+\|\sspirit:\s+(\d+)").expect("could not compile mbs regex");

        let hp_ac_res = self
            .hp_ac_regex
            .captures(lines[0])
            .ok_or_else(|| SimpleError::new("Failed to parse HP or AC"))?;

        let hp: usize = hp_ac_res[1]
            .parse::<usize>()
            .map_err(|_| SimpleError::new("Failed to parse HP"))?;
        let ac: usize = hp_ac_res[2]
            .parse::<usize>()
            .map_err(|_| SimpleError::new("Failed to parse AC"))?;

        let m_b_s_res = self
            .m_b_s_regex
            .captures(lines[1])
            .ok_or_else(|| SimpleError::new("Failed to parse Mind, Body, or Spirit"))?;

        let mind = m_b_s_res[1]
            .parse::<u8>()
            .map_err(|_| SimpleError::new("Failed to parse virus Mind stat"))?;
        let body = m_b_s_res[2]
            .parse::<u8>()
            .map_err(|_| SimpleError::new("Failed to parse virus Body stat"))?;
        let spirit = m_b_s_res[3]
            .parse::<u8>()
            .map_err(|_| SimpleError::new("Failed to parse virus Spirit stat"))?;

        let skills = VirusLibrary::convert_skills(lines[2])?;
        let abilities = VirusLibrary::convert_abilities(lines[3])?;
        let drops = VirusLibrary::convert_drops(lines[4])?;

        Ok(VirusSats {
            hp,
            ac,
            mind,
            body,
            spirit,
            skills,
            abilities,
            drops,
        })
    }

    fn convert_skills(line: &str) -> Result<HashMap<Skills, u8>, SimpleError> {
        let mut to_ret: HashMap<Skills, u8> = HashMap::new();
        for skill in line.split('|') {
            let name_skill = skill.trim().split(':').collect::<Vec<&str>>();
            if name_skill.len() != 2 {
                return Err(SimpleError::new(format!(
                    "Failed to parse skill:\n{}",
                    skill
                )));
            }
            let name = name_skill[0]
                .trim()
                .parse::<Skills>()
                .map_err(|_| SimpleError::new(format!("Failed to parse skill:\n{}", skill)))?;
            let value = name_skill[1]
                .trim()
                .parse::<u8>()
                .map_err(|_| SimpleError::new(format!("Failed to parse skill:\n{}", skill)))?;
            if to_ret.insert(name, value).is_some() {
                return Err(SimpleError::new(format!("Failed to parse:\n{}", skill)));
            }
        }

        Ok(to_ret)
    }

    fn convert_drops(line: &str) -> Result<VirusDrops, SimpleError> {
        let mut table: Vec<(String, String)> = Vec::new();
        let drop_line = line.splitn(2, ':').collect::<Vec<&str>>();
        if drop_line.len() != 2 {
            return Err(SimpleError::new(format!(
                "Failed to parse drops:\n{}",
                line
            )));
        }
        for drops in drop_line[1].split('|') {
            let drop = drops.trim().split(':').collect::<Vec<&str>>();
            if drop.len() != 2 {
                return Err(SimpleError::new(format!(
                    "Failed to parse drops:\n{}",
                    drops
                )));
            }
            let range = drop[0].trim().to_string();
            let value = drop[1].trim().to_string();
            table.push((range, value));
        }
        Ok(VirusDrops { table })
    }

    fn convert_abilities(line: &str) -> Result<Option<Vec<String>>, SimpleError> {
        let abilities = line.split(':').collect::<Vec<&str>>();
        if abilities.len() != 2 {
            return Err(SimpleError::new(format!(
                "Failed to parse ability:\n{}",
                line
            )));
        }

        if abilities[0].trim().to_ascii_lowercase() != "abilities" {
            return Err(SimpleError::new(format!(
                "Failed to parse ability:\n{}",
                line
            )));
        }

        let abilities_list = abilities[1].trim().split(',').collect::<Vec<&str>>();
        if abilities_list.len() == 1 && abilities_list[0].trim().to_ascii_lowercase() == "none" {
            return Ok(None);
        }

        Ok(Some(
            abilities_list
                .iter()
                .map(|a| a.trim().to_owned())
                .collect::<Vec<String>>(),
        ))
    }

    pub fn get_cr(&self, cr_to_get: u8) -> Option<Vec<&Arc<Virus>>> {
        if cr_to_get > self.highest_cr {
            return None;
        }
        self.search_any(cr_to_get, |a, b| a.c_r == b)
    }

    pub fn search_element(&self, elem: &str) -> Option<Vec<&Arc<Virus>>> {
        let elem_to_get = Elements::from_str(elem).ok()?;

        self.search_any(elem_to_get, |a, b| a.element == b)
    }

    pub fn get_highest_cr(&self) -> u8 {
        self.highest_cr
    }

    pub fn single_cr_random_encounter(
        &self,
        cr_to_get: u8,
        num_viruses: usize,
    ) -> Option<Vec<String>> {
        let viruses = self.get_cr(cr_to_get)?;

        Some(VirusLibrary::build_encounter(&viruses, num_viruses))
    }

    pub fn multi_cr_random_encounter(
        &self,
        low_cr: u8,
        high_cr: u8,
        num_viruses: usize,
    ) -> Option<Vec<String>> {
        let mut viruses = vec![];
        for i in low_cr..=high_cr {
            let mut to_append = self.get_cr(i)?;
            viruses.append(&mut to_append);
        }
        if viruses.is_empty() {
            return None;
        }
        Some(VirusLibrary::build_encounter(&viruses, num_viruses))
    }

    #[inline]
    fn build_encounter(viruses: &[&Arc<Virus>], num_viruses: usize) -> Vec<String> {
        let mut rng = ThreadRng::default();
        let mut to_ret: Vec<String> = vec![];
        let vir_size = viruses.len();
        let distribution = Uniform::from(0..vir_size);
        for _ in 0..num_viruses {
            let index = distribution.sample(&mut rng);
            to_ret.push(viruses[index].get_name().to_string());
        }
        to_ret
    }

    fn get_family(&self, name: &str) -> Option<Vec<&Arc<Virus>>> {
        self.get(&name.to_lowercase())?;
        let mut viruses = self.name_contains(name, Some(usize::max_value()))?;
        if viruses.len() == 1 {
            return Some(viruses);
        }
        viruses.sort_unstable_by(move |a, b| {
            let a_val = self.get(a.get_name()).unwrap();
            let b_val = self.get(b.get_name()).unwrap();
            a_val
                .c_r
                .cmp(&b_val.c_r)
                .then(a_val.element.cmp(&b_val.element))
        });
        Some(viruses)
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
/// A group of commands related to viruses, see `v virus` for the get virus command help
struct BnbViruses;

#[command("virus")]
/// Get the description of the virus with that name, or suggestions if a virus with that name does not exist
#[example = "Mettaur"]
pub(crate) async fn send_virus(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    }
    let to_search = args.rest();
    let data = ctx.data.read().await;
    let library_lock = data.get::<VirusLibrary>().expect("Virus library not found");
    let library = library_lock.read().await;
    //.expect("library was poisoned, panicking");
    library.reaction_name_search(ctx, msg, to_search).await;
    // say!(ctx, msg, search_lib_obj(&to_search, library));
    Ok(())
}

#[command("element")]
/// Get a list of all viruses which are of the given element
#[example = "Elec"]
pub(crate) async fn send_virus_element(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
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
        Some(elem) => {
            let to_send = elem.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(
            ctx,
            msg,
            "nothing matched your search, are you sure you gave an element?"
        ),
    }
    return Ok(());
}

#[command("cr")]
/// Get a list of all viruses which are of the given CR
#[example = "4"]
pub(crate) async fn send_virus_cr(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
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
        Some(val) => {
            let to_send = val.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(ctx, msg, "There are currently no viruses in that CR"),
    }
    return Ok(());
}

#[command("encounter")]
/// Builds a random encounter with a given number of viruses and within a given CR or CR range
#[example = "2-3 5"]
#[example = "4 6"]
pub(crate) async fn send_random_encounter(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    if args.len() < 2 {
        say!(
            ctx,
            msg,
            concat!(
                "You must send a CR and number of viruses; EX:\n",
                "```%virus encounter 2-3 5```",
                "This will return 5 random viruses in CR 2 & 3"
            )
        );
        return Ok(());
    }
    let first_arg = args.single::<String>().unwrap();
    // args.advance();
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
    let to_send: Vec<String>;

    // was it a single CR or a range?
    if let Ok(single_cr) = single_cr_res {
        // a single CR
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

        to_send = match first_cr_num.cmp(&second_cr_num) {
            Ordering::Equal => library
                .single_cr_random_encounter(first_cr_num, virus_count as usize)
                .expect("failed to get viruses"),
            Ordering::Greater => library
                .multi_cr_random_encounter(second_cr_num, first_cr_num, virus_count as usize)
                .expect("failed to get viruses"),
            Ordering::Less => library
                .multi_cr_random_encounter(first_cr_num, second_cr_num, virus_count as usize)
                .expect("failed to get viruses"),
        };
    }
    long_say!(ctx, msg, to_send, ", ");
    return Ok(());
}

pub(crate) fn virus_as_lib_obj(obj: Arc<Virus>) -> Arc<dyn LibraryObject> {
    obj
}

#[command("family")]
/// Lists all viruses who are determined to be of a particular family, given the name of the
/// first virus in it\nNote: Only guaranteed to work if they follow the 2 3 EX scheme
#[example = "Swordy"]
pub(crate) async fn send_family(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    }
    // let to_join = &args[1..];
    let to_search = args.rest();
    let data = ctx.data.read().await;
    let library_lock: &RwLock<VirusLibrary> =
        data.get::<VirusLibrary>().expect("Virus library not found");
    let library = library_lock.read().await;
    //.expect("library was poisoned, panicking");
    match library.get_family(&to_search) {
        Some(res) => {
            let to_send = res.iter().map(|val| (*val).get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(ctx, msg, "There is no family under that name"),
    }
    return Ok(());
}
