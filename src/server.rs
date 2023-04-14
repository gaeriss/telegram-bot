use teloxide::prelude::Requester;

pub type SharedServer = std::sync::Arc<tokio::sync::Mutex<Server>>;

#[derive(Clone)]
pub struct Server {
    sumup: sumup::SumUp,
    allowed_chat: Vec<i64>,
}

impl Server {
    pub fn new() -> crate::MyResult<Self> {
        let config = sumup::Config {
            client_id: envir::get("SUMUP_CLIENT_ID")?,
            client_secret: envir::get("SUMUP_CLIENT_SECRET")?,
            username: envir::try_get("SUMUP_USERNAME")?,
            password: envir::try_get("SUMUP_PASSWORD")?,
            grant_type: sumup::config::GrantType::Password,

            ..Default::default()
        };

        let sumup = sumup::SumUp::from(config)?;

        let allowed_chat = envir::get("ALLOWED_CHAT_ID")?
            .split(',')
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?;

        Ok(Self {
            allowed_chat,
            sumup,
        })
    }

    pub async fn refresh_token(server: &SharedServer) -> crate::MyResult {
        let expires_in = server
            .lock()
            .await
            .sumup
            .access_token()
            .expires_in
            .unwrap_or(5 * 60);
        let duration = std::time::Duration::new(expires_in.into(), 0);
        tokio::time::sleep(duration).await;

        server.lock().await.sumup.refresh_token(None)?;

        log::info!("token refreshed");

        Ok(())
    }

    pub fn is_allowed(&self, chat_id: teloxide::types::ChatId) -> bool {
        self.allowed_chat.is_empty() || self.allowed_chat.contains(&chat_id.0)
    }

    pub async fn about(
        &self,
        bot: &teloxide::Bot,
        msg: &teloxide::types::Message,
    ) -> crate::MyResult {
        bot.send_message(msg.chat.id, env!("CARGO_PKG_REPOSITORY"))
            .await?;

        Ok(())
    }

    pub async fn help(
        &self,
        bot: &teloxide::Bot,
        msg: &teloxide::types::Message,
    ) -> crate::MyResult {
        use teloxide::utils::command::BotCommands;

        bot.send_message(msg.chat.id, crate::Command::descriptions().to_string())
            .await?;

        Ok(())
    }

    pub async fn cb(
        &self,
        bot: &teloxide::Bot,
        msg: &teloxide::types::Message,
        date: chrono::NaiveDate,
    ) -> crate::MyResult {
        let end = date
            .and_hms_opt(2, 0, 0)
            .unwrap_or_else(|| chrono::offset::Local::now().naive_local());
        let start = end - chrono::Duration::hours(12);

        let filter = sumup::services::transactions::Filter {
            start_date: start.format("%Y-%m-%d").to_string(),
            end_date: end.format("%Y-%m-%d").to_string(),
            oldest_time: Some(start.format("%Y-%m-%dT%H:%M").to_string()),
            newest_time: Some(end.format("%Y-%m-%dT%H:%M").to_string()),

            ..Default::default()
        };

        let transactions = self.sumup.transactions().history(&filter)?;
        let total = transactions.iter().map(|x| x.amount).sum::<f32>();

        bot.send_message(msg.chat.id, format!("{total} €")).await?;

        Ok(())
    }
}
