#[test]
fn repository_identity_is_stable() {
    assert_eq!(alani_sdk::repository_name(), "alani-sdk");
    assert!(!alani_sdk::module_names().is_empty());
}
