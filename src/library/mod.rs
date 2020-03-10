pub(crate) mod battlechip;
pub(crate) mod chip_library;
pub(crate) mod elements;
pub(crate) mod ncp_library;
pub(crate) mod virus_library;
pub(crate) mod full_library;
pub(crate) mod blights;
use std::collections::HashMap;
use std::sync::Arc;

use serenity::{model::channel::Message, prelude::*};

use strsim::jaro_winkler;

use std::ops::Deref;

pub trait LibraryObject: std::fmt::Display {
    fn get_name(&self) -> &str;
}

impl<T: LibraryObject> LibraryObject for Arc<Box<T>> {
    fn get_name(&self) -> &str {
        return self.deref().get_name();
    }
}

pub trait Library: TypeMapKey {
    type LibObj: LibraryObject;

    fn get_collection(&self) -> &HashMap<String, Self::LibObj>;

    fn name_contains<'a>(&'a self, to_get: &str, limit: Option<usize>) -> Option<Vec<&'a str>> {
        let limit_val = limit.unwrap_or(5);
        let to_search = to_get.to_lowercase();
        let mut to_ret: Vec<&'a str> = vec![];
        for key in self.get_collection().keys() {
            if key.starts_with(&to_search) {
                let to_push = self.get_collection().get(key).unwrap();
                to_ret.push(to_push.get_name());
                if to_ret.len() > limit_val {
                    break;
                }
            }
        }

        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable();
        return Some(to_ret);
    }

    fn distance(&self, to_get: &str, limit: Option<usize>) -> Vec<&str> {
        let limit_val = limit.unwrap_or(5);
        let mut distances: Vec<(f64, &str)> = vec![];
        for val in self.get_collection().values() {
            let dist = jaro_winkler(&to_get.to_lowercase(), &val.get_name().to_lowercase());
            distances.push((dist, val.get_name()));
        }
        //distances.sort_unstable_by(|a,b| a.0.cmp(&b.0));
        distances.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());
        distances.truncate(limit_val);
        distances.shrink_to_fit();
        let mut to_ret: Vec<&str> = vec![];
        for val in distances {
            to_ret.push(val.1);
        }
        return to_ret;
    }

    fn get(&self, to_get: &str) -> Option<&Self::LibObj> {
        return self.get_collection().get(&to_get.to_lowercase());
    }

    fn search_any<F, T>(&self, to_search: T, cond: F) -> Option<Vec<&str>>
    where
        F: Fn(&Self::LibObj, T) -> bool,
        T: std::marker::Copy,
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

pub(crate) fn search_lib_obj<U, T>(ctx: &Context, msg: &Message, search: &str, lib: T)
    where
    U: Library,
    T: Deref<Target=U>
    {
    let item = lib.get(search);
    if item.is_some() {
        say!(ctx, msg, format!("{}", item.unwrap()));
        return;
    }
    let mut item_search;
    match lib.name_contains(search, None) {
        Some(t) => item_search = t,
        None => item_search = lib.distance(search, None),
    }
    if item_search.len() == 1 {
        let found_item = lib.get(&item_search[0]).unwrap();
        say!(ctx, msg, format!("{}", found_item));
        return;
    }
    item_search.dedup();
    let to_send: String = item_search.join(", ");
    say!(ctx, msg, format!("Did you mean: {}", to_send));
}
