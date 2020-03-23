use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use serenity::{model::channel::Message, prelude::*};
use serenity::framework::standard::{macros::*, Args, CommandResult};

pub struct DieRoll;

impl DieRoll {
    pub fn roll_dice(to_roll: &str, reroll: bool) -> (i64, Vec<i64>) {
        let mut to_ret: i64 = 0;
        let mut rolls : Vec<i64> = vec![];
        let a: Vec<&str> = to_roll.split('+').collect();
        if a.len() > 1 {
            for b in a {
                let (res, mut vec_to_push) = DieRoll::roll_dice(b, reroll);
                to_ret += res;
                rolls.append(&mut vec_to_push);
            }
        } else {
            let d: Vec<&str> = a[0].split('d').collect();
            if d.len() == 1 {
                let res = d[0].trim().parse::<i64>();

                match res {
                    Ok(val) => {
                        rolls.push(val);
                        return (val, rolls);
                    }
                    Err(_) => {
                        return (-1, vec![]);
                    }
                }
            }
            let amt_to_roll: i64;
            let res = d[0].trim().parse::<i64>();
            match res {
                Ok(val) => amt_to_roll = val,
                Err(_) => amt_to_roll = 1,
            }
            let mut rng = ThreadRng::default();
            for i in 1..d.len() {
                let f: i64;
                let res = d[i].trim().parse::<i64>();
                match res {
                    Ok(val) => {
                        if val <= 1 {
                            f = 6;
                        } else {
                            f = val;
                        }
                    }
                    Err(_) => f = 6,
                }
                let die = Uniform::from(1..=f);
                let mut u: i64 = 0;
                for _ in 0..amt_to_roll {
                    let mut to_add = die.sample(&mut rng);
                    if reroll && to_add < 2 {
                        let new_to_add = die.sample(&mut rng);
                        if new_to_add > to_add {to_add = new_to_add;}
                    }
                    rolls.push(to_add);
                    u += to_add;
                }
                to_ret += u;
            }
        }

        return (to_ret, rolls);
    }
}

#[group]
#[commands(
    roll, reroll, roll_stats
)]
struct DiceCommand;

async fn perform_roll(ctx: &mut Context, msg: &Message, to_roll: &str, reroll: bool) {

    //let mut results: Vec<i64> = vec![];
    let (amt, results) = DieRoll::roll_dice(&to_roll, reroll);
    let repl_str = format!("{:?}", results);
    let reply;
    if repl_str.len() > 1850 {
        reply = format!(
            "{}, you rolled: {}\n[There were too many die rolls to show the result of each one]",
            msg.author.mention().await,
            amt
        );
    } else {
        reply = format!(
            "{}, you rolled: {}\n{}",
            msg.author.mention().await,
            amt,
            repl_str
        );
    }
    say!(ctx, msg, reply);

}

#[command]
#[min_args(1)]
async fn reroll(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    
    if args.len() < 1 {
        say!(
            ctx,
            msg,
            format!(
                "{}, you must supply a number of dice to roll",
                msg.author.mention().await
            )
        );
        return Ok(());
    }
    perform_roll(ctx, msg, args.rest(), true).await;
    return Ok(());
}


#[command]
#[min_args(1)]
pub(crate) async fn roll(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if args.len() < 1 {
        say!(
            ctx,
            msg,
            format!(
                "{}, you must supply a number of dice to roll",
                msg.author.mention().await
            )
        );
        return Ok(());
    }
    perform_roll(ctx, msg, args.rest(), false).await;
    return Ok(());
}

#[command]
#[aliases("rollstats")]
pub(crate) async fn roll_stats(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    let mut stats: [i64; 6] = [0; 6];
    //let mut rolls: Vec<i64> = vec![];
    for i in &mut stats {
        //rolls.clear();
        let (_, mut rolls) = DieRoll::roll_dice("4d6", false);

        //sort reverse to put lowest at the end
        rolls.sort_unstable_by(|a, b| b.cmp(a));
        rolls.pop();

        *i = rolls.iter().fold(0, |acc, val| acc + val);
    }

    say!(
        ctx,
        msg,
        format!(
            "{}, 4d6 drop the lowest:\n{:?}",
            msg.author.mention().await,
            stats
        )
    );
    return Ok(());
}
