use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;


use crate::library::{
    battlechip::BattleChip, ncp_library::NCP, search_lib_obj, virus_library::Virus, Library,
    LibraryObject,
};
//use crate::library::ncp_library::NCP;
//use crate::library::search_lib_obj;
//use crate::library::virus_library::Virus;
//use crate::library::{Library, LibraryObject};
use serenity::{model::channel::Message, prelude::*, model::permissions::Permissions};
use simple_error::SimpleError;
use std::fmt::Formatter;

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


const NUMBERS: &[&str] = &["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣"];


pub(crate) async fn search_full_library(ctx: &Context, msg: &Message, args: &[&str]) {
    let to_search = args.join(" ");
    let data = ctx.data.read().await;
    let library_lock = data.get::<FullLibrary>().expect("Full library not found");
    let library = library_lock.read().await;

    //.expect("library was poisoned, panicking");
    let res : Vec<&str>;
    match search_lib_obj(&to_search, &library) {
        Ok(val) => {
            say!(ctx, msg, val);
            return;
        }
        Err(val) => res = val,
        //say!(ctx, msg, format!("Did you mean: {},", val.join(", ")))
    }
    
    let mut msg_to_await : Message;

    match msg.channel_id.say(ctx, format!("Did you mean: {}", res.join(", "))).await {
        Ok(val) => msg_to_await = val,
        Err(why) => {
            println!("Could not send message: {:?}", why);
            return;
        }
    }

    let channel = match ctx.cache.read().await.guild_channel(msg.channel_id) {
        Some(channel) => channel,
        None => return,
    };

    let current_user_id = ctx.cache.read().await.user.id;
    let permissions =
        channel.read().await.permissions_for_user(ctx, current_user_id).await.unwrap();

        if !permissions.contains(Permissions::ADD_REACTIONS | Permissions::MANAGE_MESSAGES) {
            return;
        }

    for num in 0..res.len() {
        if let Err(why) = msg_to_await.react(ctx, NUMBERS[num]).await {
            println!("Could not react to message: {:?}", why);
            return;
        }
    }
    let mut got_proper_rection = false;
    while !got_proper_rection {
        if let Some(reaction) = &msg_to_await.await_reaction(&ctx).timeout(Duration::from_secs(15)).author_id(msg.author.id).await {
            let emoji = &reaction.as_inner_ref().emoji.as_data();
            let emoji_str = emoji.as_str();
            for num in 0..res.len() {
                if NUMBERS[num] == emoji_str {
                    let to_say = search_lib_obj(res[num], &library);
                    match to_say {
                        Err(_) => {
                            say!(ctx, msg, "An error occurred, inform the owner");
                            return;
                        }
                        Ok(val) => {
                            if let Err(why) = msg_to_await.edit(ctx, |m| m.content(val)).await {
                                println!("Could not edit message: {:?}", why);
                            }
                            #[cfg(debug_assertions)]
                            println!("Got a correct reaction, edited message");
                            got_proper_rection = true;
                            break;
                        }
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            println!("reaction wait timed out");
            break;
        }
        #[cfg(debug_assertions)]
        println!("Did not get a correct reaction, waiting again");
    }

    
    if let Err(why) = msg_to_await.delete_reactions(ctx).await {
        println!("Could not delete reactions: {:?}", why);
    }

    
    //say!(ctx, msg, search_lib_obj(&to_search, library));
}

impl TypeMapKey for FullLibrary {
    type Value = RwLock<FullLibrary>;
}
