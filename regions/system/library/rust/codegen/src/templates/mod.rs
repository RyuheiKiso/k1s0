use tera::Tera;

const CARGO_TOML: &str = include_str!("cargo_toml.tera");
const MAIN_RS: &str = include_str!("main_rs.tera");
const LIB_RS: &str = include_str!("lib_rs.tera");
const ERROR_RS: &str = include_str!("error_rs.tera");
const BUILD_RS: &str = include_str!("build_rs.tera");
const CONFIG_YAML: &str = include_str!("config_yaml.tera");
const PROTO: &str = include_str!("proto.tera");
const MIGRATION_UP: &str = include_str!("migration_up.tera");
const MIGRATION_DOWN: &str = include_str!("migration_down.tera");
const README: &str = include_str!("readme.tera");
const HANDLER_RS: &str = include_str!("handler_rs.tera");
const CONFIG_RS: &str = include_str!("config_rs.tera");
const MOD_RS: &str = include_str!("mod_rs.tera");

pub fn create_tera_engine() -> Result<Tera, tera::Error> {
    let mut tera = Tera::default();
    tera.add_raw_templates(vec![
        ("cargo_toml", CARGO_TOML),
        ("main_rs", MAIN_RS),
        ("lib_rs", LIB_RS),
        ("error_rs", ERROR_RS),
        ("build_rs", BUILD_RS),
        ("config_yaml", CONFIG_YAML),
        ("proto", PROTO),
        ("migration_up", MIGRATION_UP),
        ("migration_down", MIGRATION_DOWN),
        ("readme", README),
        ("handler_rs", HANDLER_RS),
        ("config_rs", CONFIG_RS),
        ("mod_rs", MOD_RS),
    ])?;
    Ok(tera)
}
