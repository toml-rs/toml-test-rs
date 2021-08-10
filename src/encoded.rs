#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Encoded {
    Value(EncodedValue),
    Table(std::collections::HashMap<String, Encoded>),
    Array(Vec<Encoded>),
}

impl Encoded {
    pub fn from_slice(v: &[u8]) -> Result<Self, crate::Error> {
        serde_json::from_slice(v).map_err(|e| crate::Error::new(e.to_string()))
    }

    pub fn to_string_pretty(&self) -> Result<String, crate::Error> {
        serde_json::to_string_pretty(self).map_err(|e| crate::Error::new(e.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EncodedValue {
    #[serde(rename = "type")]
    toml_type: EncodedValueType,
    value: String,
}

impl<'a> From<&'a str> for EncodedValue {
    fn from(other: &'a str) -> Self {
        Self {
            toml_type: EncodedValueType::String,
            value: other.to_owned(),
        }
    }
}

impl From<String> for EncodedValue {
    fn from(other: String) -> Self {
        Self {
            toml_type: EncodedValueType::String,
            value: other,
        }
    }
}

impl From<i64> for EncodedValue {
    fn from(other: i64) -> Self {
        Self {
            toml_type: EncodedValueType::Integer,
            value: other.to_string(),
        }
    }
}

impl From<bool> for EncodedValue {
    fn from(other: bool) -> Self {
        Self {
            toml_type: EncodedValueType::Integer,
            value: other.to_string(),
        }
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
        Self {
            toml_type: EncodedValueType::Integer,
            value: s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EncodedValueType {
    String,
    Integer,
    Float,
    Bool,
    Datetime,
    DatetimeLocal,
    DateLocal,
    TimeLocal,
}
