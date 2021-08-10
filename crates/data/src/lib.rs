const VALID_DIR: include_dir::Dir = include_dir::include_dir!("assets/toml-test/tests/valid");
const INVALID_DIR: include_dir::Dir = include_dir::include_dir!("assets/toml-test/tests/invalid");

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Valid<'a> {
    pub name: &'a str,
    pub fixture: &'a [u8],
    pub expected: &'a [u8],
}

pub fn valid() -> impl Iterator<Item = Valid<'static>> {
    valid_files(VALID_DIR.files()).chain(VALID_DIR.dirs().iter().flat_map(|d| {
        assert!(d.dirs().is_empty());
        valid_files(d.files())
    }))
}

fn valid_files(
    files: &'static [include_dir::File<'static>],
) -> impl Iterator<Item = Valid<'static>> {
    files
        .iter()
        .filter(|f| f.path().extension().unwrap_or_default() == "toml")
        .map(move |f| {
            let t = f;
            let j = files
                .iter()
                .find(|f| {
                    f.path().parent() == t.path().parent()
                        && f.path().file_stem() == t.path().file_stem()
                        && f.path().extension().unwrap() == "json"
                })
                .unwrap();
            let name = t.path().to_str().unwrap();
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
    pub name: &'a str,
    pub fixture: &'a [u8],
}

pub fn invalid() -> impl Iterator<Item = Invalid<'static>> {
    assert!(INVALID_DIR.files().is_empty());
    INVALID_DIR.dirs().iter().flat_map(|d| {
        assert!(d.dirs().is_empty());
        d.files().iter().map(|f| {
            let t = f;
            let name = f.path().to_str().unwrap();
            let fixture = t.contents();
            Invalid { name, fixture }
        })
    })
}
