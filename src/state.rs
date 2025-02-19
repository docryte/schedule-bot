use chrono::{DateTime, Utc};

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Default,
    AddLesson {
        name: Option<String>,
        time: Option<DateTime<Utc>>,
        duration: Option<i64>,
        lesson_type: Option<String>,
        cabinet: Option<String>,
    },
    DeleteLesson {
        name: Option<String>,
        time: Option<DateTime<Utc>>,
        duration: Option<i64>,
        lesson_type: Option<String>,
        cabinet: Option<String>,
    },
}
