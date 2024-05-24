use serenity::builder::*;
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use crate::handle_interaction::run as handle_interaction;
use crate::handle_interaction::InteractionType;
use crate::utils::handler::HashContext;
use crate::utils::search_type::SearchType;

pub async fn run(
    ctx: &Context,
    interaction: &CommandInteraction,
    context_map: Arc<Mutex<HashMap<u64, HashContext>>>,
) -> Result<(), color_eyre::Report> {
    let issue = interaction
        .data
        .options
        .first()
        .unwrap()
        .value
        .as_str()
        .unwrap()
        == SearchType::Issues.as_str();

    let pr_number = &interaction.data.options.get(1).unwrap().value;

    // panic!("Issue: {}, PR Number: {:#?}", issue, pr_number);

    let number: u64 = pr_number.as_i64().unwrap() as u64;

    let response = format!(
        "Fetching {} #{}",
        if issue { "issue" } else { "pull request" },
        number
    );

    let data = CreateInteractionResponseMessage::new().content(response);

    let builder = CreateInteractionResponse::Message(data);

    interaction.create_response(&ctx.http, builder).await?;

    {
        let mut context = context_map.lock().await;

        // Store whether it's an issue or PR in context so it can be retrieved
        // during the interaction
        context.insert(number, HashContext { is_issue: issue });
    }

    // store whether it's an issue or PR in context so it can be retrieved
    // during the interaction
    // context.insert(number, HashContext { is_issue: issue });

    let context_map = context_map.lock().await;

    if let Some(context) = context_map.get(&number) {
        debug!("Context for {}: {:?}", number, context);

        let interaction = InteractionType::CommandInteraction(interaction.clone());

        handle_interaction(ctx, interaction, number, context).await?;
    }

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("fetch")
        .description("Fetch a github issue/pull request")
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
            CreateCommandOption::new(CommandOptionType::Integer, "#", "The issue/pr number")
                .required(true),
        )
}
