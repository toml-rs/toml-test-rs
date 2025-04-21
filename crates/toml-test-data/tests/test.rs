#[test]
fn valid_doesnt_panic() {
    toml_test_data::valid().last().unwrap();
}

#[test]
fn invalid_doesnt_panic() {
    toml_test_data::invalid().last().unwrap();
}
