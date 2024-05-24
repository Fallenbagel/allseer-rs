use serenity::builder::*;
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

use crate::utils::handler::HashContext;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    _context_map: Arc<Mutex<HashMap<u64, HashContext>>>,
) -> Result<(), color_eyre::Report> {
    let pr_number = interaction
        .data
        .options
        .first()
        .unwrap()
        .value
        .as_str()
        .unwrap()
        .parse::<u64>()?;

    let page = octocrab::instance()
        .issues("Fallenbagel", "jellyseerr")
        .list_comments(pr_number)
        .since(chrono::Local::now() - chrono::Duration::days(1))
        .per_page(100)
        .page(1u32)
        .send()
        .await?;

    let embed = CreateEmbed::new();

    for item in page.items {
        let mut body = match item.body {
            Some(body) => body.trim_matches('"').to_string(),
            None => "Could not find a description".to_string(),
        };

        if body.len() > 256 {
            body.truncate(256);
        }

        // let mut response = MessageBuilder::new();

        let author = CreateEmbedAuthor::new(&item.user.login)
            .url(format!("https://github.com/{}", item.user.login))
            .icon_url(item.user.avatar_url);

        let footer = CreateEmbedFooter::new(format!(
            "Created at: {}",
            item.created_at.format("%Y-%m-%d (%H:%M:%S)")
        ))
        .icon_url("https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png");

        let mut embed = embed.clone();

        embed = embed
            .title(format!(
                "Comment for [#{}]({:#?})",
                pr_number, item.issue_url
            ))
            .description(body)
            .colour(10181046)
            .author(author)
            .footer(footer);

        let message = CreateMessage::new().embed(embed);

        match interaction
            .channel_id
            .send_message(&ctx.http, message)
            .await
        {
            Ok(_) => {}
            Err(why) => {
                error!("Error sending response: {:?}", why);
            }
        }
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("comments")
        .description("Get comments for a pr")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "pr_number",
                "The number of the PR",
            )
            .required(true),
        )
}
