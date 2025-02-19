use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Lesson {
    pub name: String,
    #[serde(rename = "type")]
    pub lesson_type: String,
    pub duration: i64,
    pub cabinet: String,
    #[serde(with = "serialze_datetime")]
    pub date: DateTime<Utc>,
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

pub fn load() -> io::Result<Vec<Lesson>> {
    let data = fs::read_to_string("schedule.json").unwrap_or_else(|_| String::from("[]"));
    let lessons: Vec<Lesson> = serde_json::from_str(&data)?;
    Ok(lessons)
}

pub fn add(lesson: Lesson) -> io::Result<()> {
    let mut lessons = load()?;
    lessons.push(lesson);

    let json = serde_json::to_string(&lessons)?;
    fs::write("schedule.json", json)?;

    Ok(())
}

pub fn delete(lesson: &Lesson) -> io::Result<()> {
    let mut lessons = load()?;
    lessons.retain(|l| l != lesson);

    let json = serde_json::to_string(&lessons)?;
    fs::write("schedule.json", json)?;

    Ok(())
}
