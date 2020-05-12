use crate::ytdl;
use crate::VoiceManager;
use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId, misc::Mentionable},
    prelude::Context,
};
use std::collections::HashSet;

#[help]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

#[group]
#[commands(ping)]
/// The general command group
struct General;

#[group]
#[commands(join, leave, play /*, search */)]
/// The voice commands
struct Voice;

#[command]
/// Ping the bot
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    // send a pong message back to the user
    msg.channel_id.say(&ctx, "pong!")?;

    Ok(())
}

#[command]
#[only_in(guild)]
/// Make the bot join a vc
fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
    // Get the guild from the message
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            msg.channel_id.say(&ctx, "Groups and DMs not supported")?;

            return Ok(());
        }
    };
    // Get the id of the guild
    let guild_id = guild.read().id;

    // Get the channel id to join
    let channel_id = match guild
        .read()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id)
    {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx, "User is not in a voice channel")?;

            return Ok(());
        }
    };

    // Access the voice manager mutex
    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected voice manager in share map");
    // Gain a lock on the mutex
    let mut manager = manager_lock.lock();

    // Try to join the channel
    if manager.join(guild_id, channel_id).is_some() {
        msg.channel_id
            .say(&ctx, &format!("Joined {}", channel_id.mention()))?;
    } else {
        msg.channel_id.say(&ctx, "Error joining the channel")?;
    }

    Ok(())
}

#[command]
#[only_in(guild)]
/// Make the bot leave a vc
fn leave(ctx: &mut Context, msg: &Message) -> CommandResult {
    // Get the guild id from the message
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx, "Groups and DMs not supported")?;

            return Ok(());
        }
    };

    // Access the voice manager mutex
    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected voice manager in share map");
    // Gain a lock on the mutex
    let mut manager = manager_lock.lock();

    // Attempt to get the current vc of the bot
    if manager.get(guild_id).is_some() {
        // Leave the channel if the bot is in one
        manager.remove(guild_id);

        msg.channel_id.say(&ctx, "Left voice channel")?;
    } else {
        msg.channel_id.say(&ctx, "Bot is not in a voice channel")?;
    }

    Ok(())
}

#[command]
#[usage = "!play <song url>"]
#[example = "!play https://www.youtube.com/watch?v=dQw4w9WgXcQ"]
#[num_args(1)]
#[only_in(guild)]
/// Plays a song
fn play(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // Get the url to play
    let url = match args.rest() {
        url if url.starts_with("http") => url,
        // url => youtube search here to get url,
        _ => {
            msg.channel_id
                .say(&ctx, "Must provide a valid URL to video or audio")?;

            return Ok(());
        }
    };

    // Get the guild id from the message
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx, "Groups and DMs not supported")?;

            return Ok(());
        }
    };

    // Access the voice manager mutex
    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected voice manager in share map");
    // Gain a lock on the mutex
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match ytdl::stream_url(url) {
            Ok(source) => source,
            Err(why) => {
                eprintln!("Error with youtube-dl: {:?}", why);

                msg.channel_id.say(&ctx, "Error loading URL")?;

                return Ok(());
            }
        };

        handler.play(source);
        // let audio = handler.play_returning(source);
        // audio.lock().volume(0.5);

        msg.channel_id.say(&ctx, "Playing song")?;
    } else {
        msg.channel_id.say(&ctx, "Not in a voice channel")?;
    }

    Ok(())
}
