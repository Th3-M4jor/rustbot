pub(crate) mod battlechip;
pub(crate) mod blights;
pub(crate) mod chip_library;
pub(crate) mod elements;
pub(crate) mod full_library;
pub(crate) mod ncp_library;
pub(crate) mod virus_library;
use std::{collections::HashMap, sync::Arc};

use serenity::{prelude::*, model::channel::Message};

use async_trait::async_trait;


use strsim::jaro_winkler;

use crate::util::{reaction_did_you_mean, has_reaction_perm, edit_message_by_id};

use std::ops::Deref;


#[allow(clippy::module_name_repetitions)]
pub trait LibraryObject: std::fmt::Display + Send + Sync {
    fn get_name(&self) -> &str;

    fn get_kind(&self) -> &str;

    fn get_formatted_name(&self) -> String {
        format!("{} ({})", self.get_name(), self.get_kind())
    }
}

impl<T: LibraryObject + ?Sized> LibraryObject for Arc<T> {
    fn get_name(&self) -> &str {
        self.deref().get_name()
    }

    fn get_kind(&self) -> &str {
        self.deref().get_kind()
    }

    fn get_formatted_name(&self) -> String {
        self.deref().get_formatted_name()
    }

}

#[allow(clippy::module_name_repetitions)]
#[async_trait]
pub trait Library: TypeMapKey {
    type LibObj: LibraryObject;

    fn get_collection(&self) -> &HashMap<String, Self::LibObj>;

    fn name_contains<'a>(&'a self, to_get: &str, limit: Option<usize>) -> Option<Vec<&'a Self::LibObj>> {
        let limit_val = limit.unwrap_or(5);
        let to_search = to_get.to_lowercase();
        let mut to_ret = vec![];
        for key in self.get_collection().keys() {
            if key.starts_with(&to_search) {
                let to_push = self.get_collection().get(key).unwrap();
                to_ret.push(to_push);
                if to_ret.len() > limit_val {
                    break;
                }
            }
        }

        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable_by(|a, b| a.get_name().cmp(b.get_name()));
        Some(to_ret)
    }

    fn distance<'a>(&'a self, to_get: &str, limit: Option<usize>) -> Vec<&'a Self::LibObj> {
        let limit_val = limit.unwrap_or(5);
        let mut distances: Vec<(f64, &Self::LibObj)> = vec![];
        for val in self.get_collection().values() {
            let dist = jaro_winkler(&to_get.to_lowercase(), &val.get_name().to_lowercase());
            distances.push((dist, val));
        }
        // distances.sort_unstable_by(|a,b| a.0.cmp(&b.0));
        distances.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap().reverse());
        distances.truncate(limit_val);
        distances.shrink_to_fit();
        let mut to_ret = vec![];
        for val in distances {
            to_ret.push(val.1);
        }
        to_ret
    }

    fn get(&self, to_get: &str) -> Option<&Self::LibObj> {
        self.get_collection().get(&to_get.to_lowercase())
    }

    fn search_any<F, T>(&self, to_search: T, cond: F) -> Option<Vec<&Self::LibObj>>
    where
        F: Fn(&Self::LibObj, T) -> bool,
        T: std::marker::Copy,
    {
        let mut to_ret = vec![];
        for val in self.get_collection().values() {
            if cond(val, to_search) {
                to_ret.push(val);
            }
        }
        if to_ret.is_empty() {
            return None;
        }
        to_ret.sort_unstable_by(|a,b | a.get_name().cmp(b.get_name()));
        Some(to_ret)
    }

    fn search_lib_obj<'a>(&'a self, search: &str) -> Result<&'a Self::LibObj, Vec<&'a Self::LibObj>> {
        if let Some(item) = self.get(search) {
            return Ok(item);
        }
        let item_search;

        match self.name_contains(search, None) {
            Some(t) => item_search = t,
            None => item_search = self.distance(search, None),
        }

        if item_search.len() == 1 {
            let found_item = self.get(&item_search[0].get_name()).unwrap();
            return Ok(found_item);
        }
        Err(item_search)
    }

    async fn reaction_name_search(&self, ctx: &Context, msg: &Message, to_get: &str) {
        let list = match self.search_lib_obj(to_get) {
            Ok(val) => {
                say!(ctx, msg, val); 
                return;
            },
            Err(val) => val,//say!(ctx, msg, format!("Did you mean: {}", val.iter().map(|a| a.get_name()).collect::<Vec<&str>>().join(", "))),
        };
        if !has_reaction_perm(ctx, msg.channel_id).await {
            say!(ctx, msg, format!("Did you mean: {}", list.iter().map(|a| a.get_name()).collect::<Vec<&str>>().join(", ")));
            return;
        }
    
        let mut msg_string = String::from("Did you mean: ");
        let mut num: isize = 1;
        for obj in list.iter() {
            msg_string.push_str(&num.to_string());
            msg_string.push_str(": ");
            // msg_string.push_str(&(*obj).format_name());
            msg_string.push_str(&(*obj).get_name());
            msg_string.push_str(", ");
            num += 1;
        }
    
        // remove last ", "
        msg_string.pop();
        msg_string.pop();
    
    
        
        let msg_to_await= match msg.channel_id.say(ctx, msg_string).await {
            Ok(val) => val,
            Err(why) => {
                println!("Could not send message: {:?}", why);
                return;
            }
        };
    
        if let Some(num) = reaction_did_you_mean(ctx, &msg_to_await, msg.author.id, list.len()).await {
            if let Err(why) = edit_message_by_id(ctx, msg_to_await.channel_id.0, msg_to_await.id.0, list[num]).await {
                println!("Could not edit message: {:?}", why);
            }
        }
    }



}

// pub(crate) fn search_lib_obj<'b, U, T>(search: &str, lib: &'b T) -> Result<String, Vec<&'b str>>
// where
// U: Library,
// T: Deref<Target = U>,
// {
// let item = lib.get(search);
// if item.is_some() {
// return Ok(format!("{}", item.unwrap()));
// }
// let mut item_search;
// match lib.name_contains(search, None) {
// Some(t) => item_search = t,
// None => item_search = lib.distance(search, None),
// }
// if item_search.len() == 1 {
// let found_item = lib.get(&item_search[0]).unwrap();
// return Ok(format!("{}", found_item));
// }
// item_search.dedup();
// return Err(item_search);
// let to_send: String = item_search.join(", ");
// return format!("Did you mean: {}", to_send);
// }
