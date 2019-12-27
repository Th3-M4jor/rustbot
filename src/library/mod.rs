


pub(crate) mod chip_library;
pub(crate) mod ncp_library;
pub(crate) mod virus_library;
pub(crate) mod elements;
pub(crate) mod battlechip;

use std::collections::HashMap;

use serenity::{
    model::channel::Message,
    prelude::*,
};


use strsim::jaro_winkler;

use std::ops::Deref;

pub trait LibraryObject: std::fmt::Display {
    fn get_name(&self) -> &str;
}


pub trait Library : TypeMapKey {
    type LibObj: LibraryObject;

    fn get_collection(&self) -> &HashMap<String, Box<Self::LibObj>>;

    fn name_contains(&self, to_get: &str) -> Option<Vec<String>> {
        let to_search = to_get.to_lowercase();
        let mut to_ret: Vec<String> = vec![];
        for key in self.get_collection().keys() {
            if key.starts_with(&to_search) {
                let to_push = self.get_collection().get(key).unwrap();
                to_ret.push(to_push.deref().get_name().to_string());
                if to_ret.len() > 5 {
                    break;
                }
            }
        }

        if to_ret.is_empty() { return None; }
        to_ret.sort_unstable();
        return Some(to_ret);
    }

    fn distance(&self, to_get: &str) -> Vec<String> {
        let mut distances: Vec<(f64, String)> = vec![];
        for val in self.get_collection().values() {
            let dist = jaro_winkler(&to_get.to_lowercase(), &val.get_name().to_lowercase());
            distances.push((dist, val.get_name().to_string()));
        }
        //distances.sort_unstable_by(|a,b| a.0.cmp(&b.0));
        distances.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());
        distances.truncate(5);
        distances.shrink_to_fit();
        let mut to_ret: Vec<String> = vec![];
        for val in distances {
            to_ret.push(val.1.clone());
        }
        return to_ret;
    }

    fn get(&self, to_get: &str) -> Option<&Box<Self::LibObj>> {
        return self.get_collection().get(&to_get.to_lowercase());
    }

    fn search_any<F, T>(&self, to_search: T, cond: F,) -> Option<Vec<&str>>
        where F: Fn(&Box<Self::LibObj>, T) -> bool,
        T: std::marker::Copy
    {
        let mut to_ret: Vec<&str> = vec![];
        for val in self.get_collection().values() {
            if cond(val, to_search) {
                to_ret.push(val.get_name());
            }
        }
        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable();
        return Some(to_ret);
    }

}

pub(crate) fn search_lib_obj<T: Library>(ctx: &Context, msg: &Message, search: &str, lib: &T) {
    let item = lib.get(search);
    if item.is_some() {
        say!(ctx, msg, format!("{}", item.unwrap()));
        return;
    }
    let item_search;
    match lib.name_contains(search) {
        Some(t) => item_search = t,
        None => item_search = lib.distance(search),
    }
    if item_search.len() == 1 {
        let found_item = lib.get(&item_search[0]).unwrap();
        say!(ctx, msg, format!("{}", found_item));
        return;
    }
    let to_send: String = item_search.join(", ");
    say!(ctx, msg, format!("Did you mean: {}", to_send));
}