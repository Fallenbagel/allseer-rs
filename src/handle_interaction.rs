use regex::Regex;
use serenity::all::{ComponentInteraction, CreateEmbedAuthor, CreateInteractionResponse};
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateInteractionResponseMessage};
use serenity::prelude::*;
use tracing::debug;

use crate::utils::handler::HashContext;

pub async fn run(
    ctx: &Context,
    interaction: &ComponentInteraction,
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

    let message = CreateInteractionResponseMessage::new().embed(embed);

    let builder = CreateInteractionResponse::Message(message);

    interaction.create_response(&ctx.http, builder).await?;

    Ok(())
}
