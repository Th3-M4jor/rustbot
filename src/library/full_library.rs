use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::library::{
    battlechip::BattleChip, ncp_library::NCP, search_lib_obj, virus_library::Virus, Library,
    LibraryObject,
};
//use crate::library::ncp_library::NCP;
//use crate::library::search_lib_obj;
//use crate::library::virus_library::Virus;
//use crate::library::{Library, LibraryObject};
use serenity::{model::channel::Message, model::permissions::Permissions, prelude::*};
use simple_error::SimpleError;
use std::fmt::Formatter;

use strsim::jaro_winkler;

pub struct FullLibrary {
    library: HashMap<String, FullLibraryType>,
}

pub enum FullLibraryType {
    #[non_exhaustive]
    BattleChip(Arc<BattleChip>),
    NCP(Arc<NCP>),
    Virus(Arc<Virus>),
}

impl LibraryObject for FullLibraryType {
    fn get_name(&self) -> &str {
        match self {
            FullLibraryType::BattleChip(chip) => chip.get_name(),
            FullLibraryType::NCP(ncp) => ncp.get_name(),
            FullLibraryType::Virus(virus) => virus.get_name(),
        }
    }
}

impl std::fmt::Display for FullLibraryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return match self {
            FullLibraryType::BattleChip(chip) => write!(f, "{}", chip),
            FullLibraryType::NCP(ncp) => write!(f, "{}", ncp),
            FullLibraryType::Virus(virus) => write!(f, "{}", virus),
        };
    }
}

impl FullLibraryType {
    fn format_name(&self) -> String {
        match self {
            FullLibraryType::BattleChip(chip) => format!("{} (Chip)", chip.get_name()),
            FullLibraryType::NCP(ncp) => format!("{} (NCP)", ncp.get_name()),
            FullLibraryType::Virus(virus) => format!("{} (Virus)", virus.get_name()),
        }
    }
}

impl FullLibrary {
    pub fn new() -> FullLibrary {
        FullLibrary {
            library: HashMap::new(),
        }
    }

    pub fn insert(&mut self, obj: FullLibraryType) -> Result<(), SimpleError> {
        let res;
        if self.library.contains_key(&obj.get_name().to_lowercase()) {
            let dup = match obj {
                FullLibraryType::NCP(_) => "_n",
                FullLibraryType::BattleChip(_) => "_c",
                FullLibraryType::Virus(_) => "_v",
            };
            let name = obj.get_name().to_lowercase() + dup;
            res = self.library.insert(name, obj);
        } else {
            res = self.library.insert(obj.get_name().to_lowercase(), obj);
        }
        return match res {
            Some(t) => Err(SimpleError::new(format!("{}", t.get_name()))),
            None => Ok(()),
        };
    }

    pub fn search_dist<'fl>(
        &'fl self,
        to_search: &str,
        limit: Option<usize>,
    ) -> Vec<&'fl FullLibraryType> {
        let limit_val = limit.unwrap_or(9);

        if limit_val == 0 {
            panic!("Recieved 0 as a limit value");
        }

        let obj_name = to_search.to_lowercase();

        let mut distances: Vec<(f64, &FullLibraryType)> = vec![];

        for val in &self.library {
            let dist = jaro_winkler(&obj_name, &val.1.get_name().to_lowercase());
            distances.push((dist, val.1));
        }

        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());

        distances.truncate(limit_val);

        //distances.into_iter().map()

        let mut to_ret = vec![];

        for val in distances {
            to_ret.push(val.1);
        }

        return to_ret;
    }

    pub fn search_name_contains<'fl>(
        &'fl self,
        to_search: &str,
        limit: Option<usize>,
    ) -> Option<Vec<&'fl FullLibraryType>> {
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

        if to_ret.len() == 0 {
            return None;
        } else {
            return Some(to_ret);
        }
    }

    pub fn clear(&mut self) {
        self.library.clear();
    }

    pub fn len(&self) -> usize {
        return self.library.len();
    }
}

impl Library for FullLibrary {
    type LibObj = FullLibraryType;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, FullLibraryType> {
        return &self.library;
    }
}

const NUMBERS: &[&str] = &["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣"];

pub(crate) async fn search_full_library(ctx: &Context, msg: &Message, args: &[&str]) {
    let to_search = args.join(" ");
    let data: RwLockReadGuard<ShareMap> = ctx.data.read().await;
    let library_lock = data.get::<FullLibrary>().expect("Full library not found");
    let library: RwLockReadGuard<FullLibrary> = library_lock.read().await;

    //let item: Option<&FullLibraryType> = library.get(&to_search);

    if let Some(val) = library.get(&to_search) {
        say!(ctx, msg, val);
        return;
    }
    //else nothing directly matching that name

    let channel = match ctx.cache.read().await.guild_channel(msg.channel_id) {
        Some(channel) => channel,
        None => {
            match search_lib_obj(&to_search, &library) {
                Ok(val) => say!(ctx, msg, val),
                Err(val) => say!(ctx, msg, format!("Did you mean: {}", val.join(", "))),
            }
            return;
        }
    };

    let current_user_id = ctx.cache.read().await.user.id;
    let permissions = channel
        .read()
        .await
        .permissions_for_user(ctx, current_user_id)
        .await
        .unwrap();

    if !permissions.contains(Permissions::ADD_REACTIONS | Permissions::MANAGE_MESSAGES) {
        match search_lib_obj(&to_search, &library) {
            Ok(val) => say!(ctx, msg, val),
            Err(val) => say!(ctx, msg, format!("Did you mean: {}", val.join(", "))),
        }
        return;
    }

    let res: Vec<&FullLibraryType>;

    match library.search_name_contains(&to_search, None) {
        Some(val) => res = val,
        None => res = library.search_dist(&to_search, None),
    }

    //only one item was returned, print it
    if res.len() == 1 {
        say!(ctx, msg, res[0]);
        return;
    }

    let mut msg_string = String::from("Did you mean: ");
    let mut num: isize = 1;
    for obj in &res {
        msg_string.push_str(&num.to_string());
        msg_string.push_str(": ");
        msg_string.push_str(&(*obj).format_name());
        msg_string.push_str(", ");
        num += 1;
    }

    //remove last ", "
    msg_string.pop();
    msg_string.pop();

    let mut msg_to_await: Message;

    match msg.channel_id.say(ctx, msg_string).await {
        Ok(val) => msg_to_await = val,
        Err(why) => {
            println!("Could not send message: {:?}", why);
            return;
        }
    }

    for ( _ , num_emoji) in res.iter().zip(NUMBERS.iter()) {
        if let Err(why) = msg_to_await.react(ctx, *num_emoji).await {
            println!("Could not react to message: {:?}", why);
            return;
        }
    }
    let mut got_proper_rection = false;
    while !got_proper_rection {
        if let Some(reaction) = &msg_to_await
            .await_reaction(&ctx)
            .timeout(Duration::from_secs(30))
            .author_id(msg.author.id)
            .await
        {
            let emoji = &reaction.as_inner_ref().emoji.as_data();
            let reacted_emoji = emoji.as_str();

            //iterate over both at once with .zip()
            for (response, num_emoji) in res.iter().zip(NUMBERS.iter()) {
                if *num_emoji == reacted_emoji {
                    if let Err(why) = msg_to_await.edit(ctx, |m| m.content(format!("{}",response))).await {
                        println!("Could not edit message: {:?}", why);
                    }
                    #[cfg(debug_assertions)]
                    println!("Got a correct reaction, edited message");
                    got_proper_rection = true;
                    break;
                }
            }
        } else {
            #[cfg(debug_assertions)]
            println!("reaction wait timed out");
            break;
        }
    }

    if let Err(why) = msg_to_await.delete_reactions(ctx).await {
        println!("Could not delete reactions: {:?}", why);
    }
}
impl TypeMapKey for FullLibrary {
    type Value = RwLock<FullLibrary>;
}
