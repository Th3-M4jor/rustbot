#[cfg(not(debug_assertions))]
use serde_json;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::{model::channel::Message, prelude::*};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use super::super::bot_data::BotData;

use simple_error::SimpleError;

use crate::library::battlechip::skills::Skills;
use crate::library::battlechip::BattleChip;
use crate::library::elements::Elements;
use crate::library::{search_lib_obj, Library};
use std::borrow::BorrowMut;
use std::fs;

use std::str::FromStr;

const CHIP_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1lvAKkymOplIJj6jS-N5__9aLIDXI6bETIMz01MK9MfY&exportFormat=txt";

pub struct ChipLibrary {
    chips: HashMap<String, Arc<Box<BattleChip>>>,
}

impl Library for ChipLibrary {
    type LibObj = Arc<Box<BattleChip>>;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Arc<Box<BattleChip>>> {
        return &self.chips;
    }
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
        let chip_text = reqwest::blocking::get(CHIP_URL)
            .expect("no request result")
            .text()
            .expect("no response text")
            .replace("â€™", "'")
            .replace("\u{FEFF}", "")
            .replace("\r", "");
        let mut chip_text_arr: Vec<&str> = chip_text
            .split("\n")
            .filter(|&i| !i.trim().is_empty())
            .collect();

        //load in custom chips if any
        let special_chips_res = fs::read_to_string("./custom_chips.txt");
        let special_chip_text;
        if special_chips_res.is_ok() {
            special_chip_text = special_chips_res.unwrap();
            let mut special_chip_arr: Vec<&str> = special_chip_text
                .split("\n")
                .filter(|&i| !i.trim().is_empty())
                .collect();
            chip_text_arr.append(special_chip_arr.borrow_mut());
        }

        let mut chips: Vec<Box<BattleChip>> = vec![];
        let mut bad_chips: Vec<String> = vec![];
        for i in (0..chip_text_arr.len()).step_by(2) {
            let to_add_res = BattleChip::from_chip_string(chip_text_arr[i], chip_text_arr[i + 1]);
            match to_add_res {
                Ok(chip) => {
                    chips.push(chip);
                }
                Err(_) => {
                    bad_chips.push(String::from(chip_text_arr[i]));
                }
            }
        }

        chips.shrink_to_fit();
        chips.sort_unstable();

        //only write json file if not debug
        #[cfg(not(debug_assertions))]
        {
            let j = serde_json::to_string_pretty(&chips).expect("could not serialize to json");
            fs::write("chips.json", j).expect("could nto write to chips.json");
        }

        while !chips.is_empty() {
            let chip = chips.pop().expect("Something went wrong popping a chip");
            self.chips.insert(chip.name.to_lowercase(), Arc::new(chip));
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

    pub fn search_element(&self, to_get: &str) -> Option<Vec<&str>> {
        //let elem_res = Elements::from_str(to_get);

        let elem_to_get = Elements::from_str(to_get).ok()?;

        return self.search_any(elem_to_get, |a, b| a.element.contains(&b));
    }

    pub fn search_skill(&self, to_get: &str) -> Option<Vec<&str>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        //special case for where you want chips with more than one possible skill
        if skill_to_get == Skills::Varies {
            return self.search_any(skill_to_get, |a, _| a.skills.len() > 1);
        } else {
            return self.search_any(skill_to_get, |a, b| a.skills.contains(&b));
        }
    }

    pub fn search_skill_target(&self, to_get: &str) -> Option<Vec<&str>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        return self.search_any(skill_to_get, |a, b| a.skill_target == b);
    }

    pub fn search_skill_user(&self, to_get: &str) -> Option<Vec<&str>> {
        let skill_to_get = Skills::from_str(to_get).ok()?;

        return self.search_any(skill_to_get, |a, b| a.skill_user == b);
    }

    pub fn search_skill_check(&self, to_get: &str) -> Option<Vec<&str>> {
        //return search_skill_spec!(val.SkillTarget == skill_to_get || val.SkillUser == skill_to_get);
        let skill_to_get = Skills::from_str(to_get).ok()?;

        return self.search_any(skill_to_get, |a, b| {
            a.skill_target == b || a.skill_user == b
        });
    }
}

impl TypeMapKey for ChipLibrary {
    type Value = RwLock<ChipLibrary>;
}

#[command]
#[aliases("chip")]
pub(crate) fn send_chip(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() == 0 {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    };

    let to_get = args.current().unwrap();
    let data = ctx.data.read();
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library = library_lock
        .read()
        .expect("library was poisoned, panicking");
    //let library = locked_library.read().expect("library was poisoned");
    //search!(ctx, msg, to_get, library);
    search_lib_obj(ctx, msg, to_get, library);
    return Ok(());
}

#[command]
#[aliases("skill","skilluser","skillcheck","skilltarget")]
pub(crate) fn send_chip_skill(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut args: Vec<&str>;
    let new_first;

    {
        let data = ctx.data.read();
        let config = data.get::<BotData>().expect("no config found");
        //msg_content_clone = msg.content.clone();
        args = msg.content.split(" ").collect();
        new_first = args[0].replacen(&config.cmd_prefix, "", 1);
        args[0] = new_first.as_str();
    }

    if args.len() < 1 {
        say!(ctx, msg, "you must provide a skill");
        return Ok(());
    }
    let data = ctx.data.read();
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library = library_lock
        .read()
        .expect("chip library poisoned, panicking");
    let skill_res; // = library.search_skill(args[1]);
    match args[0].to_lowercase().as_str() {
        "skill" => skill_res = library.search_skill(args[1]),
        "skilluser" => skill_res = library.search_skill_user(args[1]),
        "skilltarget" => skill_res = library.search_skill_target(args[1]),
        "skillcheck" => skill_res = library.search_skill_check(args[1]),
        "send_chip_skill" => skill_res = library.search_skill(args[1]),
        _ => unreachable!(),
    }

    match skill_res {
        Some(skills) => long_say!(ctx, msg, skills, ", "),
        None => say!(ctx, msg, "nothing matched your search"),
    }
    return Ok(());
}

#[command]
#[aliases("element")]
pub(crate) fn send_chip_element(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide an element");
        return Ok(());
    }
    let data = ctx.data.read();
    let library_lock = data.get::<ChipLibrary>().expect("chip library not found");
    let library = library_lock
        .read()
        .expect("chip library poisoned, panicking");
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
