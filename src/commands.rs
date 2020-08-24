use crate::ytdl;
use crate::VoiceManager;
use chrono::prelude::*;
use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandError, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId, misc::Mentionable},
    prelude::Context,
    utils::Color,
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
#[commands(join, leave, play, stop /*, search */)]
/// The voice commands
struct Voice;

#[command]
fn stop(ctx: &mut Context, msg: &Message) -> CommandResult {
    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected voice manager in share map");
    // Gain a lock on the mutex
    let mut manager = manager_lock.lock();

    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx, "Groups and DMs not supported")?;

            return Ok(());
        }
    };
    if let Some(handler) = manager.get_mut(guild_id) {
        handler.stop();
    }

    Ok(())
}

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
#[usage = "!play <song url | song title>"]
#[example = "!play https://www.youtube.com/watch?v=dQw4w9WgXcQ"]
#[min_args(1)]
#[only_in(guild)]
/// Plays a song
fn play(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // Get the args
    let arg = args.rest();

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
        handler.stop();

        // Detect if given a url or given a search term
        if arg.starts_with("http") {
            let metadata = match ytdl::metadata(arg) {
                Ok(meta) => meta,
                Err(why) => {
                    eprintln!("Encountered an error while fetching the metadata: {}", why);
                    return Err(CommandError(format!("{}", why)));
                }
            };

            let source = match ytdl::stream_url(arg) {
                Ok(source) => source,
                Err(why) => {
                    eprintln!("Encountered an error while streaming the audio: {}", why);
                    return Err(CommandError(format!("{}", why)));
                }
            };
            handler.play(source);
            // let audio = handler.play_returning(source);
            // audio.lock().volume(0.5);

            msg.channel_id
                .send_message(&ctx, |m| {
                    m.embed(|e| {
                        e.title(format!("Playing: {}", metadata.fulltitle))
                            .url(&metadata.webpage_url)
                            .timestamp(
                                &Utc.datetime_from_str(&metadata.upload_date, "%Y%m%d")
                                    .unwrap_or_else(|_| Utc::now()),
                            )
                            .thumbnail(&metadata.thumbnail)
                            .author(|a| a.name(&metadata.uploader).url(&metadata.uploader_url))
                            .field("duration", &metadata.duration, true)
                            .field("view_count", &metadata.view_count, true)
                            .color(Color::DARK_GREEN)
                    })
                })
                .ok();
        } else {
            let results = match ytdl::search(arg, 1) {
                Ok(r) => r,
                Err(why) => {
                    eprintln!("Encountered an error while searching for the song: {}", why);
                    return Err(CommandError(format!("{}", why)));
                }
            };

            if results.len() == 0 {
                msg.channel_id.say(&ctx, "No results")?;
                return Ok(());
            }

            let metadata = &results[0];

            let source = match ytdl::stream_url(&metadata.webpage_url) {
                Ok(source) => source,
                Err(why) => {
                    eprintln!("Encountered an error while streaming the audio: {}", why);
                    return Err(CommandError(format!("{}", why)));
                }
            };
            handler.play(source);

            msg.channel_id
                .send_message(&ctx, |m| {
                    m.embed(|e| {
                        e.title(format!("Playing: {}", metadata.fulltitle))
                            .url(&metadata.webpage_url)
                            .timestamp(
                                &Utc.datetime_from_str(&metadata.upload_date, "%Y%m%d")
                                    .unwrap_or_else(|_| Utc::now()),
                            )
                            .thumbnail(&metadata.thumbnail)
                            .author(|a| a.name(&metadata.uploader).url(&metadata.uploader_url))
                            .field("duration", &metadata.duration, true)
                            .field("view_count", &metadata.view_count, true)
                            .color(Color::DARK_GREEN)
                    })
                })
                .ok();
        }
    } else {
        msg.channel_id.say(&ctx, "Not in a voice channel")?;
    }

    Ok(())
}
