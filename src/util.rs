
use serenity::{
    model::{channel::Message},
    prelude::*,
};


///fn say(ctx: Context, msg: Message, say: an expression returning a string)

macro_rules! say {
    ($ctx: ident, $msg: ident, $say: expr) => {
        if let Err(why) = $msg.channel_id.say(&$ctx.http, $say) {
            println!("Could not send message: {:?}", why);
        }
    };
}


macro_rules! long_say {
    ($ctx: ident,  $msg: ident, $say: expr, $sep: expr) => {
        if let Err(why) = $crate::send_long_message(&$ctx, &$msg, $say, $sep) {
            println!("Could not send message: {:?}", why);
        }
    }
}

pub(crate) fn send_long_message<T, S>(ctx: &Context, msg: &Message, to_send: T, separator: S) -> serenity::Result<Message>
    where
        T: std::iter::IntoIterator,
        T::Item: std::fmt::Display,
        S: Into<String>,
{
    let mut reply = String::new();
    let sep = separator.into();
    for val in to_send.into_iter() {

        let to_push = format!("{}", val);
        //a single message cannot be greater than 2000 chars
        if reply.len() + to_push.len() > 1950 {
            msg.channel_id.say(&ctx.http, &reply)?;
            reply.clear();
        }
        reply.push_str(&to_push);
        reply.push_str(&sep);
    }
    //remove last seperator
    for _ in 0..sep.len() {
        reply.pop();
    }
    return msg.channel_id.say(&ctx.http, &reply);
}

pub(crate) fn build_time_rem(now: i64, end: i64) -> String {
    let time_rem = end - now;
    if time_rem < 0 {
        return String::from("Expired");
    }
    let hours_rem = time_rem / (60 * 60);
    let min_rem = (time_rem % (60 * 60)) / 60;
    let sec_rem = (time_rem % (60 * 60)) % 60;
    if hours_rem == 0 {
        return format!("{:02}m:{:02}s", min_rem, sec_rem);
    } else {
        return format!("{:02}h:{:02}m:{:02}s", hours_rem, min_rem, sec_rem);
    }
}