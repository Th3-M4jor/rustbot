use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{
    library::{Library, LibraryObject},
    ChipLibrary, VirusLibrary,
    util::{edit_message_by_id, has_reaction_perm},
};

use serenity::{
    model::channel::{Message, ReactionType},
    prelude::*,
    framework::standard::{macros::*, Args, CommandResult},
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

const NUMBERS: &[&str] = &[
    "\u{31}\u{fe0f}\u{20e3}", // 1
    "\u{32}\u{fe0f}\u{20e3}", // 2
    "\u{33}\u{fe0f}\u{20e3}", // 3
    "\u{34}\u{fe0f}\u{20e3}", // 4
    "\u{35}\u{fe0f}\u{20e3}", // 5
    "\u{36}\u{fe0f}\u{20e3}", // 6
    "\u{37}\u{fe0f}\u{20e3}", // 7
    "\u{38}\u{fe0f}\u{20e3}", // 8
    "\u{39}\u{fe0f}\u{20e3}", // 9
];

pub(crate) async fn search_full_library(ctx: &Context, msg: &Message, args: &[&str]) {
    let to_search = args.join(" ");
    let data = ctx.data.read().await;
    let library_lock = data.get::<FullLibrary>().expect("Full library not found");
    let library: RwLockReadGuard<FullLibrary> = library_lock.read().await;

    // let item: Option<&FullLibraryType> = library.get(&to_search);

    if let Some(val) = library.get(&to_search) {
        say!(ctx, msg, val);
        return;
    }
    // else nothing directly matching that name

    if !has_reaction_perm(ctx, msg.channel_id).await {
        match library.search_lib_obj(&to_search) {
            Ok(val) => say!(ctx, msg, val),
            Err(val) => say!(ctx, msg, format!("Did you mean: {}", val.join(", "))),
        }
        return;
    }

    let res;

    match library.search_name_contains(&to_search, None) {
        Some(val) => res = val,
        None => res = library.search_dist(&to_search, None),
    }

    // only one item was returned, print it
    if res.len() == 1 {
        say!(ctx, msg, res[0]);
        return;
    }

    let mut msg_string = String::from("Did you mean: ");
    let mut num: isize = 1;
    for obj in &res {
        msg_string.push_str(&num.to_string());
        msg_string.push_str(": ");
        // msg_string.push_str(&(*obj).format_name());
        msg_string.push_str(&format!("{} ({})", (*obj).get_name(), (*obj).get_kind()));
        msg_string.push_str(", ");
        num += 1;
    }

    // remove last ", "
    msg_string.pop();
    msg_string.pop();

    let msg_to_await: Message;

    match msg.channel_id.say(ctx, msg_string).await {
        Ok(val) => msg_to_await = val,
        Err(why) => {
            println!("Could not send message: {:?}", why);
            return;
        }
    }

    let http_clone = Arc::clone(&ctx.http);
    let msg_id = msg_to_await.id.0;
    let channel_id = msg.channel_id.0;
    let res_len = res.len();

    let all_reactions_added = tokio::spawn(async move {
        for number in NUMBERS.iter().take(res_len) {
            if let Err(why) = http_clone
                .create_reaction(
                    channel_id,
                    msg_id,
                    &ReactionType::Unicode((*number).to_string()),
                )
                .await
            {
                println!("Could not react to message: {:?}", why);
                return false;
            }
        }
        return true;
    });

    let mut got_proper_rection = false;
    let mut edited_msg = String::new();
    while !got_proper_rection {
        if let Some(reaction) = &msg_to_await
            .await_reaction(&ctx)
            .timeout(Duration::from_secs(30))
            .author_id(msg.author.id)
            .await
        {
            let emoji = &reaction.as_inner_ref().emoji.as_data();
            let reacted_emoji = emoji.as_str();
            // zipping here to constrain length to that of "res"
            let pos = res
                .iter()
                .zip(NUMBERS.iter())
                .position(|(_, num_emoji)| *num_emoji == reacted_emoji);

            if let Some(index) = pos {
                #[cfg(debug_assertions)]
                println!("Got a correct reaction, editing message");

                edited_msg.push_str(&format!("{}", res[index]));
                got_proper_rection = true;
                break;
            }
        } else {
            #[cfg(debug_assertions)]
            println!("reaction wait timed out");
            break;
        }
    }

    if let Err(why) = all_reactions_added.await {
        println!("{:?}", why);
    }

    let delete_reactions = msg_to_await.delete_reactions(ctx);

    if got_proper_rection {
        // let edit_message = msg_to_await.edit(ctx, |m| m.content(edited_msg));

        let edit_message = edit_message_by_id(
            ctx,
            msg_to_await.channel_id.0,
            msg_to_await.id.0,
            edited_msg,
        );

        if let Err(why) = tokio::try_join!(edit_message, delete_reactions) {
            println!("Could not delete reactions or edit message: {:?}", why);
        }
    } else if let Err(why) = delete_reactions.await {
        println!("Could not delete reactions: {:?}", why);
    }
}


#[command("drops")]
async fn chip_drop(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    
    if args.is_empty() {
        say!(ctx, msg, "You must provide a chip name");
        return Ok(());
    }
    let chip_name = args.current().unwrap();

    let data = ctx.data.read().await;
    let chip_library_lock = data.get::<ChipLibrary>().expect("No chip library");
    let chip_library:  RwLockReadGuard<ChipLibrary> = chip_library_lock.read().await;
    let chip_res = chip_library.search_lib_obj(chip_name);
    let chip = match chip_res {
        Ok(chip) => chip,
        Err(chips) => {
            say!(ctx, msg, format!("Did you mean: {}", chips.join(", ")));
            return Ok(());
        }
    };
    let virus_libary_lock = data.get::<VirusLibrary>().expect("No virus library");
    let virus_libary: RwLockReadGuard<VirusLibrary> = virus_libary_lock.read().await;
    let mut dropped_by: Vec<&str> = vec![];
    for virus in virus_libary.get_collection().values() {
        for drop in virus.drops.iter() {
            if drop.1 == chip.name {
                dropped_by.push(&virus.name);
            }
        }
    }

    if dropped_by.is_empty() {
        say!(ctx, msg, format!("No known virus currently drops {}", chip.name));
    } else {
        say!(ctx, msg, format!("{} is dropped by: {}", chip.name, dropped_by.join(", ")));
    }
    
    Ok(())
}


impl TypeMapKey for FullLibrary {
    type Value = RwLock<FullLibrary>;
}
