use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{ComponentInteractionDataKind::StringSelect, Interaction};
use serenity::model::{gateway::Ready, id::GuildId};
use serenity::prelude::*;
use std::collections::HashMap;
use utils::handler::Handler;

use tracing::{debug, error, info};

use crate::handle_interaction::InteractionType;

mod commands;
mod handle_interaction;
mod utils;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(ref command) = interaction {
            let content: Option<String> = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "search" => {
                    commands::search::run(&ctx, command, self.context_map.clone())
                        .await
                        .ok();
                    None
                }
                "comments" => {
                    commands::comments::run(&ctx, command, self.context_map.clone())
                        .await
                        .ok();
                    None
                }
                "fetch" => {
                    commands::fetch::run(&ctx, command, self.context_map.clone())
                        .await
                        .ok();
                    None
                }
                _ => Some("Command not implemented".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Error sending response: {:?}", why);
                }
            }
        }

        if let Interaction::Component(ref component_interaction) = interaction {
            match &component_interaction.data.kind {
                StringSelect { values } => {
                    for value in values {
                        debug!("Received value: {}", value);

                        let number = value.parse::<u64>().unwrap();

                        let context_map = self.context_map.lock().await;
                        if let Some(context) = context_map.get(&number) {
                            debug!("Context for {}: {:?}", number, context);

                            let component_interaction = InteractionType::ComponentInteraction(
                                component_interaction.clone(),
                            );

                            let _ = handle_interaction::run(
                                &ctx,
                                component_interaction,
                                number,
                                context,
                            )
                            .await;
                        }
                    }
                }
                _ => {
                    error!("Interaction kind is not StringSelect");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    commands::ping::register(),
                    commands::search::register(),
                    commands::comments::register(),
                    commands::fetch::register(),
                ],
            )
            .await;

        debug!("{commands:#?}");
        println!("{commands:#?}")
    }
}

#[tokio::main]
async fn main() -> Result<(), color_eyre::Report> {
    dotenv::dotenv().ok();

    utils::diagnostics::setup()?;

    let token = env::var("DISCORD_TOKEN")?;

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            context_map: Arc::new(Mutex::new(HashMap::new())),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }

    Ok(())
}
