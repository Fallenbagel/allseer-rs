use regex::Regex;
use serenity::all::{
    CommandInteraction, ComponentInteraction, CreateEmbedAuthor, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateMessage, EditInteractionResponse,
};
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateInteractionResponseMessage};
use serenity::prelude::*;
use tracing::{debug, error};

use crate::utils::handler::HashContext;

#[derive(Debug)]
pub enum InteractionType {
    ComponentInteraction(ComponentInteraction),
    CommandInteraction(CommandInteraction),
}

pub async fn run(
    ctx: &Context,
    interaction: InteractionType,
    number: u64,
    context: &HashContext,
) -> Result<(), color_eyre::Report> {
    let page = octocrab::instance()
        .issues("Fallenbagel", "jellyseerr")
        .get(number)
        .await?;

    let body = match page.body {
        Some(body) => body.trim_matches('"').to_string(),
        None => "Could not find a description".to_string(),
    };

    let re = Regex::new(r"\n{2,}").unwrap();
    let sanitized_body = re.replace_all(&body, "\n");

    let re_heading = Regex::new(r"(?m)####").unwrap();
    let sanitized_body = re_heading.replace_all(&sanitized_body, "###");

    let author = CreateEmbedAuthor::new(&page.user.login)
        .url(format!("https://github.com/{}", page.user.login))
        .icon_url(page.user.avatar_url);

    let footer = CreateEmbedFooter::new(format!(
        "Created at: {}",
        page.created_at.format("%Y-%m-%d (%H:%M:%S)")
    ))
    .icon_url("https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png");

    let re_images = Regex::new(r"!\[.*?\]\((.*?)\)").unwrap();

    let mut embed = CreateEmbed::new();

    if let Some(captures) = re_images.captures(&sanitized_body) {
        if let Some(url) = captures.get(1) {
            let image_url = url.as_str();
            debug!("Image URL: {}", image_url);

            embed = embed.image(image_url);
        }
    }

    embed = embed
        .title(format!(
            "{} #{}",
            if context.is_issue { "Issue" } else { "PR" },
            page.number,
        ))
        .description(format!(
            "### [{}]({})\n{}",
            page.title, page.html_url, sanitized_body
        ))
        .colour(10181046)
        .author(author)
        .footer(footer);

    println!("interaction: {:#?}", interaction);

    match interaction {
        InteractionType::ComponentInteraction(interaction) => {
            let message = CreateInteractionResponseMessage::new().embed(embed);

            let builder = CreateInteractionResponse::Message(message);

            if let Err(why) = interaction.create_response(&ctx.http, builder).await {
                error!("Error sending response: {:?}", why);

                let error_message =
                    EditInteractionResponse::new().content("Error finding issues/prs");

                interaction.edit_response(&ctx.http, error_message).await?;
            }
        }
        InteractionType::CommandInteraction(interaction) => {
            let builder = CreateInteractionResponseFollowup::new().embed(embed);

            // interaction.create_response(&ctx.http, builder).await?;
            if let Err(why) = interaction.create_followup(&ctx.http, builder).await {
                error!("Error sending response: {:?}", why);

                let error_message = CreateMessage::new().content("Error finding issues/prs");

                interaction
                    .channel_id
                    .send_message(&ctx.http, error_message)
                    .await?;
            }
        }
    }

    Ok(())
}
