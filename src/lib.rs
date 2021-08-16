use std::io::Read;
use std::io::Write;

mod error;

pub mod encoded;
pub mod verify;
pub use error::Error;

/// External parser helper for [verify]
pub fn encoder_in() -> Result<encoded::Encoded, Error> {
    let mut buf = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buf)
        .map_err(crate::Error::new)?;
    encoded::Encoded::from_slice(&buf)
}

/// External parser helper for [verify]
pub fn decoder_out(e: encoded::Encoded) -> Result<(), Error> {
    let s = e.to_string_pretty()?;
    std::io::stdout()
        .write_all(s.as_bytes())
        .map_err(crate::Error::new)
}
