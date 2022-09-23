use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Decoded {
    Value(DecodedValue),
    Table(std::collections::HashMap<String, Decoded>),
    Array(Vec<Decoded>),
}

impl Decoded {
    pub fn from_slice(v: &[u8]) -> Result<Self, crate::Error> {
        serde_json::from_slice(v).map_err(|e| {
            crate::Error::new(format!(
                "failed decoding: {}\n```json\n{}\n```",
                e,
                String::from_utf8_lossy(v)
            ))
        })
    }

    pub fn to_string_pretty(&self) -> Result<String, crate::Error> {
        serde_json::to_string_pretty(self).map_err(crate::Error::new)
    }

    /// External parser helper for [`verify`][crate::verify]
    pub fn from_stdin() -> Result<Self, crate::Error> {
        let mut buf = Vec::new();
        std::io::stdin()
            .read_to_end(&mut buf)
            .map_err(crate::Error::new)?;
        Self::from_slice(&buf)
    }

    /// External parser helper for [`verify`][crate::verify]
    pub fn into_stdout(&self) -> Result<(), crate::Error> {
        let s = self.to_string_pretty()?;
        std::io::stdout()
            .write_all(s.as_bytes())
            .map_err(crate::Error::new)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "type", content = "value")]
pub enum DecodedValue {
    String(String),
    Integer(String),
    Float(String),
    Bool(String),
    Datetime(String),
    DatetimeLocal(String),
    DateLocal(String),
    TimeLocal(String),
}

impl DecodedValue {
    pub fn as_str(&self) -> &str {
        match self {
            DecodedValue::String(v)
            | DecodedValue::Integer(v)
            | DecodedValue::Float(v)
            | DecodedValue::Bool(v)
            | DecodedValue::Datetime(v)
            | DecodedValue::DatetimeLocal(v)
            | DecodedValue::DateLocal(v)
            | DecodedValue::TimeLocal(v) => v.as_str(),
        }
    }
}

impl<'a> From<&'a str> for DecodedValue {
    fn from(other: &'a str) -> Self {
        DecodedValue::String(other.to_owned())
    }
}

impl<'a> From<&'a String> for DecodedValue {
    fn from(other: &'a String) -> Self {
        DecodedValue::String(other.clone())
    }
}

impl From<String> for DecodedValue {
    fn from(other: String) -> Self {
        DecodedValue::String(other)
    }
}

impl From<i64> for DecodedValue {
    fn from(other: i64) -> Self {
        DecodedValue::Integer(other.to_string())
    }
}

impl From<f64> for DecodedValue {
    fn from(other: f64) -> Self {
        let s = if other.is_nan() {
            "nan".to_owned()
        } else if other.is_infinite() && other.is_sign_negative() {
            "-inf".to_owned()
        } else if other.is_infinite() && other.is_sign_positive() {
            "inf".to_owned()
        } else {
            let s = format!("{:.15}", other);
            let s = s.trim_end_matches('0');
            if s.ends_with('.') {
                format!("{}0", s)
            } else {
                s.to_owned()
            }
        };
        DecodedValue::Float(s)
    }
}

impl From<bool> for DecodedValue {
    fn from(other: bool) -> Self {
        DecodedValue::Bool(other.to_string())
    }
}

impl PartialEq for DecodedValue {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::if_same_then_else)]
        match (self, other) {
            (DecodedValue::String(s), DecodedValue::String(o)) => s == o,
            (DecodedValue::Integer(s), DecodedValue::Integer(o)) => s == o,
            (DecodedValue::Float(s), DecodedValue::Float(o)) => {
                if s == "inf" && o == "+inf" {
                    true
                } else if s == "+inf" && o == "inf" {
                    true
                } else if s == "nan" && o == "nan" {
                    true
                } else {
                    let s = s.parse::<f64>().unwrap();
                    let o = o.parse::<f64>().unwrap();
                    s == o
                }
            }
            (DecodedValue::Bool(s), DecodedValue::Bool(o)) => s == o,
            (DecodedValue::Datetime(s), DecodedValue::Datetime(o)) => {
                parse_date_time(s) == parse_date_time(o)
            }
            (DecodedValue::DatetimeLocal(s), DecodedValue::DatetimeLocal(o)) => {
                parse_date_time_local(s) == parse_date_time_local(o)
            }
            (DecodedValue::DateLocal(s), DecodedValue::DateLocal(o)) => {
                parse_date_local(s) == parse_date_local(o)
            }
            (DecodedValue::TimeLocal(s), DecodedValue::TimeLocal(o)) => {
                parse_time_local(s) == parse_time_local(o)
            }
            (_, _) => false,
        }
    }
}

fn parse_date_time(s: &str) -> chrono::DateTime<chrono::FixedOffset> {
    match normalize_datetime(s).parse() {
        Ok(d) => d,
        Err(err) => panic!("Failed to parse {:?}: {}", s, err),
    }
}

fn parse_date_time_local(s: &str) -> chrono::NaiveDateTime {
    match normalize_datetime(s).parse() {
        Ok(d) => d,
        Err(err) => panic!("Failed to parse {:?}: {}", s, err),
    }
}

fn parse_date_local(s: &str) -> chrono::NaiveDate {
    match s.parse() {
        Ok(d) => d,
        Err(err) => panic!("Failed to parse {:?}: {}", s, err),
    }
}

fn parse_time_local(s: &str) -> chrono::NaiveTime {
    match s.parse() {
        Ok(d) => d,
        Err(err) => panic!("Failed to parse {:?}: {}", s, err),
    }
}

fn normalize_datetime(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => 'T',
            't' => 'T',
            'z' => 'Z',
            _ => c,
        })
        .collect()
}

impl Eq for DecodedValue {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string_equality() {
        assert_eq!(DecodedValue::from("foo"), DecodedValue::from("foo"));
        assert_ne!(DecodedValue::from("foo"), DecodedValue::from("bar"));
        assert_ne!(DecodedValue::from("42"), DecodedValue::from(42));
        assert_ne!(DecodedValue::from("true"), DecodedValue::from(true));
    }

    #[test]
    fn integer_equality() {
        assert_eq!(DecodedValue::from(42), DecodedValue::from(42));
        assert_ne!(DecodedValue::from(42), DecodedValue::from(21));
        assert_ne!(DecodedValue::from(42), DecodedValue::from("42"));
    }

    #[test]
    fn float_equality() {
        assert_eq!(DecodedValue::from(42.0), DecodedValue::from(42.0));
        assert_ne!(DecodedValue::from(42.0), DecodedValue::from(21.0));
        assert_ne!(DecodedValue::from(42.0), DecodedValue::from("42.0"));
    }

    #[test]
    fn nan_equality() {
        assert_eq!(DecodedValue::from(f64::NAN), DecodedValue::from(f64::NAN));
        assert_eq!(
            DecodedValue::from(f64::NAN),
            DecodedValue::Float("nan".to_owned())
        );
        assert_ne!(DecodedValue::from(f64::NAN), DecodedValue::from("nan"));
    }

    #[test]
    fn inf_equality() {
        assert_eq!(
            DecodedValue::from(f64::INFINITY),
            DecodedValue::from(f64::INFINITY)
        );
        assert_ne!(
            DecodedValue::from(f64::INFINITY),
            DecodedValue::from(f64::NEG_INFINITY)
        );
        assert_eq!(
            DecodedValue::from(f64::INFINITY),
            DecodedValue::Float("inf".to_owned())
        );
        assert_eq!(
            DecodedValue::from(f64::INFINITY),
            DecodedValue::Float("+inf".to_owned())
        );
        assert_ne!(DecodedValue::from(f64::INFINITY), DecodedValue::from("inf"));
    }

    #[test]
    fn float_exp_equality() {
        assert_eq!(DecodedValue::from(3.0e14), DecodedValue::from(3.0e14));
        assert_eq!(
            DecodedValue::from(3.0e14),
            DecodedValue::Float("3.0e14".to_owned())
        );
    }

    #[test]
    fn float_binary_equality() {
        #![allow(clippy::excessive_precision)]

        // These cases are equivalent, just wanting to call out how Rust, at times, encodes the
        // number in a string.
        assert_eq!(
            DecodedValue::from(3141.5927),
            DecodedValue::Float("3141.5927".to_owned())
        );
        assert_eq!(
            DecodedValue::from(3141.59270000000015),
            DecodedValue::Float("3141.5927".to_owned())
        );
    }

    #[test]
    fn neg_inf_equality() {
        assert_eq!(
            DecodedValue::from(f64::NEG_INFINITY),
            DecodedValue::from(f64::NEG_INFINITY)
        );
        assert_ne!(
            DecodedValue::from(f64::NEG_INFINITY),
            DecodedValue::from(f64::INFINITY)
        );
        assert_eq!(
            DecodedValue::from(f64::NEG_INFINITY),
            DecodedValue::Float("-inf".to_owned())
        );
        assert_ne!(
            DecodedValue::from(f64::NEG_INFINITY),
            DecodedValue::from("-inf")
        );
    }

    #[test]
    fn bool_equality() {
        assert_eq!(DecodedValue::from(true), DecodedValue::from(true));
        assert_ne!(DecodedValue::from(true), DecodedValue::from(false));
        assert_ne!(DecodedValue::from(true), DecodedValue::from("true"));
    }

    #[test]
    fn datetime_equality() {
        assert_eq!(
            DecodedValue::Datetime("1987-07-05 17:45:00Z".to_owned()),
            DecodedValue::Datetime("1987-07-05 17:45:00Z".to_owned())
        );
        assert_eq!(
            DecodedValue::Datetime("1987-07-05T17:45:56.123456Z".to_owned()),
            DecodedValue::Datetime("1987-07-05T17:45:56.123456Z".to_owned()),
        );
        assert_ne!(
            DecodedValue::Datetime("1987-07-05 17:45:00Z".to_owned()),
            DecodedValue::Datetime("2000-07-05 17:45:00Z".to_owned())
        );
        assert_eq!(
            DecodedValue::Datetime("1987-07-05t17:45:00z".to_owned()),
            DecodedValue::Datetime("1987-07-05 17:45:00Z".to_owned())
        );
        assert_ne!(
            DecodedValue::Datetime("1987-07-05 17:45:00Z".to_owned()),
            DecodedValue::from("1987-07-05 17:45:00Z")
        );
    }

    #[test]
    fn datetime_local_equality() {
        assert_eq!(
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned()),
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned())
        );
        assert_eq!(
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00.444".to_owned()),
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00.444".to_owned())
        );
        assert_ne!(
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned()),
            DecodedValue::DatetimeLocal("2000-07-05 17:45:00".to_owned())
        );
        assert_eq!(
            DecodedValue::DatetimeLocal("1987-07-05t17:45:00".to_owned()),
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned())
        );
        assert_ne!(
            DecodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned()),
            DecodedValue::from("1987-07-05 17:45:00")
        );
    }

    #[test]
    fn date_local_equality() {
        assert_eq!(
            DecodedValue::DateLocal("1987-07-05".to_owned()),
            DecodedValue::DateLocal("1987-07-05".to_owned())
        );
        assert_ne!(
            DecodedValue::DateLocal("1987-07-05".to_owned()),
            DecodedValue::DateLocal("2000-07-05".to_owned())
        );
        assert_ne!(
            DecodedValue::DateLocal("1987-07-05".to_owned()),
            DecodedValue::from("1987-07-05")
        );
    }

    #[test]
    fn time_local_equality() {
        assert_eq!(
            DecodedValue::TimeLocal("17:45:00".to_owned()),
            DecodedValue::TimeLocal("17:45:00".to_owned())
        );
        assert_eq!(
            DecodedValue::TimeLocal("17:45:00.444".to_owned()),
            DecodedValue::TimeLocal("17:45:00.444".to_owned())
        );
        assert_ne!(
            DecodedValue::TimeLocal("17:45:00".to_owned()),
            DecodedValue::TimeLocal("19:45:00".to_owned())
        );
        assert_ne!(
            DecodedValue::TimeLocal("17:45:00".to_owned()),
            DecodedValue::from("17:45:00")
        );
    }
}
