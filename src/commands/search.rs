use serenity::builder::*;
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::utils::handler::HashContext;
use crate::utils::search_type::{SearchStatus, SearchType};

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    context_map: Arc<Mutex<HashMap<u64, HashContext>>>,
) -> Result<(), color_eyre::Report> {
    let open = interaction
        .data
        .options
        .first()
        .unwrap()
        .value
        .as_str()
        .unwrap()
        == SearchStatus::Open.as_str();

    let issue = interaction
        .data
        .options
        .get(1)
        .unwrap()
        .value
        .as_str()
        .unwrap()
        == SearchType::Issues.as_str();

    let query = interaction
        .data
        .options
        .get(2)
        .unwrap()
        .value
        .as_str()
        .unwrap();

    let response = format!(
        "Searching for {} {} with: {}",
        if issue { "issues" } else { "pull requests" },
        if open { "open" } else { "closed" },
        query
    );

    let data = CreateInteractionResponseMessage::new().content(response);

    let builder = CreateInteractionResponse::Message(data);
    interaction.create_response(&ctx.http, builder).await?;

    search_github(query, issue, open, ctx, interaction, context_map).await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("search")
        .description("Search for github issues/pull requests")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "status", "Search Status")
                .add_string_choice(SearchStatus::Open.as_str(), SearchStatus::Open.as_str())
                .add_string_choice(SearchStatus::Closed.as_str(), SearchStatus::Closed.as_str())
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "type",
                "Search for issues or pull requests",
            )
            .add_string_choice(SearchType::Issues.as_str(), SearchType::Issues.as_str())
            .add_string_choice(
                SearchType::PullRequests.as_str(),
                SearchType::PullRequests.as_str(),
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "query", "Search query")
                .required(true),
        )
}

async fn search_github(
    query: &str,
    issue: bool,
    open: bool,
    ctx: &Context,
    interaction: &CommandInteraction,
    context_map: Arc<Mutex<HashMap<u64, HashContext>>>,
) -> Result<(), color_eyre::Report> {
    let search_query = format!(
        "user:Fallenbagel repo:jellyseerr {} {} {}",
        if issue { "type:issue" } else { "type:pr" },
        if open { "is:open" } else { "is:closed" },
        query
    );

    let page = octocrab::instance()
        .search()
        .issues_and_pull_requests(&search_query)
        .sort("comments")
        .order("desc")
        .send()
        .await?;

    let mut options = Vec::new();

    for item in page.items {
        if item.number == 0 {
            break;
        }

        let title = item.title;
        let number: u64 = item.number;

        let mut context = context_map.lock().await;

        // store whether it's an issue or PR in context so it can be retrieved
        // during the interaction
        context.insert(number, HashContext { is_issue: issue });

        options.push(CreateSelectMenuOption::new(
            title.to_string(),
            number.to_string(),
        ));

        if options.len() >= 25 {
            break;
        }
    }

    if options.is_empty() {
        let message = CreateInteractionResponseFollowup::default()
            .content("No results found")
            .tts(false);

        interaction.create_followup(&ctx.http, message).await?;

        return Ok(());
    }

    let select_menu_kind = CreateSelectMenuKind::String { options };

    let select_menu = CreateSelectMenu::new("search_results", select_menu_kind);

    let message = CreateInteractionResponseFollowup::default()
        .content(format!(
            "Select {}",
            if issue { "an issue" } else { "a pull request" }
        ))
        .select_menu(select_menu);

    interaction.create_followup(&ctx.http, message).await?;

    Ok(())
}
