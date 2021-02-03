use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
};
use itertools::Itertools;
use serenity::{
    framework::standard::{macros::{command, group}, Args, CommandResult},
    model::channel::Message,
    prelude::*,
};

use simple_error::simple_error;

use std::{borrow::Cow, i16};

pub struct DieRoll;

impl DieRoll {
    pub fn roll_dice(to_roll: &str, reroll: bool) -> Option<(i64, Vec<i16>)> {
        let mut to_ret: i64 = 0;
        let mut rolls: Vec<i16> = vec![];
        let a: Vec<&str> = to_roll.split('+').collect();
        if a.len() > 1 {
            for b in a {
                let (res, mut vec_to_push) = DieRoll::roll_dice(b, reroll)?;
                to_ret += res;
                rolls.append(&mut vec_to_push);
                if rolls.len() >= u16::MAX as usize {
                    return None; // too many rolls asked for
                }
            }
        } else {
            let d: Vec<&str> = a[0].split('d').collect();
            if d.len() == 1 {
                let res = d[0].trim().parse::<i64>();

                if let Ok(val) = res {
                    rolls.push(val as i16);
                    return Some((val, rolls));
                } else {
                    return None;
                }
            }
            let amt_to_roll = d[0].trim().parse::<i16>().ok()?;
            let mut rng = ThreadRng::default();
            for i in d.iter().skip(1) {
                let mut f = i.trim().parse::<i16>().ok()?;
                if f <= 1 {
                    f = 6;
                }
                let die = Uniform::from(1..=f);
                let mut u: i16 = 0;
                for _ in 0..amt_to_roll {
                    let mut to_add = die.sample(&mut rng);
                    if reroll && to_add < 2 {
                        let new_to_add = die.sample(&mut rng);
                        if new_to_add > to_add {
                            to_add = new_to_add;
                        }
                    }
                    rolls.push(to_add);

                    if rolls.len() >= u16::MAX as usize {
                        return None;
                    }

                    u += to_add;
                }
                to_ret += u as i64;
            }
        }

        Some((to_ret, rolls))
    }
}

#[group]
#[commands(roll, reroll, roll_stats, shuffle)]
/// A group of commands related to rolling dice
struct Dice;

async fn perform_roll(ctx: &Context, msg: &Message, to_roll: &str, reroll: bool) {
    // let mut results: Vec<i64> = vec![];
    let owned_to_roll = to_roll.to_owned();
    let res = tokio::task::spawn_blocking(move || DieRoll::roll_dice(&owned_to_roll, reroll)).await;

    let (amt, results) = match res {
        Ok(Some(val)) => val,
        _ => {
            eprintln!("Error occurred trying to roll dice... {}", to_roll);
            reply!(ctx, msg, "Error occurred, you probably tried to roll way too many dice");
            return;
        }
    };
    let repl_str = format!("{:?}", results);
    let reply = if repl_str.len() > 1850 {
        format!(
            "You rolled: {}\n[There were too many die rolls to show the result of each one]",
            amt
        )
    } else {
        format!(
            "You rolled: {}\n{}",
            amt,
            repl_str
        )
    };
    reply!(ctx, msg, reply);
}

#[command]
/// Same as the roll command, except 1's and 2's will be re-rolled once, keeping the higher result
async fn reroll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(
            ctx,
            msg,
            "You must supply a number of dice to roll"
        );
        return Ok(());
    }
    perform_roll(ctx, msg, args.rest(), true).await;
    return Ok(());
}

#[command]
/// Roll a number of dice, using the format XdY where X is the number of dice, and Y is the number of sides on the die to roll
#[example("1d20")]
#[example("4d27")]
pub(crate) async fn roll(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(
            ctx,
            msg,
            "You must supply a number of dice to roll"
        );
        return Ok(());
    }
    perform_roll(ctx, msg, args.rest(), false).await;
    return Ok(());
}

#[command("rollstats")]
/// Roll character stats for D&D 5e by rolling 4d6 and dropping the lowest 6 times
pub(crate) async fn roll_stats(ctx: &Context, msg: &Message, _: Args) -> CommandResult {
    let mut stats: [i16; 6] = [0; 6];
    // let mut rolls: Vec<i64> = vec![];
    for i in &mut stats {
        // rolls.clear();
        let (_, mut rolls) = DieRoll::roll_dice("4d6", false).ok_or_else(|| simple_error!("Error rolling 4d6, how???"))?;

        // sort reverse to put lowest at the end
        rolls.sort_unstable_by(|a, b| b.cmp(a));
        rolls.pop();

        *i = rolls.iter().sum();
    }

    reply!(
        ctx,
        msg,
        format!(
            "4d6 drop the lowest:\n{:?}",
            stats
        )
    );
    return Ok(());
}

pub(crate) fn perform_shuffle(size: usize) -> Cow<'static, str> {
    if size < 2 {
        return Cow::Borrowed("Cannot shuffle a number less than 2");
    }

    if size > 64 {
        return Cow::Borrowed("Cowardly refusing to shuffle a number greater than 64");
    }

    let mut list: Vec<usize> = Vec::with_capacity(size);

    for num in 1..(size + 1) {
        list.push(num);
    }

    let shuffler = Uniform::from(0..size);
    let mut rng = ThreadRng::default();
    for index in 0..list.len() {
        let rand_index = shuffler.sample(&mut rng);
        list.swap(rand_index, index);
    }

    Cow::Owned(format!("{}",list.iter().format(", ")))

}

#[command]
/// shuffle a series of numbers from 1 to the given argument (inclusive)
#[example = "20"]
pub(crate) async fn shuffle(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        reply!(
            ctx,
            msg,
            "You must supply a number of numbers to shuffle"
        );
        return Ok(());
    }

    let size = match args.single::<usize>() {
        Ok(size) => size,
        Err(_) => {
            reply!(
                ctx,
                msg,
                "An invalid number was provided"
            );
            return Ok(());
        }
    };

    let typing_fut = msg.channel_id.broadcast_typing(ctx);
    let shuffle_handle = tokio::task::spawn_blocking(move || {
        perform_shuffle(size)
    });

    let (_, shuffle_res) = tokio::join!(typing_fut, shuffle_handle);

    reply!(ctx, msg, shuffle_res?);

    Ok(())

}