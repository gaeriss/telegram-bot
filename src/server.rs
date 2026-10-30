use teloxide::prelude::Requester;

pub type SharedServer = std::sync::Arc<tokio::sync::Mutex<Server>>;

#[derive(Clone)]
pub struct Server {
    sumup: sumup::Client,
    allowed_chat: Vec<i64>,
}

impl Server {
    pub fn new() -> crate::MyResult<Self> {
        let sumup = sumup::Client::default()
            .with_authorization(sumup::Authorization::APIKey(envir::get("SUMUP_API_KEY")?));

        let allowed_chat = envir::get("ALLOWED_CHAT_ID")?
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
        date: jiff::civil::Date,
    ) -> crate::MyResult {
        use jiff::ToSpan as _;

        let merchant_code = envir::get("SUMUP_MERCHANT_CODE")?;

        let end = date
            .at(2, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap_or_else(|_| jiff::Zoned::now());
        let start = end.clone() - 12.hours();

        let params = sumup::resources::transactions::ListParams {
            oldest_time: Some(start.timestamp()),
            newest_time: Some(end.timestamp()),

            ..Default::default()
        };

        let transactions = self
            .sumup
            .transactions()
            .list(merchant_code, params)
            .await?;
        let total = transactions
            .items
            .unwrap_or_default()
            .iter()
            .filter_map(|x| x.amount)
            .sum::<f32>();

        bot.send_message(msg.chat.id, format!("{total} €")).await?;

        Ok(())
    }
}
