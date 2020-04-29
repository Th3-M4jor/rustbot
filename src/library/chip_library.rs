#[cfg(not(debug_assertions))]
use serde_json;
use serenity::{
    framework::standard::{macros::*, Args, CommandResult},
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
    },
    LibraryObject,
};
use std::borrow::BorrowMut;

#[cfg(not(debug_assertions))]
use tokio::fs;

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
        // get chip text and replace necessary characters for compatibility
        // let chip_text = ChipLibrary::get_chip_text(&self.chip_url).await;
        let mut chip_text_arr: Vec<&str> = chip_text
            .split('\n')
            .filter(|&i| !i.trim().is_empty())
            .collect();

        // load in custom chips if any
        // let special_chips_res = ChipLibrary::get_custom_chip_text(&self.custom_chip_url).await;
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
        // let mut bad_chips: Vec<String> = vec![];
        for i in (0..chip_text_arr.len()).step_by(2) {
            let chip = BattleChip::from_chip_string(chip_text_arr[i], chip_text_arr[i + 1])
                .map_err(|_| {
                    SimpleError::new(format!("Found an invalid chip:\n{}", chip_text_arr[i]))
                })?;
            chips.push(chip);

            // yield after each chip parsed to avoid blocking
            tokio::task::yield_now().await;
        }

        chips.shrink_to_fit();
        chips.sort_unstable();

        // only write json file if not debug
        #[cfg(not(debug_assertions))]
        {
            let j = serde_json::to_string_pretty(&chips).expect("could not serialize to json");
            fs::write("chips.json", j)
                .await
                .expect("could not write to chips.json");
        }

        while !chips.is_empty() {
            let chip = chips.pop().expect("Something went wrong popping a chip");
            self.chips.insert(chip.name.to_lowercase(), Arc::new(chip));
        }

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

    pub fn search_element(&self, to_get: &str) -> Option<Vec<&str>> {
        // let elem_res = Elements::from_str(to_get);

        let elem_to_get = Elements::from_str(to_get).ok()?;

        self.search_any(elem_to_get, |a, b| a.element.contains(&b))
    }

    pub fn search_skill(&self, to_get: &str) -> Option<Vec<&str>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        // special case for where you want chips with more than one possible skill
        if skill_to_get == Skills::Varies {
            self.search_any(skill_to_get, |a, _| a.skills.len() > 1)
        } else {
            self.search_any(skill_to_get, |a, b| a.skills.contains(&b))
        }
    }

    pub fn search_skill_target(&self, to_get: &str) -> Option<Vec<&str>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        self.search_any(skill_to_get, |a, b| a.skill_target == b)
    }

    pub fn search_skill_user(&self, to_get: &str) -> Option<Vec<&str>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        self.search_any(skill_to_get, |a, b| a.skill_user == b)
    }

    pub fn search_skill_check(&self, to_get: &str) -> Option<Vec<&str>> {
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
#[commands(send_chip, send_chip_element)]
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
pub(crate) async fn send_chip(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    };

    let to_get = args.current().unwrap();
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library = library_lock.read().await;
    // let library = locked_library.read().expect("library was poisoned");
    // search!(ctx, msg, to_get, library);

    match library.search_lib_obj(to_get) {
        Ok(val) => say!(ctx, msg, val),
        Err(val) => say!(ctx, msg, format!("Did you mean: {}", val.join(", "))),
    }
    // say!(ctx, msg, search_lib_obj(to_get, library));
    return Ok(());
}

#[command("skill")]
/// get a list of chips that use the specified skill in it's attack roll
#[example = "Sense"]
async fn send_chip_skill(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill(&skill) {
        Some(chips) => long_say!(ctx, msg, chips, ", "),
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command("user")]
/// get a list of chips that have a save and the DC is determined by the specified skill
#[example = "Strength"]
async fn send_chip_skill_user(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill_user(&skill) {
        Some(chips) => long_say!(ctx, msg, chips, ", "),
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command("target")]
/// get a list of chips where the specified skill is used to make the save by the target
#[example = "Speed"]
async fn send_chip_skill_target(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill_target(&skill) {
        Some(chips) => long_say!(ctx, msg, chips, ", "),
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command("check")]
/// get a list of chips where the specified skill is used either to determine the save DC or to make the save
#[example = "Bravery"]
async fn send_chip_skill_check(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let skill = args.single::<String>()?;
    let data = ctx.data.read().await;
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library: RwLockReadGuard<ChipLibrary> = library_lock.read().await;
    match library.search_skill_check(&skill) {
        Some(chips) => long_say!(ctx, msg, chips, ", "),
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command("element")]
/// get a list of chips which are of the specified element
#[example = "Aqua"]
pub(crate) async fn send_chip_element(
    ctx: &mut Context,
    msg: &Message,
    args: Args,
) -> CommandResult {
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
        Some(elem) => long_say!(ctx, msg, elem, ", "),
        None => say!(
            ctx,
            msg,
            "nothing matched your search, are you sure you gave an element?"
        ),
    }
    return Ok(());
}
