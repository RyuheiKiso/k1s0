fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = "test_config.yaml";
    let _ = std::fs::remove_file(file);

    yaml_controller::add_key_value(file, "database.host", "localhost")?;
    yaml_controller::add_key_value(file, "database.port", 5432)?;

    let db = yaml_controller::read_yaml(file, Some("database"))?;
    println!("database: {:?}", db);

    yaml_controller::update_key_value(file, "database.port", 5433)?;
    let port = yaml_controller::read_yaml(file, Some("database.port"))?;
    println!("port: {:?}", port);

    yaml_controller::delete_key(file, "database.host")?;
    let all = yaml_controller::read_yaml(file, None)?;
    println!("all: {:?}", all);

    Ok(())
}
