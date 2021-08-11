use std::io::Write;

use proc_exit::WithCodeResultExt;
use structopt::StructOpt;

fn main() {
    let result = run();
    proc_exit::exit(result);
}

fn run() -> proc_exit::ExitResult {
    let args = Args::from_args();
    init_logging(args.verbose);

    let matches = Matches::new(args.ignore.iter().map(|s| s.as_str()))
        .with_code(proc_exit::Code::USAGE_ERR)?;
    for bin in args.bin.iter() {
        if args.encoder {
            verify_encoder(bin.as_path(), &matches)?;
        } else {
            verify_decoder(bin.as_path(), &matches)?;
        }
    }

    Ok(())
}

fn verify_encoder(_bin: &std::path::Path, _matches: &Matches) -> proc_exit::ExitResult {
    unimplemented!("Not yet implemented");
}

fn verify_decoder(bin: &std::path::Path, matches: &Matches) -> proc_exit::ExitResult {
    let mut passed = 0;
    let mut failed = 0;

    for case in toml_test_data::valid() {
        if !matches.matched(case.name) {
            log::debug!("Skipped {}", case.name.display());
            continue;
        }
        match run_decoder(bin, case.fixture) {
            Ok(actual) => {
                let expected = toml_test::encoded::Encoded::from_slice(&case.expected)
                    .with_code(proc_exit::Code::USAGE_ERR)?;
                if actual == expected {
                    passed += 1;
                } else {
                    log::debug!("{}: failed", case.name.display());
                    log::trace!(
                        "{}: expected\n{}",
                        case.name.display(),
                        expected.to_string_pretty().unwrap()
                    );
                    log::trace!(
                        "{}: actual\n{}",
                        case.name.display(),
                        actual.to_string_pretty().unwrap()
                    );
                    failed += 1;
                }
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
        match run_decoder(bin, case.fixture) {
            Ok(actual) => {
                log::debug!("{}: should have failed", case.name.display());
                log::trace!(
                    "{}: actual\n{}",
                    case.name.display(),
                    actual.to_string_pretty().unwrap()
                );
                failed += 1;
            }
            Err(err) => {
                log::debug!("{}: failed successfully", case.name.display());
                log::trace!("{}: {}", case.name.display(), err);
                passed += 1;
            }
        }
    }

    let _ = writeln!(
        std::io::stdout(),
        "toml-test [{}]: using embedded tests: {} passed, {} failed",
        bin.file_name().unwrap().to_string_lossy(),
        passed,
        failed
    );
    if 0 < failed {
        proc_exit::Code::FAILURE.ok()?;
    }

    Ok(())
}

fn run_decoder(
    bin: &std::path::Path,
    toml: &[u8],
) -> Result<toml_test::encoded::Encoded, eyre::Error> {
    let mut cmd = std::process::Command::new(bin);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd.spawn()?;
    child.stdin.as_ref().unwrap().write_all(toml)?;

    let output = child.wait_with_output()?;
    if output.status.success() {
        let output = toml_test::encoded::Encoded::from_slice(&output.stdout)?;
        Ok(output)
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        eyre::bail!(
            "{} failed with {:?}: {}",
            bin.display(),
            output.status.code(),
            message
        )
    }
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

#[derive(StructOpt, Debug, Clone)]
struct Args {
    /// Encoder/decoder binary
    bin: Vec<std::path::PathBuf>,

    /// `bin` is an encoder, instead of a decoder
    #[structopt(long)]
    encoder: bool,

    /// Cases to ignore (gitignore glob syntax)
    #[structopt(long)]
    ignore: Vec<String>,

    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}
