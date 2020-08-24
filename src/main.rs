use dotenv::dotenv;
use serenity::{
    client::{bridge::voice::ClientVoiceManager, Client, Context, EventHandler},
    framework::{standard::CommandError, StandardFramework},
    model::{
        gateway::{Activity, Ready},
        id::UserId,
        prelude::OnlineStatus,
    },
    prelude::*,
    utils::Color,
};
use std::{env, sync::Arc};

mod commands;
mod ytdl;

use commands::{GENERAL_GROUP, HELP, VOICE_GROUP};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.tag());
        ctx.set_presence(Some(Activity::listening("cum")), OnlineStatus::Online);
    }
}

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

fn main() {
    // Load the .env file into the environment
    dotenv().ok();

    // Gets discord token from the enviroment
    let discord_token = env::var("DISCORD_TOKEN").expect("Env variable DISCORD_TOKEN NOT FOUND");

    // Creates the client and errors if it fails
    let mut client = Client::new(&discord_token, Handler).expect("Failed to create the client");

    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    // Setup the command framework
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.prefix("!").owners(
                    //               Elliot                       Zach
                    vec![UserId(272727593881567242), UserId(168827261682843648)]
                        .into_iter()
                        .collect(),
                )
            })
            .group(&GENERAL_GROUP)
            .group(&VOICE_GROUP)
            .help(&HELP)
            .after(|ctx, msg, cmd_name, error| {
                // Print out error if/when it occurs
                if let Err(CommandError(why)) = error {
                    println!("Error in {}: {:?}", cmd_name, why);
                    msg.channel_id
                        .send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.title(format!("Command `{}` failed", cmd_name))
                                    .description(format!("```\n{}```", why))
                                    .color(Color::DARK_RED)
                            })
                        })
                        .ok();
                }
            }),
    );

    // Setup smooth shutdown
    {
        let shard_manager = Arc::clone(&client.shard_manager);

        ctrlc::set_handler(move || {
            println!("Shutting down now");
            shard_manager.lock().shutdown_all();
        })
        .expect("Failed to enable the ctrl+c handler");
    }

    // Starts client
    client.start().expect("Error running the client");
}
