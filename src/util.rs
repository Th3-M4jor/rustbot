
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
    ($ctx: ident,  $msg: ident, $say: expr) => {
        if let Err(why) = $crate::send_long_message(&$ctx, &$msg, $say) {
            println!("Could not send message: {:?}", why);
        }
    }
}

pub(crate) fn send_long_message<T>(ctx: &Context, msg: &Message, to_send: T) -> serenity::Result<Message>
    where
        T: std::iter::IntoIterator,
        T::Item: std::fmt::Display,
{
    let mut reply = String::new();
    for val in to_send.into_iter() {

        let to_push = format!("{}", val);
        //a single message cannot be greater than 2000 chars
        if reply.len() + to_push.len() > 1950 {
            msg.channel_id.say(&ctx.http, &reply)?;
            reply.clear();
        }
        reply.push_str(&to_push);
        reply.push_str(", ");
    }
    //remove last ", "
    reply.pop();
    reply.pop();
    return msg.channel_id.say(&ctx.http, &reply);
}