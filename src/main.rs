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

    let shared_server = std::sync::Arc::new(tokio::sync::Mutex::new(Server::new()?));
    let bot = teloxide::Bot::from_env();

    let server = shared_server.clone();
    let _ = tokio::spawn(async move {
        loop {
            if let Err(err) = Server::refresh_token(&server).await {
                log::error!("{err}");
            }
        }
    }).await;

    let handler = teloxide::types::Update::filter_message()
        .filter_command::<Command>()
        .endpoint(command);

    teloxide::dispatching::Dispatcher::builder(bot, handler)
        .dependencies(teloxide::dptree::deps![shared_server])
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
    cmd: Command,
    server: SharedServer,
) -> MyResult {
    let server = server.lock().await;

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
