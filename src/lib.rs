//! > DESCRIPTION

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod error;

pub mod decoded;
pub mod verify;
pub use error::Error;
