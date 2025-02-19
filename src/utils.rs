use chrono::{Duration, Utc};

use crate::state::State;

pub fn state_message(state: State) -> String {
    let (name, time, duration, lesson_type, cabinet) = match state {
        State::AddLesson {
            name,
            time,
            duration,
            lesson_type,
            cabinet,
        } => (name, time, duration, lesson_type, cabinet),
        State::DeleteLesson {
            name,
            time,
            duration,
            lesson_type,
            cabinet,
        } => (name, time, duration, lesson_type, cabinet),
        _ => {
            return "".into();
        }
    };

    let time = time.unwrap_or(Utc::now());
    let duration = time + Duration::minutes(duration.unwrap_or(80));
    let name = name.unwrap_or("Дисциплина".into());
    let lesson_type = lesson_type.unwrap_or("Тип".into());
    let cabinet = cabinet.unwrap_or("Кабинет".into());

    format!(
        "{}-{}\n<b>{}</b>\n{} {}",
        time.format("%d.%m.%Y: %H:%M"),
        duration.format("%H:%M"),
        name,
        lesson_type,
        cabinet
    )
}
