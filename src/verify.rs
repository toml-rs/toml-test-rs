use std::io::Write;

pub trait Encoder {
    fn encode(&self, data: crate::decoded::Decoded) -> Result<String, crate::Error>;

    fn name(&self) -> &str;
}

pub trait Decoder {
    fn decode(&self, data: &[u8]) -> Result<crate::decoded::Decoded, crate::Error>;

    fn verify_valid_case(&self, fixture: &[u8], expected: &[u8]) -> Result<(), crate::Error> {
        let actual = self.decode(fixture)?;
        let expected = crate::decoded::Decoded::from_slice(expected)?;
        if actual == expected {
            Ok(())
        } else {
            Err(crate::Error::new(format!(
                "Unexpected decoding.\nExpected\n{}\nActual\n{}",
                expected.to_string_pretty().unwrap(),
                actual.to_string_pretty().unwrap()
            )))
        }
    }

    fn verify_invalid_case(&self, fixture: &[u8]) -> Result<crate::Error, crate::Error> {
        match self.decode(fixture) {
            Ok(value) => Err(crate::Error::new(format!(
                "Should have failed but got:\n{}",
                value.to_string_pretty().unwrap()
            ))),
            Err(err) => Ok(err),
        }
    }

    fn name(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    bin: std::path::PathBuf,
}

impl Command {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            bin: path.as_ref().to_owned(),
        }
    }
}

impl Decoder for Command {
    fn decode(&self, data: &[u8]) -> Result<crate::decoded::Decoded, crate::Error> {
        let mut cmd = std::process::Command::new(&self.bin);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = cmd.spawn().map_err(crate::Error::new)?;
        child
            .stdin
            .as_ref()
            .unwrap()
            .write_all(data)
            .map_err(crate::Error::new)?;

        let output = child.wait_with_output().map_err(crate::Error::new)?;
        if output.status.success() {
            let output =
                crate::decoded::Decoded::from_slice(&output.stdout).map_err(crate::Error::new)?;
            Ok(output)
        } else {
            let message = String::from_utf8_lossy(&output.stderr);
            Err(crate::Error::new(format!(
                "{} failed with {:?}: {}",
                self.bin.display(),
                output.status.code(),
                message
            )))
        }
    }

    fn name(&self) -> &str {
        self.bin.to_str().expect("we'll always get valid UTF-8")
    }
}
