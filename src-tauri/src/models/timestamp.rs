use chrono::{DateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    date.timestamp().serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let ts = i64::deserialize(deserializer)?;
    Utc.timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {}", ts)))
}

pub mod optional {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.map(|date| date.timestamp()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ts = Option::<i64>::deserialize(deserializer)?;
        ts.map(|value| {
            Utc.timestamp_opt(value, 0)
                .single()
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {}", value)))
        })
        .transpose()
    }
}
