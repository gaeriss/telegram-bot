mod errors;
mod server;

use errors::*;
use server::*;

#[tokio::main]
async fn main() -> crate::MyResult {
    use teloxide::dispatching::{HandlerExt, UpdateFilterExt};

    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    env_logger::init();

    let server = Server::new()?;
    let bot = teloxide::Bot::from_env();

    let handler = teloxide::types::Update::filter_message()
        .filter_command::<Command>()
        .endpoint(command);

    teloxide::prelude::Dispatcher::builder(bot, handler)
        .dependencies(teloxide::dptree::deps![server])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(teloxide::utils::command::BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "À propos de moi")]
    About,
    #[command(description = "Affiche la somme des transactions CB des 12 dernières heures", parse_with = date)]
    Cb { date: chrono::NaiveDate },
    #[command(description = "Affiche ce message")]
    Help,
}

fn date(input: String) -> Result<(chrono::NaiveDate,), teloxide::utils::command::ParseError> {
    let date = input
        .parse()
        .unwrap_or_else(|_| chrono::offset::Local::now().date_naive());

    Ok((date,))
}

async fn command(
    bot: teloxide::Bot,
    msg: teloxide::types::Message,
    server: Server,
    cmd: Command,
) -> MyResult {
    if !server.is_allowed(msg.chat.id) {
        return Err(Error::Auth);
    }

    match cmd {
        Command::About => server.about(&bot, &msg).await?,
        Command::Cb { date } => server.cb(&bot, &msg, date).await?,
        Command::Help => server.help(&bot, &msg).await?,
    };

    Ok(())
}
