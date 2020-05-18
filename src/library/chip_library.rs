#[cfg(not(debug_assertions))]
use serde_json;
use serenity::{
    framework::standard::{macros::{command, group}, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, RwLockReadGuard};

use simple_error::SimpleError;

use crate::{
    library::{
        battlechip::{skills::Skills, BattleChip},
        elements::Elements,
        Library,
        virus_library::VirusLibrary,
    },
    LibraryObject,
};
use std::borrow::BorrowMut;

use std::str::FromStr;

// const CHIP_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1lvAKkymOplIJj6jS-N5__9aLIDXI6bETIMz01MK9MfY&exportFormat=txt";

pub struct ChipLibrary {
    chips: HashMap<String, Arc<BattleChip>>,
    chip_url: String,
    custom_chip_url: String,
}

impl Library for ChipLibrary {
    type LibObj = Arc<BattleChip>;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Arc<BattleChip>> {
        &self.chips
    }
}

impl ChipLibrary {
    pub fn new(url: &str, custom_url: &str) -> ChipLibrary {
        ChipLibrary {
            chips: HashMap::new(),
            chip_url: String::from(url),
            custom_chip_url: String::from(custom_url),
        }
    }

    // returns number of chips loaded or a simple error
    pub async fn load_chips(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.chips.clear();

        let chip_text_future = ChipLibrary::get_chip_text(&self.chip_url);
        let custom_chip_text_future = ChipLibrary::get_custom_chip_text(&self.custom_chip_url);

        let (chip_text_res, special_chips_res) =
            tokio::join!(chip_text_future, custom_chip_text_future);

        let chip_text = chip_text_res?;
        self.chips = tokio::task::spawn_blocking(move || {
            let mut chip_text_arr: Vec<&str> = chip_text
            .split('\n')
            .filter(|&i| !i.trim().is_empty())
            .collect();
        let special_chip_text;
        if special_chips_res.is_some() {
            special_chip_text = special_chips_res.unwrap();
            let mut special_chip_arr: Vec<&str> = special_chip_text
                .split('\n')
                .filter(|&i| !i.trim().is_empty())
                .collect();
            chip_text_arr.append(special_chip_arr.borrow_mut());
        }
            let mut chips: Vec<BattleChip> = vec![];
            for val in chip_text_arr
                .iter()
                .step_by(2)
                .zip(chip_text_arr.iter().skip(1).step_by(2))
            {
                let first_line = val.0;
                let second_line = val.1;
                let chip = match BattleChip::from_chip_string(first_line, second_line) {
                    Ok(val) => val,
                    Err(_) => {
                        return Err(SimpleError::new(format!(
                            "Found an invalid chip:\n{}",
                            first_line
                        )))
                    }
                };
                chips.push(chip);
            }
            chips.shrink_to_fit();
            chips.sort_unstable();

            // only write json file if not debug
            #[cfg(not(debug_assertions))]
            {
                let j = serde_json::to_string_pretty(&chips).expect("could not serialize to json");
                std::fs::write("chips.json", j).expect("could not write to chips.json");
            }
            let mut new_chips = HashMap::new();
            while !chips.is_empty() {
                let chip = chips.pop().expect("Something went wrong popping a chip");
                new_chips.insert(chip.name.to_lowercase(), Arc::new(chip));
            }
            Ok(new_chips)
        }).await??;

        Ok(self.chips.len())
    }

    async fn get_chip_text(url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let res = reqwest::get(url)
            .await?
            .text()
            .await?
            .replace("\u{e2}\u{20ac}\u{2122}", "'")
            .replace("\u{FEFF}", "")
            .replace("\r", "");
        Ok(res)
    }

    async fn get_custom_chip_text(url: &str) -> Option<String> {
        let special_chips_res = reqwest::get(url).await.ok()?;
        let res = special_chips_res
            .text()
            .await
            .ok()?
            .replace("\u{e2}\u{20ac}\u{2122}", "'")
            .replace("\u{FEFF}", "")
            .replace("\r", "");
        Some(res)
    }

    pub fn search_element(&self, to_get: &str) -> Option<Vec<&Arc<BattleChip>>> {
        // let elem_res = Elements::from_str(to_get);

        let elem_to_get = Elements::from_str(to_get).ok()?;

        self.search_any(elem_to_get, |a, b| a.element.contains(&b))
    }

    pub fn search_skill(&self, to_get: &str) -> Option<Vec<&Arc<BattleChip>>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        // special case for where you want chips with more than one possible skill
        if skill_to_get == Skills::Varies {
            self.search_any(skill_to_get, |a, _| a.skills.len() > 1)
        } else {
            self.search_any(skill_to_get, |a, b| a.skills.contains(&b))
        }
    }

    pub fn search_skill_target(&self, to_get: &str) -> Option<Vec<&Arc<BattleChip>>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        self.search_any(skill_to_get, |a, b| a.skill_target == b)
    }

    pub fn search_skill_user(&self, to_get: &str) -> Option<Vec<&Arc<BattleChip>>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        self.search_any(skill_to_get, |a, b| a.skill_user == b)
    }

    pub fn search_skill_check(&self, to_get: &str) -> Option<Vec<&Arc<BattleChip>>> {
        // return search_skill_spec!(val.SkillTarget == skill_to_get || val.SkillUser == skill_to_get);
        let skill_to_get = Skills::from_str(to_get).ok()?;

        self.search_any(skill_to_get, |a, b| {
            a.skill_target == b || a.skill_user == b
        })
    }
}

impl TypeMapKey for ChipLibrary {
    type Value = RwLock<ChipLibrary>;
}

pub(crate) fn battlechip_as_lib_obj(obj: Arc<BattleChip>) -> Arc<dyn LibraryObject> {
    obj
}

#[group]
#[prefixes("c", "chip")]
#[default_command(send_chip)]
#[commands(send_chip, send_chip_element, chip_drop_cr)]
/// A group of commands related to Navi-Customizer Parts, see `c chip` for the get chip command help
struct BnbChips;

#[group]
#[prefixes("s", "skill")]
#[default_command(send_chip_skill)]
#[commands(
    send_chip_skill,
    send_chip_skill_user,
    send_chip_skill_target,
    send_chip_skill_check
)]
/// A group of commands related to Battlechip skills, see `s skill` for the get chip by skill help
struct BnBSkills;

#[command("chip")]
/// get the description of a chip with the specified name, or suggestions if there is not a chip with that name
#[example = "Airshot"]
async fn send_chip(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    };

    let to_get = args.current().unwrap();
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library = library_lock.read().await;

    library.reaction_name_search(ctx, msg, to_get).await;
    Ok(())
}

#[command("skill")]
/// get a list of chips that use the specified skill in it's attack roll
#[example = "Sense"]
async fn send_chip_skill(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill(&skill) {
        Some(chips) => {
            let to_send = chips.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command("user")]
/// get a list of chips that have a save and the DC is determined by the specified skill
#[example = "Strength"]
async fn send_chip_skill_user(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill_user(&skill) {
        Some(chips) => {
            let to_send = chips.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command("target")]
/// get a list of chips where the specified skill is used to make the save by the target
#[example = "Speed"]
async fn send_chip_skill_target(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill_target(&skill) {
        Some(chips) => {
            let to_send = chips.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(ctx, msg, "nothing matched your search"),
    }
    Ok(())
}

#[command("check")]
/// get a list of chips where the specified skill is used either to determine the save DC or to make the save
#[example = "Bravery"]
async fn send_chip_skill_check(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill_check(&skill) {
        Some(chips) => {
            let to_send = chips.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(ctx, msg, "nothing matched your search"),
    }
    Ok(())
}

#[command("element")]
/// get a list of chips which are of the specified element
#[example = "Aqua"]
async fn send_chip_element(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide an element");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library = library_lock.read().await;
    //.expect("chip library poisoned, panicking");
    let elem_res = library.search_element(args.rest());

    match elem_res {
        Some(chips) => {
            let to_send = chips.iter().map(|a| a.get_name()).collect::<Vec<&str>>();
            long_say!(ctx, msg, to_send, ", ")
        },
        None => say!(
            ctx,
            msg,
            "nothing matched your search, are you sure you gave an element?"
        ),
    }
    Ok(())
}

#[command("cr")]
/// get a list of chips dropped by viruses of a particular CR
#[example = "3"]
async fn chip_drop_cr(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    
    if args.is_empty() {
        say!(ctx, msg, "You must provide a CR to search for");
        return Ok(());
    }
    
    let cr_to_get = match args.single::<u8>() {
        Ok(cr) => cr,
        Err(_) => {
            say!(ctx, msg, "An invalid number was provided");
            return Ok(());
        }
    };

    let data = ctx.data.read().await;
    let virus_lock = data.get::<VirusLibrary>().expect("Virus library not found");
    let virus_library: RwLockReadGuard<VirusLibrary> = virus_lock.read().await;

    let cr_list = match virus_library.get_cr(cr_to_get) {
        Some(list) => list,
        None => {
            say!(ctx, msg, "There are no viruses in that CR");
            return Ok(());
        }
    };

    let mut drop_list = Vec::new();

    for virus in cr_list {
        //skip 1 because first is always zenny
        for drop in virus.drops.iter().skip(1) {
            drop_list.push(drop.1.as_str());
        }
    }

    drop_list.sort_unstable();

    long_say!(ctx, msg, drop_list, ", ");
    
    Ok(())
}