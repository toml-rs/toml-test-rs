use std::io::Write;

pub use toml_test::decoded::Decoded;
pub use toml_test::decoded::DecodedValue;
pub use toml_test::verify::Decoder;
pub use toml_test::verify::Encoder;
pub use toml_test::Error;

pub struct DecoderHarness<D> {
    decoder: D,
    matches: Option<Matches>,
}

impl<D> DecoderHarness<D>
where
    D: toml_test::verify::Decoder + Copy + Send + Sync + 'static,
{
    pub fn new(decoder: D) -> Self {
        Self {
            decoder,
            matches: None,
        }
    }

    pub fn ignore<'p>(
        &mut self,
        patterns: impl IntoIterator<Item = &'p str>,
    ) -> Result<&mut Self, toml_test::Error> {
        self.matches = Some(Matches::new(patterns.into_iter())?);
        Ok(self)
    }

    pub fn test(self) -> ! {
        let args = libtest_mimic::Arguments::from_args();
        let nocapture = args.nocapture;

        let mut tests = Vec::new();
        let decoder = self.decoder;
        tests.extend(
            toml_test_data::valid()
                .map(|case| {
                    let ignore = self
                        .matches
                        .as_ref()
                        .map(|m| !m.matched(case.name))
                        .unwrap_or_default();
                    (case, ignore)
                })
                .map(move |(case, ignore)| {
                    libtest_mimic::Trial::test(case.name.display().to_string(), move || {
                        decoder
                            .verify_valid_case(case.fixture, case.expected)
                            .map_err(libtest_mimic::Failed::from)
                    })
                    .with_ignored_flag(ignore)
                }),
        );
        tests.extend(
            toml_test_data::invalid()
                .map(|case| {
                    let ignore = self
                        .matches
                        .as_ref()
                        .map(|m| !m.matched(case.name))
                        .unwrap_or_default();
                    (case, ignore)
                })
                .map(move |(case, ignore)| {
                    libtest_mimic::Trial::test(case.name.display().to_string(), move || {
                        match decoder.verify_invalid_case(case.fixture) {
                            Ok(err) => {
                                if nocapture {
                                    let _ = writeln!(std::io::stdout(), "{}", err);
                                }
                                Ok(())
                            }
                            Err(err) => Err(libtest_mimic::Failed::from(err)),
                        }
                    })
                    .with_ignored_flag(ignore)
                }),
        );

        libtest_mimic::run(&args, tests).exit()
    }
}

pub struct EncoderHarness<E, D> {
    encoder: E,
    fixture: D,
    matches: Option<Matches>,
}

impl<E, D> EncoderHarness<E, D>
where
    E: toml_test::verify::Encoder + Copy + Send + Sync + 'static,
    D: toml_test::verify::Decoder + Copy + Send + Sync + 'static,
{
    pub fn new(encoder: E, fixture: D) -> Self {
        Self {
            encoder,
            fixture,
            matches: None,
        }
    }

    pub fn ignore<'p>(
        &mut self,
        patterns: impl IntoIterator<Item = &'p str>,
    ) -> Result<&mut Self, toml_test::Error> {
        self.matches = Some(Matches::new(patterns.into_iter())?);
        Ok(self)
    }

    pub fn test(self) -> ! {
        let args = libtest_mimic::Arguments::from_args();
        let mut tests = Vec::new();
        let encoder = self.encoder;
        let fixture = self.fixture;
        tests.extend(
            toml_test_data::valid()
                .map(|case| {
                    let ignore = self
                        .matches
                        .as_ref()
                        .map(|m| !m.matched(case.name))
                        .unwrap_or_default();
                    (case, ignore)
                })
                .map(move |(case, ignore)| {
                    libtest_mimic::Trial::test(case.name.display().to_string(), move || {
                        encoder
                            .verify_valid_case(case.fixture, &fixture)
                            .map_err(libtest_mimic::Failed::from)
                    })
                    .with_ignored_flag(ignore)
                }),
        );
        libtest_mimic::run(&args, tests).exit()
    }
}

struct Matches {
    ignores: ignore::gitignore::Gitignore,
}

impl Matches {
    fn new<'p>(patterns: impl Iterator<Item = &'p str>) -> Result<Self, toml_test::Error> {
        let mut ignores = ignore::gitignore::GitignoreBuilder::new(".");
        for line in patterns {
            ignores
                .add_line(None, line)
                .map_err(toml_test::Error::new)?;
        }
        let ignores = ignores.build().map_err(toml_test::Error::new)?;
        Ok(Self { ignores })
    }

    fn matched(&self, path: &std::path::Path) -> bool {
        match self.ignores.matched_path_or_any_parents(path, false) {
            ignore::Match::None | ignore::Match::Whitelist(_) => true,
            ignore::Match::Ignore(_) => false,
        }
    }
}
