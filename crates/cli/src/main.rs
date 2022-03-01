use std::io::Write;

use clap::Parser;
use proc_exit::WithCodeResultExt;

fn main() {
    let result = run();
    proc_exit::exit(result);
}

fn run() -> proc_exit::ExitResult {
    let args = Args::parse();
    init_logging(args.verbose);

    let matches = Matches::new(args.ignore.iter().map(|s| s.as_str()))
        .with_code(proc_exit::Code::USAGE_ERR)?;
    for bin in args.bin.iter() {
        let bin = toml_test::verify::Command::new(bin);
        if args.encoder {
            unimplemented!("Not yet implemented, waiting on a verified decoder");
        } else {
            verify_decoder(&bin, &matches)?;
        }
    }

    Ok(())
}

fn verify_decoder(
    bin: &dyn toml_test::verify::Decoder,
    matches: &Matches,
) -> proc_exit::ExitResult {
    let mut passed = 0;
    let mut failed = 0;

    for case in toml_test_data::valid() {
        if !matches.matched(case.name) {
            log::debug!("Skipped {}", case.name.display());
            continue;
        }
        match bin.verify_valid_case(case.fixture, case.expected) {
            Ok(()) => {
                passed += 1;
            }
            Err(err) => {
                log::debug!("{}: failed", case.name.display());
                log::trace!("{}: {}", case.name.display(), err);
                failed += 1;
            }
        }
    }

    for case in toml_test_data::invalid() {
        if !matches.matched(case.name) {
            log::debug!("Skipped {}", case.name.display());
            continue;
        }
        match bin.verify_invalid_case(case.fixture) {
            Ok(err) => {
                log::debug!("{}: failed successfully", case.name.display());
                log::trace!("{}: {}", case.name.display(), err);
                passed += 1;
            }
            Err(err) => {
                log::debug!("{}: should have failed", case.name.display());
                log::trace!("{}: {}", case.name.display(), err);
                failed += 1;
            }
        }
    }

    let _ = writeln!(
        std::io::stdout(),
        "toml-test [{}]: using embedded tests: {} passed, {} failed",
        bin.name(),
        passed,
        failed
    );
    if 0 < failed {
        proc_exit::Code::FAILURE.ok()?;
    }

    Ok(())
}

fn init_logging(mut level: clap_verbosity_flag::Verbosity) {
    level.set_default(Some(log::Level::Info));

    if let Some(level) = level.log_level() {
        let mut builder = env_logger::Builder::new();

        builder.filter(None, level.to_level_filter());

        if level == log::LevelFilter::Trace || level == log::LevelFilter::Debug {
            builder.format_timestamp_secs();
        } else {
            builder.format(|f, record| {
                if record.level() == log::LevelFilter::Info {
                    writeln!(f, "{}", record.args())
                } else {
                    writeln!(f, "[{}] {}", record.level(), record.args())
                }
            });
        }

        builder.init();
    }
}

struct Matches {
    ignores: ignore::gitignore::Gitignore,
}

impl Matches {
    fn new<'p>(patterns: impl Iterator<Item = &'p str>) -> Result<Self, eyre::Error> {
        let mut ignores = ignore::gitignore::GitignoreBuilder::new(".");
        for line in patterns {
            ignores.add_line(None, line)?;
        }
        let ignores = ignores.build()?;
        Ok(Self { ignores })
    }

    fn matched(&self, path: &std::path::Path) -> bool {
        match self.ignores.matched_path_or_any_parents(path, false) {
            ignore::Match::None | ignore::Match::Whitelist(_) => true,
            ignore::Match::Ignore(_) => false,
        }
    }
}

#[derive(Parser, Debug, Clone)]
struct Args {
    /// Encoder/decoder binary
    bin: Vec<std::path::PathBuf>,

    /// `bin` is an encoder, instead of a decoder
    #[clap(long)]
    encoder: bool,

    /// Cases to ignore (gitignore glob syntax)
    #[clap(long)]
    ignore: Vec<String>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
