use teloxide::prelude::Requester;

#[derive(Clone)]
pub struct Server {
    sumup: sumup::SumUp,
    allowed_chat: Vec<i64>,
}

impl Server {
    pub fn new() -> crate::MyResult<Self> {
        let config = sumup::Config {
            client_id: env("SUMUP_CLIENT_ID"),
            client_secret: env("SUMUP_CLIENT_SECRET"),
            username: Some(env("SUMUP_USERNAME")),
            password: Some(env("SUMUP_PASSWORD")),
            grant_type: sumup::config::GrantType::Password,

            ..Default::default()
        };

        let sumup = sumup::SumUp::from(config)?;

        let allowed_chat = env("ALLOWED_CHAT_ID")
            .split(',')
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?;

        Ok(Self {
            allowed_chat,
            sumup,
        })
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

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Missing {name} env variable"))
}
