use std::io::Read;
use std::io::Write;

mod error;

pub mod encoded;
pub use error::Error;

pub fn encoder_in() -> Result<encoded::Encoded, Error> {
    let mut buf = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buf)
        .map_err(|e| crate::Error::new(e.to_string()))?;
    encoded::Encoded::from_slice(&buf)
}

pub fn decoder_output(e: encoded::Encoded) -> Result<(), Error> {
    let s = e.to_string_pretty()?;
    std::io::stdout()
        .write_all(s.as_bytes())
        .map_err(|e| crate::Error::new(e.to_string()))
}
