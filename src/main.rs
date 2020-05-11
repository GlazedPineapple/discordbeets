use serenity::{
    client::{Client, Context, EventHandler},
    model::gateway::Ready,
};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
    }
}

const DISCORD_TOKEN: &str = include_str!("token");

fn main() {
    let mut client = Client::new(&DISCORD_TOKEN, Handler).expect("Failed to create the client");

    client.start().expect("Error running the client");
}
