#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]
#![allow(clippy::self_named_module_files)] // `include_dir!`?

const TESTS_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/toml-test/tests");

pub fn version(ver: &str) -> impl Iterator<Item = &'static std::path::Path> {
    TESTS_DIR
        .get_file(format!("files-toml-{ver}"))
        .and_then(|f| std::str::from_utf8(f.contents()).ok())
        .into_iter()
        .flat_map(|f| f.lines())
        .map(std::path::Path::new)
}

pub fn versions() -> std::collections::HashMap<&'static str, Vec<&'static std::path::Path>> {
    TESTS_DIR
        .files()
        .filter_map(|f| {
            let name = f.path().file_name()?;
            let version = name.to_str()?.strip_prefix("files-toml-")?;
            let paths = std::str::from_utf8(f.contents())
                .ok()?
                .lines()
                .map(std::path::Path::new)
                .collect::<Vec<_>>();
            Some((version, paths))
        })
        .collect()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Valid<'a> {
    pub name: &'a std::path::Path,
    pub fixture: &'a [u8],
    pub expected: &'a [u8],
}

pub fn valid() -> impl Iterator<Item = Valid<'static>> {
    let valid_dir = TESTS_DIR.get_dir("valid").unwrap();
    valid_files(valid_dir).chain(valid_dir.dirs().flat_map(|d| {
        assert_eq!(d.dirs().count(), 0);
        valid_files(d)
    }))
}

fn valid_files<'d>(
    dir: &'d include_dir::Dir<'static>,
) -> impl Iterator<Item = Valid<'static>> + 'd {
    dir.files()
        .filter(|f| f.path().extension().unwrap_or_default() == "toml")
        .map(move |f| {
            let t = f;
            let j = dir
                .files()
                .find(|f| {
                    f.path().parent() == t.path().parent()
                        && f.path().file_stem() == t.path().file_stem()
                        && f.path().extension().unwrap() == "json"
                })
                .unwrap();
            let name = t.path();
            let fixture = t.contents();
            let expected = j.contents();
            Valid {
                name,
                fixture,
                expected,
            }
        })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Invalid<'a> {
    pub name: &'a std::path::Path,
    pub fixture: &'a [u8],
}

pub fn invalid() -> impl Iterator<Item = Invalid<'static>> {
    let invalid_dir = TESTS_DIR.get_dir("invalid").unwrap();
    assert_eq!(invalid_dir.files().count(), 0);
    invalid_dir.dirs().flat_map(|d| {
        assert_eq!(d.dirs().count(), 0);
        d.files().map(|f| {
            let t = f;
            let name = f.path();
            let fixture = t.contents();
            Invalid { name, fixture }
        })
    })
}
