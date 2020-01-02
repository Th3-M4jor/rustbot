use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use serenity::{model::channel::Message, prelude::*};
use crate::library::{Library, LibraryObject};
use simple_error::SimpleError;
use crate::library::battlechip::BattleChip;
use crate::library::ncp_library::NCP;
use crate::library::virus_library::Virus;
use crate::library::search_lib_obj;
use std::fmt::Formatter;

pub struct FullLibrary {
    library: HashMap<String, FullLibraryType>,
}


pub enum FullLibraryType {
    #[non_exhaustive]
    BattleChip(Arc<Box<BattleChip>>),
    NCP(Arc<Box<NCP>>),
    Virus(Arc<Box<Virus>>),
}

impl LibraryObject for FullLibraryType {
    fn get_name(&self) -> &str {
        match self {
            FullLibraryType::BattleChip(chip) => {
                chip.get_name()
            },
            FullLibraryType::NCP(ncp) => {
                ncp.get_name()
            },
            FullLibraryType::Virus(virus) => {
                virus.get_name()
            }
        }
    }
}

impl std::fmt::Display for FullLibraryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return match self {
            FullLibraryType::BattleChip(chip) => {
                write!(f, "{}", chip)
            },
            FullLibraryType::NCP(ncp) => {
                write!(f, "{}", ncp)
            },
            FullLibraryType::Virus(virus) => {
                write!(f, "{}", virus)
            }
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
                FullLibraryType::NCP(_) => {"_n"},
                FullLibraryType::BattleChip(_) => {"_c"},
                FullLibraryType::Virus(_) => {"_v"},
            };
            let name = obj.get_name().to_lowercase() + dup;
            res = self.library.insert(name, obj);
        } else {
            res = self.library.insert(obj.get_name().to_lowercase(), obj);
        }
        return match res {
            Some(t) => Err(SimpleError::new(format!("{}", t.get_name()))),
            None => Ok(()),
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

pub (crate) fn search_full_library(ctx: Context, msg: Message, args: &[&str]) {
    let to_search = args.join(" ");
    let data = ctx.data.read();
    let library_lock =
        data.get::<FullLibrary>().expect("Full library not found");
    let library = library_lock
        .read()
        .expect("library was poisoned, panicking");
    search_lib_obj(&ctx, msg, &to_search, library);
}

impl TypeMapKey for FullLibrary {
    type Value = RwLock<FullLibrary>;
}