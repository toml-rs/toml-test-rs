//! Verify Rust TOML parsers

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod decoded;
mod error;
mod verify;

pub use decoded::DecodedScalar;
pub use decoded::DecodedValue;
pub use error::Error;
pub use verify::Command;
pub use verify::Decoder;
pub use verify::Encoder;
