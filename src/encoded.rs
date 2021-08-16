#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Encoded {
    Value(EncodedValue),
    Table(std::collections::HashMap<String, Encoded>),
    Array(Vec<Encoded>),
}

impl Encoded {
    pub fn from_slice(v: &[u8]) -> Result<Self, crate::Error> {
        serde_json::from_slice(v).map_err(crate::Error::new)
    }

    pub fn to_string_pretty(&self) -> Result<String, crate::Error> {
        serde_json::to_string_pretty(self).map_err(crate::Error::new)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type", content = "value")]
pub enum EncodedValue {
    String(String),
    Integer(String),
    Float(String),
    Bool(String),
    Datetime(String),
    DatetimeLocal(String),
    DateLocal(String),
    TimeLocal(String),
}

impl EncodedValue {
    pub fn as_str(&self) -> &str {
        match self {
            EncodedValue::String(v)
            | EncodedValue::Integer(v)
            | EncodedValue::Float(v)
            | EncodedValue::Bool(v)
            | EncodedValue::Datetime(v)
            | EncodedValue::DatetimeLocal(v)
            | EncodedValue::DateLocal(v)
            | EncodedValue::TimeLocal(v) => v.as_str(),
        }
    }
}

impl<'a> From<&'a str> for EncodedValue {
    fn from(other: &'a str) -> Self {
        EncodedValue::String(other.to_owned())
    }
}

impl<'a> From<&'a String> for EncodedValue {
    fn from(other: &'a String) -> Self {
        EncodedValue::String(other.clone())
    }
}

impl From<String> for EncodedValue {
    fn from(other: String) -> Self {
        EncodedValue::String(other)
    }
}

impl From<i64> for EncodedValue {
    fn from(other: i64) -> Self {
        EncodedValue::Integer(other.to_string())
    }
}

impl From<f64> for EncodedValue {
    fn from(other: f64) -> Self {
        let s = format!("{:.15}", other);
        let s = s.trim_end_matches('0');
        let s = if s.ends_with('.') {
            format!("{}0", s)
        } else {
            s.to_owned()
        };
        EncodedValue::Float(s)
    }
}

impl From<bool> for EncodedValue {
    fn from(other: bool) -> Self {
        EncodedValue::Bool(other.to_string())
    }
}
