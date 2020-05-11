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
#[commands(ping, join)]
struct General;

#[command]
#[description = "ping the bot"]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx, "pong!")?;

    Ok(())
}

#[command]
#[only_in(guild)]
fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            msg.channel_id.say(&ctx, "Groups and DMs not supported")?;

            return Ok(());
        }
    };
    let guild_id = guild.read().id;

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

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected voice manager in share map");

    let mut manager = manager_lock.lock();

    if manager.join(guild_id, channel_id).is_some() {
        msg.channel_id
            .say(&ctx, &format!("Joined {}", channel_id.mention()))?;
    } else {
        msg.channel_id.say(&ctx, "Error joining the channel")?;
    }

    Ok(())
}
