use serenity::framework::standard::{macros::*, Args, CommandResult};
use serenity::{model::channel::Message, prelude::*};

use serde_json::Value;
use simple_error::SimpleError;
use std::f64::NAN;

//https://docs.google.com/document/d/1121cjBNN4BeZdMBGil6Qbuqse-sWpEXPpitQH5fb_Fo/edit#heading=h.yi84u2lickud
//URL for warframe market API

async fn make_request(name: &str) -> Result<Vec<String>, SimpleError> {
    let url = format!("https://api.warframe.market/v1/items/{}/orders", name);

    let text = reqwest::get(&url)
        .await
        .map_err(|_| SimpleError::new("Could not make market request"))?
        .text()
        .await
        .map_err(|_| SimpleError::new("Could not parse response of market request"))?;

    let mut json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|_| SimpleError::new("Could not parse market json data"))?;
    let orders = json["payload"]["orders"]
        .as_array_mut()
        .ok_or_else(|| SimpleError::new("could not convert to array"))?;
    let mut res: Vec<&Value> = orders
        .iter()
        .filter(|val| {
            val["platform"].as_str().unwrap_or("") == "pc"
                && val["order_type"].as_str().unwrap_or("") == "sell"
                && val["user"]["status"].as_str().unwrap_or("offline") != "offline"
                && val["user"]["region"].as_str().unwrap_or("") == "en"
        })
        .collect();

    res.reverse();
    res.truncate(10);
    let mut to_ret: Vec<String> = vec![format!("Players selling {} on pc:", name)];

    for val in res {
        let poster = val["user"]["ingame_name"].as_str().unwrap_or("null");
        let price = val["platinum"].as_f64().unwrap_or(NAN);
        to_ret.push(format!("{} is selling for {:.0} platinum", poster, price));
    }

    Ok(to_ret)
}

#[command]
#[bucket = "Warframe_Market"]
#[description = "Search warframe.market for people selling a given item"]
#[example = "wukong prime"]
pub(crate) async fn market(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say!(
            ctx,
            msg,
            "you must provide an item to search the market for"
        );
        return Ok(());
    }

    let new_args = args.rest().split(' ').collect::<Vec<&str>>();

    let last_word = new_args[new_args.len() - 1].to_lowercase();

    let mut to_search: String = new_args.join("_").to_lowercase();

    if last_word == "prime" {
        to_search.push_str("_set");
    }

    match make_request(&to_search).await {
        Ok(res) => long_say!(ctx, msg, res, "\n"),
        Err(e) => say!(
            ctx,
            msg,
            format!(
                "Did not get a proper response for {}, perhaps you spelled it wrong?\n{:?}",
                to_search, e
            )
        ),
    }
    return Ok(());
}
