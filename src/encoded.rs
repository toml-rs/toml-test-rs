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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        EncodedValue::Float(s)
    }
}

impl From<bool> for EncodedValue {
    fn from(other: bool) -> Self {
        EncodedValue::Bool(other.to_string())
    }
}

impl PartialEq for EncodedValue {
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::if_same_then_else)]
        match (self, other) {
            (EncodedValue::String(s), EncodedValue::String(o)) => s == o,
            (EncodedValue::Integer(s), EncodedValue::Integer(o)) => s == o,
            (EncodedValue::Float(s), EncodedValue::Float(o)) => {
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
            (EncodedValue::Bool(s), EncodedValue::Bool(o)) => s == o,
            (EncodedValue::Datetime(s), EncodedValue::Datetime(o)) => {
                parse_date_time(s) == parse_date_time(o)
            }
            (EncodedValue::DatetimeLocal(s), EncodedValue::DatetimeLocal(o)) => {
                parse_date_time_local(s) == parse_date_time_local(o)
            }
            (EncodedValue::DateLocal(s), EncodedValue::DateLocal(o)) => {
                parse_date_local(s) == parse_date_local(o)
            }
            (EncodedValue::TimeLocal(s), EncodedValue::TimeLocal(o)) => {
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

impl Eq for EncodedValue {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string_equality() {
        assert_eq!(EncodedValue::from("foo"), EncodedValue::from("foo"));
        assert_ne!(EncodedValue::from("foo"), EncodedValue::from("bar"));
        assert_ne!(EncodedValue::from("42"), EncodedValue::from(42));
        assert_ne!(EncodedValue::from("true"), EncodedValue::from(true));
    }

    #[test]
    fn integer_equality() {
        assert_eq!(EncodedValue::from(42), EncodedValue::from(42));
        assert_ne!(EncodedValue::from(42), EncodedValue::from(21));
        assert_ne!(EncodedValue::from(42), EncodedValue::from("42"));
    }

    #[test]
    fn float_equality() {
        assert_eq!(EncodedValue::from(42.0), EncodedValue::from(42.0));
        assert_ne!(EncodedValue::from(42.0), EncodedValue::from(21.0));
        assert_ne!(EncodedValue::from(42.0), EncodedValue::from("42.0"));
    }

    #[test]
    fn nan_equality() {
        assert_eq!(EncodedValue::from(f64::NAN), EncodedValue::from(f64::NAN));
        assert_eq!(
            EncodedValue::from(f64::NAN),
            EncodedValue::Float("nan".to_owned())
        );
        assert_ne!(EncodedValue::from(f64::NAN), EncodedValue::from("nan"));
    }

    #[test]
    fn inf_equality() {
        assert_eq!(
            EncodedValue::from(f64::INFINITY),
            EncodedValue::from(f64::INFINITY)
        );
        assert_ne!(
            EncodedValue::from(f64::INFINITY),
            EncodedValue::from(f64::NEG_INFINITY)
        );
        assert_eq!(
            EncodedValue::from(f64::INFINITY),
            EncodedValue::Float("inf".to_owned())
        );
        assert_eq!(
            EncodedValue::from(f64::INFINITY),
            EncodedValue::Float("+inf".to_owned())
        );
        assert_ne!(EncodedValue::from(f64::INFINITY), EncodedValue::from("inf"));
    }

    #[test]
    fn float_exp_equality() {
        assert_eq!(EncodedValue::from(3.0e14), EncodedValue::from(3.0e14));
        assert_eq!(
            EncodedValue::from(3.0e14),
            EncodedValue::Float("3.0e14".to_owned())
        );
    }

    #[test]
    fn float_binary_equality() {
        #![allow(clippy::excessive_precision)]

        // These cases are equivalent, just wanting to call out how Rust, at times, encodes the
        // number in a string.
        assert_eq!(
            EncodedValue::from(3141.5927),
            EncodedValue::Float("3141.5927".to_owned())
        );
        assert_eq!(
            EncodedValue::from(3141.59270000000015),
            EncodedValue::Float("3141.5927".to_owned())
        );
    }

    #[test]
    fn neg_inf_equality() {
        assert_eq!(
            EncodedValue::from(f64::NEG_INFINITY),
            EncodedValue::from(f64::NEG_INFINITY)
        );
        assert_ne!(
            EncodedValue::from(f64::NEG_INFINITY),
            EncodedValue::from(f64::INFINITY)
        );
        assert_eq!(
            EncodedValue::from(f64::NEG_INFINITY),
            EncodedValue::Float("-inf".to_owned())
        );
        assert_ne!(
            EncodedValue::from(f64::NEG_INFINITY),
            EncodedValue::from("-inf")
        );
    }

    #[test]
    fn bool_equality() {
        assert_eq!(EncodedValue::from(true), EncodedValue::from(true));
        assert_ne!(EncodedValue::from(true), EncodedValue::from(false));
        assert_ne!(EncodedValue::from(true), EncodedValue::from("true"));
    }

    #[test]
    fn datetime_equality() {
        assert_eq!(
            EncodedValue::Datetime("1987-07-05 17:45:00Z".to_owned()),
            EncodedValue::Datetime("1987-07-05 17:45:00Z".to_owned())
        );
        assert_eq!(
            EncodedValue::Datetime("1987-07-05T17:45:56.123456Z".to_owned()),
            EncodedValue::Datetime("1987-07-05T17:45:56.123456Z".to_owned()),
        );
        assert_ne!(
            EncodedValue::Datetime("1987-07-05 17:45:00Z".to_owned()),
            EncodedValue::Datetime("2000-07-05 17:45:00Z".to_owned())
        );
        assert_eq!(
            EncodedValue::Datetime("1987-07-05t17:45:00z".to_owned()),
            EncodedValue::Datetime("1987-07-05 17:45:00Z".to_owned())
        );
        assert_ne!(
            EncodedValue::Datetime("1987-07-05 17:45:00Z".to_owned()),
            EncodedValue::from("1987-07-05 17:45:00Z")
        );
    }

    #[test]
    fn datetime_local_equality() {
        assert_eq!(
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned()),
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned())
        );
        assert_eq!(
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00.444".to_owned()),
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00.444".to_owned())
        );
        assert_ne!(
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned()),
            EncodedValue::DatetimeLocal("2000-07-05 17:45:00".to_owned())
        );
        assert_eq!(
            EncodedValue::DatetimeLocal("1987-07-05t17:45:00".to_owned()),
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned())
        );
        assert_ne!(
            EncodedValue::DatetimeLocal("1987-07-05 17:45:00".to_owned()),
            EncodedValue::from("1987-07-05 17:45:00")
        );
    }

    #[test]
    fn date_local_equality() {
        assert_eq!(
            EncodedValue::DateLocal("1987-07-05".to_owned()),
            EncodedValue::DateLocal("1987-07-05".to_owned())
        );
        assert_ne!(
            EncodedValue::DateLocal("1987-07-05".to_owned()),
            EncodedValue::DateLocal("2000-07-05".to_owned())
        );
        assert_ne!(
            EncodedValue::DateLocal("1987-07-05".to_owned()),
            EncodedValue::from("1987-07-05")
        );
    }

    #[test]
    fn time_local_equality() {
        assert_eq!(
            EncodedValue::TimeLocal("17:45:00".to_owned()),
            EncodedValue::TimeLocal("17:45:00".to_owned())
        );
        assert_eq!(
            EncodedValue::TimeLocal("17:45:00.444".to_owned()),
            EncodedValue::TimeLocal("17:45:00.444".to_owned())
        );
        assert_ne!(
            EncodedValue::TimeLocal("17:45:00".to_owned()),
            EncodedValue::TimeLocal("19:45:00".to_owned())
        );
        assert_ne!(
            EncodedValue::TimeLocal("17:45:00".to_owned()),
            EncodedValue::from("17:45:00")
        );
    }
}
