use chrono::{Datelike, Duration, Local, Locale, NaiveDateTime};
use dptree::case;
use teloxide::{
    dispatching::dialogue::{self, InMemStorage},
    prelude::*,
    utils::command::BotCommands,
};
mod schedule;
mod state;
use state::State;
mod utils;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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
    #[command(description = "Отмена")]
    Cancel,
}

#[tokio::main]
async fn main() {
    let bot = Bot::from_env();

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Add].endpoint(add_lesson_handler))
        .branch(case![Command::Delete].endpoint(delete_lesson_handler))
        .branch(case![Command::Start].endpoint(default))
        .branch(case![Command::Help].endpoint(default))
        .branch(case![Command::Cancel].endpoint(cancel))
        .branch(dptree::endpoint(get_schedule));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(
            case![State::AddLesson {
                name,
                time,
                duration,
                lesson_type,
                cabinet
            }]
            .endpoint(manage_lessons_handler),
        )
        .branch(
            case![State::DeleteLesson {
                name,
                time,
                duration,
                lesson_type,
                cabinet
            }]
            .endpoint(manage_lessons_handler),
        );

    Dispatcher::builder(
        bot,
        dialogue::enter::<Update, InMemStorage<State>, State, _>().branch(message_handler),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn default(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Привет! Я бот, который показывает расписание. \n
<b>Доступные команды:</b> \n
/day - расписание на сегодня \n
/tomorrow - расписание на завтра \n
/week - расписание на эту неделю \n
/nweek - расписание на следующую неделю \n
/add - добавить занятие в расписание \n
/delete - удалить добавленное занятие",
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;
    Ok(())
}

async fn cancel(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
) -> HandlerResult {
    dialogue.update(State::Default).await?;
    bot.send_message(msg.chat.id, "Команда сброшена").await?;
    Ok(())
}

async fn manage_lessons_handler(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
    mut state: State,
) -> HandlerResult {
    let Some(text) = msg.text() else {
        bot.send_message(msg.chat.id, "Сообщение не содержит текст")
            .await?;
        return Ok(());
    };
    let (name, time, duration, lesson_type, cabinet, add) = match &mut state {
        State::AddLesson {
            name,
            time,
            duration,
            lesson_type,
            cabinet,
        } => (name, time, duration, lesson_type, cabinet, true),
        State::DeleteLesson {
            name,
            time,
            duration,
            lesson_type,
            cabinet,
        } => (name, time, duration, lesson_type, cabinet, false),
        _ => {
            bot.send_message(msg.chat.id, "Произошла ошибка, попробуйте ещё раз.")
                .await?;
            return Ok(());
        }
    };

    let next_step: &str;

    if name.is_none() {
        *name = Some(text.to_string());
        next_step = "Введите время в формате <b>19.02.2025 10:25</b>.";
    } else if time.is_none() {
        let parsed_time = match NaiveDateTime::parse_from_str(text.trim(), "%d.%m.%Y %H:%M") {
            Ok(time) => time,
            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "Введена дата в неподходящем формате. Введите время в формате: <b>19.02.2025 10:25</b>."
                ).parse_mode(teloxide::types::ParseMode::Html).await?;
                return Ok(());
            }
        };
        *time = Some(parsed_time.and_utc());
        next_step = "Введите продолжительность в формате: <b>80</b>.";
    } else if duration.is_none() {
        let parsed_duration = match text.trim().parse::<i64>() {
            Ok(time) => time,
            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "Введена некорректная продолжительность. Введите продолжительность в формате: <b>80</b>."
                ).parse_mode(teloxide::types::ParseMode::Html).await?;
                return Ok(());
            }
        };
        *duration = Some(parsed_duration);
        next_step = "Введите тип занятия (лекция/семинар/etc).";
    } else if lesson_type.is_none() {
        *lesson_type = Some(text.to_string());
        next_step = "Введите место проведения занятия.";
    } else if cabinet.is_none() {
        *cabinet = Some(text.to_string());
        next_step = "Готово. Введите что угодно, или нажмите /cancel.";
    } else {
        let lesson = schedule::Lesson {
            name: name.take().unwrap(),
            lesson_type: lesson_type.take().unwrap(),
            duration: duration.take().unwrap(),
            cabinet: cabinet.take().unwrap(),
            date: time.take().unwrap(),
        };
        if add {
            schedule::add(lesson)?;
            next_step = "Занятие добавлено.";
        } else {
            schedule::delete(&lesson)?;
            next_step = "Занятие удалено.";
        }
        state = State::Default;
    }

    bot.send_message(
        msg.chat.id,
        format!("{}\n\n{}", utils::state_message(state.clone()), next_step),
    )
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;
    dialogue.update(state).await?;
    Ok(())
}

async fn add_lesson_handler(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
) -> HandlerResult {
    dialogue
        .update(State::AddLesson {
            name: None,
            time: None,
            duration: None,
            lesson_type: None,
            cabinet: None,
        })
        .await?;
    bot.send_message(msg.chat.id, "Введите название дисциплины:")
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn delete_lesson_handler(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue<State, InMemStorage<State>>,
) -> HandlerResult {
    dialogue
        .update(State::DeleteLesson {
            name: None,
            time: None,
            duration: None,
            lesson_type: None,
            cabinet: None,
        })
        .await?;
    bot.send_message(msg.chat.id, "Введите название дисциплины:")
        .parse_mode(teloxide::types::ParseMode::Html)
        .await?;
    Ok(())
}

async fn get_schedule(bot: Bot, msg: Message, cmd: Command) -> HandlerResult {
    let schedule = schedule::load()?;
    let mut start = Local::now().to_utc();
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
        let curr = curr.date_naive();
        let lessons = schedule
            .iter()
            .filter(|lesson| lesson.date.date_naive() == curr)
            .map(|lesson| {
                format!(
                    "{}-{}\n<b>{}</b>\n{} {}",
                    lesson.date.format("%H:%M"),
                    (lesson.date + Duration::minutes(lesson.duration)).format("%H:%M"),
                    lesson.name,
                    lesson.lesson_type,
                    if lesson.cabinet.is_empty() {
                        ""
                    } else {
                        &lesson.cabinet
                    }
                )
            })
            .collect::<Vec<_>>()
            .join("\n------\n\n");

        if lessons.is_empty() {
            message.push_str("Нет пар");
        } else {
            message.push_str(&lessons);
        }
        bot.send_message(msg.chat.id, message)
            .parse_mode(teloxide::types::ParseMode::Html)
            .await?;
    }

    Ok(())
}
