fn main() {
    let catalog = alani_sdk::sdk_catalog();
    println!(
        "{} {} modules={} features=0x{:x}",
        catalog.repository,
        catalog.version,
        alani_sdk::module_names().len(),
        catalog.features
    );
}
