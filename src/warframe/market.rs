use serenity::{
    model::channel::Message,
    prelude::*,
};

use simple_error::SimpleError;
use serde_json::Value;
use std::f64::NAN;

//https://docs.google.com/document/d/1121cjBNN4BeZdMBGil6Qbuqse-sWpEXPpitQH5fb_Fo/edit#heading=h.yi84u2lickud
//URL for warframe market API

fn make_request(name: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let url = format!("https://api.warframe.market/v1/items/{}/orders", name);

    let mut res = reqwest::get(&url)?;
    if res.status() != 200 {
        return Err(Box::new(SimpleError::new("An error occurred")));
    }

    let text = res.text()?;

    let mut json : serde_json::Value = serde_json::from_str(&text)?;
    let orders = json["payload"]["orders"].as_array_mut().ok_or(SimpleError::new("could not convert to array"))?;
    let mut res : Vec<&Value> = orders.iter().filter(|val| {
        val["platform"].as_str().unwrap_or("") == "pc" &&
        val["order_type"].as_str().unwrap_or("") == "sell" &&
            val["user"]["status"].as_str().unwrap_or("offline") != "offline" &&
            val["user"]["region"].as_str().unwrap_or("") == "en"
    }).collect();

    res.reverse();
    res.truncate(10);
    let mut to_ret : Vec<String> = vec![format!("Players selling {} on pc:", name)];

    for val in res {
        let poster = val["user"]["ingame_name"].as_str().unwrap_or("null");
        let price = val["platinum"].as_f64().unwrap_or(NAN);
        to_ret.push(format!("{} is selling for {:.0} platinum", poster, price));
    }

    return Ok(to_ret);

}

pub (crate) fn get_market_info(ctx: Context, msg: Message, args: &[&str]) {

    if args.len() < 2 {
        say!(ctx, msg, "you must provide an item to search the market for");
        return;
    }



    let last_word = args[args.len() - 1].to_lowercase();

    let to_join = &args[1..];
    let mut to_search : String = to_join.join("_").to_lowercase();

    if last_word == "prime" {
        to_search.push_str("_set");
    }

    match make_request(&to_search) {
        Ok(res) => long_say!(ctx, msg, res, "\n"),
        Err(e) => say!(ctx, msg, format!("Did not get a proper response for {}, perhaps you spelled it wrong?\n{:?}", to_search, e)),
    }
}