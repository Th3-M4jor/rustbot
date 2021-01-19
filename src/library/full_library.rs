use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{
    library::{Library, LibraryObject},
    util::{edit_message_by_id, has_reaction_perm, reaction_did_you_mean, send_reply},
    ChipLibrary, VirusLibrary,
};

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};
use simple_error::SimpleError;

use strsim::jaro_winkler;

pub struct FullLibrary {
    library: HashMap<String, Arc<dyn LibraryObject>>,
}

impl FullLibrary {
    pub fn new() -> FullLibrary {
        FullLibrary {
            library: HashMap::new(),
        }
    }
#[allow(clippy::map_entry)]
    pub fn insert(&mut self, obj: Arc<dyn LibraryObject>) -> Result<(), SimpleError> {
        let res = if self.library.contains_key(&obj.get_name().to_lowercase()) {
            let dup = match obj.get_kind() {
                "NCP" => "_n",
                "Chip" => "_c",
                "Virus" => "_v",
                _ => unreachable!(),
            };
            let name = obj.get_name().to_lowercase() + dup;
            self.library.insert(name, obj)
        } else {
            self.library.insert(obj.get_name().to_lowercase(), obj)
        };

        match res {
            Some(t) => Err(SimpleError::new(t.get_name().to_string())),
            None => Ok(()),
        }
    }

    pub fn search_dist<'fl>(
        &'fl self,
        to_search: &str,
        limit: Option<usize>,
    ) -> Vec<&'fl Arc<dyn LibraryObject>> {
        let limit_val = limit.unwrap_or(9);

        if limit_val == 0 {
            panic!("Recieved 0 as a limit value");
        }

        let obj_name = to_search.to_lowercase();

        let mut distances: Vec<(f64, &Arc<dyn LibraryObject>)> = vec![];

        for val in &self.library {
            let dist = jaro_winkler(&obj_name, &val.1.get_name().to_lowercase());
            distances.push((dist, val.1));
        }

        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());

        distances.truncate(limit_val);

        // distances.into_iter().map()

        let mut to_ret = vec![];

        for val in distances {
            to_ret.push(val.1);
        }

        to_ret
    }

    pub fn search_name_contains<'fl>(
        &'fl self,
        to_search: &str,
        limit: Option<usize>,
    ) -> Option<Vec<&'fl Arc<dyn LibraryObject>>> {
        let limit_val = limit.unwrap_or(9);

        if limit_val == 0 {
            panic!("Recieved 0 as a limit value");
        }

        let obj_name = to_search.to_lowercase();

        let mut to_ret = vec![];

        for val in &self.library {
            if val.0.starts_with(&obj_name) {
                to_ret.push(val.1);
                if to_ret.len() == limit_val {
                    break;
                }
            }
        }

        if to_ret.is_empty() {
            None
        } else {
            Some(to_ret)
        }
    }

    pub fn clear(&mut self) {
        self.library.clear();
    }

    pub fn len(&self) -> usize {
        self.library.len()
    }
}

impl Library for FullLibrary {
    type LibObj = Arc<dyn LibraryObject>;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Arc<dyn LibraryObject>> {
        &self.library
    }
}

pub(crate) async fn search_full_library(ctx: &Context, msg: &Message, args: &[&str]) {
    let to_search = args.join(" ");
    let data = ctx.data.read().await;
    let library_lock = data.get::<FullLibrary>().expect("Full library not found");
    let library: RwLockReadGuard<FullLibrary> = library_lock.read().await;

    // let item: Option<&FullLibraryType> = library.get(&to_search);

    if let Some(val) = library.get(&to_search) {
        reply!(ctx, msg, val);
        return;
    }
    // else nothing directly matching that name

    if !has_reaction_perm(ctx, msg.channel_id).await {
        let to_say = match library.search_lib_obj(&to_search) {
            Ok(val) => val.to_string(),
            Err(val) => format!("Did you mean: {}", val.iter().map(|a| format!("{} ({})",a.get_name(), a.get_kind())).collect::<Vec<String>>().join(", ")),
        };
        reply!(ctx, msg, to_say, false);
        return;
    }

    let res;

    match library.search_name_contains(&to_search, None) {
        Some(val) => res = val,
        None => res = library.search_dist(&to_search, None),
    }

    // only one item was returned, print it
    if res.len() == 1 {
        reply!(ctx, msg, res[0]);
        return;
    }

    let mut msg_string = String::from("Did you mean: ");
    let mut num: isize = 1;
    for obj in res.iter() {
        msg_string.push_str(&num.to_string());
        msg_string.push_str(": ");
        // msg_string.push_str(&(*obj).format_name());
        msg_string.push_str(&(*obj).get_formatted_name());
        msg_string.push_str(", ");
        num += 1;
    }

    // remove last ", "
    msg_string.pop();
    msg_string.pop();

    let msg_to_await = match send_reply(ctx, msg, &msg_string, false).await {
        Ok(val) => val,
        Err(why) => {
            println!("Could not send reply: {:?}", why);
            return;
        }
    };

    if let Some(num) = reaction_did_you_mean(ctx, &msg_to_await, msg.author.id, res.len()).await {
        if let Err(why) = edit_message_by_id(ctx, msg_to_await.channel_id.0, msg_to_await.id.0, res[num]).await {
            println!("Could not edit message: {:?}", why);
        }
    }
}

#[command("drops")]
#[example("Widesword")]
/// Returns a list of viruses who drop the given chip
async fn chip_drop(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(ctx, msg, "You must provide a chip name");
        return Ok(());
    }
    let chip_name = args.current().unwrap();

    let data = ctx.data.read().await;
    let chip_library_lock = data.get::<ChipLibrary>().expect("No chip library");
    let chip_library: RwLockReadGuard<ChipLibrary> = chip_library_lock.read().await;
    let chip_res = chip_library.search_lib_obj(chip_name);

    let chip = match chip_res {
        Ok(chip) => chip,
        Err(chips) => {
            let to_say = chips.iter().map(|a| a.get_name()).collect::<Vec<&str>>().join(", ");
            reply!(ctx, msg, format!("Did you mean: {}", to_say));
            return Ok(());
        }
    };

    let virus_libary_lock = data.get::<VirusLibrary>().expect("No virus library");
    let virus_libary: RwLockReadGuard<VirusLibrary> = virus_libary_lock.read().await;
    let mut dropped_by: Vec<&str> = vec![];
    for virus in virus_libary.get_collection().values() {
        for drop in virus.drops.0.iter() {
            if drop.1 == chip.name {
                dropped_by.push(&virus.name);
            }
        }
    }

    if dropped_by.is_empty() {
        reply!(
            ctx,
            msg,
            format!("No known virus currently drops {}", chip.name)
        );
    } else {
        reply!(
            ctx,
            msg,
            format!("{} is dropped by: {}", chip.name, dropped_by.join(", "))
        );
    }

    Ok(())
}

pub(crate) fn check_virus_drops(
    virus_lib: &VirusLibrary,
    chip_lib: &ChipLibrary,
) -> Result<(), SimpleError> {
    for virus in virus_lib.get_collection().values() {
        for drop in virus.drops.0.iter() {
            if drop.1.to_ascii_lowercase().contains("zenny") {
                continue;
            }
            if chip_lib.get(&drop.1).is_none() {
                return Err(SimpleError::new(format!(
                    "Warning, {} drops {} at {}, however it is not in the chip library",
                    virus.name, drop.1, drop.0
                )));
            }
        }
    }
    Ok(())
}

impl TypeMapKey for FullLibrary {
    type Value = RwLock<FullLibrary>;
}
