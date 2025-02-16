use chrono::{DateTime, Datelike, Locale, Utc};
use chrono::{Duration, Local};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    Command::repl(bot, proccess_commands).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Эти команды доступны:")]
enum Command {
    #[command(description = "Приветственное сообщение")]
    Help,
    #[command(description = "Приветственное сообщение")]
    Start,
    #[command(description = "Добавить пару")]
    Add,
    #[command(description = "Получить расписание на сегодня")]
    Day,
    #[command(description = "Получить расписание на завтра")]
    Tomorrow,
    #[command(description = "Получить расписание на неделю")]
    Week,
    #[command(description = "Получить расписание на следующую неделю")]
    Nweek,
    #[command(description = "Удалить пару")]
    Delete,
}

async fn proccess_commands(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help | Command::Start => default(bot, msg).await?,
        Command::Day | Command::Tomorrow | Command::Nweek | Command::Week => {
            get_schedule(bot, msg, cmd).await?
        }
        Command::Add => todo!(),
        Command::Delete => todo!(),
    }

    Ok(())
}

async fn default(bot: Bot, msg: Message) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        "Привет! Я бот, который показывает расписание. \n
        Доступные команды: \n
        /day - расписание на сегодня \n
        /tomorrow - расписание на завтра \n
        /week - расписание на эту неделю \n
        /nweek - расписание на следующую неделю \n
        /add - добавить занятие в расписание \n
        /delete - удалить добавленное занятие",
    )
    .await?;
    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
struct Lesson {
    name: String,
    #[serde(rename = "type")]
    lesson_type: String,
    duration: i64,
    cabinet: String,
    #[serde(with = "serialze_datetime")]
    date: DateTime<Utc>,
}

mod serialze_datetime {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%d-%m-%Y %H:%M";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

fn load_schedule() -> io::Result<Vec<Lesson>> {
    let data = fs::read_to_string("schedule.json")?;
    let lessons: Vec<Lesson> = serde_json::from_str(&data).unwrap();
    Ok(lessons)
}

async fn get_schedule(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let schedule = load_schedule()?;
    let mut start = Local::now();
    let mut end = start + Duration::days(1);
    match cmd {
        Command::Tomorrow => {
            start += Duration::days(1);
            end += Duration::days(1);
        }
        Command::Week => {
            start -= Duration::days(start.weekday() as i64);
            end = start + Duration::days(7);
        }
        Command::Nweek => {
            start += Duration::days(7 - start.weekday() as i64);
            end = start + Duration::days(7);
        }
        _ => {}
    }
    for day in 0..(end - start).num_days() {
        let curr = start + Duration::days(day);
        let mut message = format!(
            "<b>{}</b>\n\n",
            curr.format_localized("%d %B (%A)", Locale::ru_RU)
        );
        let mut lessons: Vec<String> = Vec::new();
        for lesson in schedule.iter() {
            if lesson.date.date_naive() == curr.date_naive() {
                if lesson.cabinet != "" {
                    lessons.push(format!(
                        "{}-{}\n<b>{}</b>\n{} ({})\n",
                        lesson.date.format("%H:%M"),
                        (lesson.date + Duration::minutes(lesson.duration)).format("%H:%M"),
                        lesson.name,
                        lesson.lesson_type,
                        lesson.cabinet
                    ))
                } else {
                    lessons.push(format!(
                        "{}-{}\n<b>{}</b>\n{}\n",
                        lesson.date.format("%H:%M"),
                        (lesson.date + Duration::minutes(lesson.duration)).format("%H:%M"),
                        lesson.name,
                        lesson.lesson_type
                    ))
                }
            }
        }
        if lessons.len() > 0 {
            message.push_str(&lessons.join("------\n\n"));
        } else {
            message.push_str("Нет пар");
        }
        bot.send_message(msg.chat.id, message)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    }
    Ok(())
}
