#[test]
fn can_load() {
    for valid in toml_test_data::valid() {
        toml_test::DecodedValue::from_slice(valid.expected()).unwrap();
    }
}
