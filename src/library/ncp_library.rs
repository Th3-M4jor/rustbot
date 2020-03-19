use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

use serde::{Serialize};
use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{macros::command, Args, CommandResult};

#[cfg(not(debug_assertions))]
use serde_json;
#[cfg(not(debug_assertions))]
use tokio::fs;

use regex::Regex;

use crate::library::{search_lib_obj, Library, LibraryObject};
use std::fmt::Formatter;

use unicode_normalization::UnicodeNormalization;

const NCP_URL: &'static str = "https://docs.google.com/feeds/download/documents/export/Export?id=1VhZSnjvwSTMxKKfJvKcwqaJDqxD_dXarmAlAYRmlV2k&exportFormat=txt";

#[derive(Serialize)]
#[serde(rename_all(serialize = "PascalCase"))]
pub struct NCP {
    pub name: String,
    pub e_b_cost: u8,
    pub color: String,
    pub all: String,
    pub description: String,
}

impl LibraryObject for NCP {
    #[inline]
    fn get_name(&self) -> &str {
        return &self.name;
    }
}

impl NCP {
    pub fn new<T: Into<String>, S: Into<u8>>(name: T, cost: S, color: T, all: T, desc: T) -> NCP {
        NCP {
            name: name.into().nfc().collect::<String>(),
            e_b_cost: cost.into(),
            color: color.into().nfc().collect::<String>(),
            all: all.into().nfc().collect::<String>(),
            description: desc.into().nfc().collect::<String>(),
        }
    }
}

impl std::fmt::Display for NCP {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(
            f,
            "```{} - ({} EB) - {}\n{}```",
            self.name, self.e_b_cost, self.color, self.description
        );
    }
}

pub struct NCPLibrary {
    library: HashMap<String, Arc<Box<NCP>>>,
}

const COLORS: &[&str] = &["white", "pink", "yellow", "green", "blue", "red", "gray"];

impl Library for NCPLibrary {
    type LibObj = Arc<Box<NCP>>;

    #[inline]
    fn get_collection(&self) -> &HashMap<String, Arc<Box<NCP>>> {
        return &self.library;
    }
}

impl NCPLibrary {
    pub fn new() -> NCPLibrary {
        NCPLibrary {
            library: HashMap::new(),
        }
    }

    pub async fn load_programs(&mut self) -> usize {
        lazy_static! {
            static ref NCP_TEST: Regex =
                Regex::new(r"(.+)\s\((\d+)\sEB\)\s-\s(.+)").expect("Bad NCP regex");
        }
        self.library.clear();
        let mut ncp_list: Vec<Box<NCP>> = vec![];
        let ncp_text = reqwest::get(NCP_URL).await
            .expect("no request result")
            .text().await
            .expect("no response text")
            .replace("â€™", "'")
            .replace("\u{FEFF}", "")
            .replace("\r", "");
        let ncp_text_arr: Vec<&str> = ncp_text
            .split("\n")
            .filter(|&i| !i.trim().is_empty())
            .collect();
        let mut curr_color: String = String::new();
        //let mut new_color : String;
        for ncp in ncp_text_arr {
            if COLORS.contains(&ncp.trim().to_lowercase().as_str()) {
                curr_color = String::from(ncp.trim());
                continue;
            }

            let ncp_cap_res = NCP_TEST.captures(ncp);
            let ncp_cap;
            match ncp_cap_res {
                Some(val) => ncp_cap = val,
                None => continue,
            }
            let name = ncp_cap.get(1);
            let cost = ncp_cap.get(2);
            let desc = ncp_cap.get(3);
            if name.is_none() || cost.is_none() || desc.is_none() {
                continue;
            }
            let cost_val = cost
                .unwrap()
                .as_str()
                .parse::<u8>()
                .unwrap_or(u8::max_value());
            ncp_list.push(Box::new(NCP::new(
                name.unwrap().as_str(),
                cost_val,
                &curr_color,
                ncp,
                desc.unwrap().as_str(),
            )));
        }

        //only write json file if not debug
        #[cfg(not(debug_assertions))]
        {
            let j = serde_json::to_string_pretty(&ncp_list).expect("could not serialize to json");
            fs::write("naviCust.json", j).await.expect("could not write to naviCust.json");
        }
        while !ncp_list.is_empty() {
            let ncp = ncp_list.pop().unwrap();
            self.library.insert(ncp.name.to_lowercase(), Arc::new(ncp));
        }
        return self.library.len();
    }

    pub fn search_color(&self, color: &str) -> Option<Vec<&str>> {
        if !COLORS.contains(&color.to_lowercase().as_str()) {
            return None;
        }
        return self.search_any(color, |a, b| a.color.to_lowercase() == b.to_lowercase());
    }
}

impl TypeMapKey for NCPLibrary {
    type Value = RwLock<NCPLibrary>;
}

#[command]
#[aliases("ncp")]
pub(crate) async fn send_ncp(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let library_lock = data.get::<NCPLibrary>().expect("NCP library not found");
    let library = library_lock.read().await;
    say!(ctx, msg, search_lib_obj(args.current().await.unwrap(), library));
    return Ok(());
}

#[command]
#[aliases("ncpcolor")]
pub(crate) async fn send_ncp_color(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        say!(ctx, msg, "you must provide a name");
        return Ok(());
    }
    let data = ctx.data.read().await;
    let library_lock = data.get::<NCPLibrary>().expect("NCP library not found");
    let library = library_lock.read().await;
    match library.search_color(args.current().await.unwrap()) {
        Some(list) => long_say!(ctx, msg, list, ", "),
        None => say!(ctx, msg, "Nothing matched your search"),
    }
    return Ok(());
}
