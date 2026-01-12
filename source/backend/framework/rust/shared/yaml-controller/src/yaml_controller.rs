use serde_yaml::{Mapping, Value};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub enum YamlControllerError {
    EmptyFileName,
    EmptyKey,
    NonMappingRoot,
}

impl Display for YamlControllerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlControllerError::EmptyFileName => write!(f, "file_name must not be empty"),
            YamlControllerError::EmptyKey => write!(f, "key must not be empty"),
            YamlControllerError::NonMappingRoot => write!(f, "YAML root is not a mapping"),
        }
    }
}

impl Error for YamlControllerError {}

#[derive(Debug)]
pub struct FileNotFoundError(pub String);

impl Display for FileNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "YAML file not found: {}", self.0)
    }
}

impl Error for FileNotFoundError {}

#[derive(Debug)]
pub struct KeyNotFoundError(pub String);

impl Display for KeyNotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "key not found: {}", self.0)
    }
}

impl Error for KeyNotFoundError {}

#[derive(Debug)]
pub struct KeyAlreadyExistsError(pub String);

impl Display for KeyAlreadyExistsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "key already exists: {}", self.0)
    }
}

impl Error for KeyAlreadyExistsError {}

pub fn read_yaml(file_name: &str, key: Option<&str>) -> Result<Value, Box<dyn Error>> {
    if file_name.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyFileName));
    }

    let path = resolve_existing_yaml_path(file_name)
        .ok_or_else(|| Box::new(FileNotFoundError(file_name.to_string())) as Box<dyn Error>)?;

    let contents = fs::read_to_string(&path)?;
    let value: Value = serde_yaml::from_str(&contents)?;

    if let Some(key_path) = key {
        let segments = split_key(key_path);
        let found = find_value(&value, &segments)
            .ok_or_else(|| Box::new(KeyNotFoundError(key_path.to_string())) as Box<dyn Error>)?;
        return Ok(found.clone());
    }

    Ok(value)
}

pub fn add_key_value(
    file_name: &str,
    key: &str,
    value: impl serde::Serialize,
) -> Result<bool, Box<dyn Error>> {
    if file_name.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyFileName));
    }
    if key.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyKey));
    }

    let path = resolve_target_yaml_path(file_name);
    let mut root = match fs::read_to_string(&path) {
        Ok(contents) => serde_yaml::from_str::<Value>(&contents)?,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Value::Mapping(Mapping::new()),
        Err(err) => return Err(Box::new(err)),
    };

    let segments = split_key(key);
    let new_value = serde_yaml::to_value(value)?;

    insert_value(&mut root, &segments, new_value)?;

    write_yaml_atomic(&path, &root)?;
    Ok(true)
}

pub fn update_key_value(
    file_name: &str,
    key: &str,
    new_value: impl serde::Serialize,
) -> Result<bool, Box<dyn Error>> {
    if file_name.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyFileName));
    }
    if key.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyKey));
    }

    let path = resolve_existing_yaml_path(file_name)
        .unwrap_or_else(|| resolve_target_yaml_path(file_name));

    let contents = fs::read_to_string(&path)
        .map_err(|_| Box::new(FileNotFoundError(file_name.to_string())) as Box<dyn Error>)?;
    let mut root: Value = serde_yaml::from_str(&contents)?;

    let segments = split_key(key);
    let new_value = serde_yaml::to_value(new_value)?;

    replace_value(&mut root, &segments, new_value)?;

    write_yaml_atomic(&path, &root)?;
    Ok(true)
}

pub fn delete_key(file_name: &str, key: &str) -> Result<bool, Box<dyn Error>> {
    if file_name.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyFileName));
    }
    if key.trim().is_empty() {
        return Err(Box::new(YamlControllerError::EmptyKey));
    }

    let path = resolve_existing_yaml_path(file_name)
        .ok_or_else(|| Box::new(FileNotFoundError(file_name.to_string())) as Box<dyn Error>)?;

    let contents = fs::read_to_string(&path)?;
    let mut root: Value = serde_yaml::from_str(&contents)?;

    let segments = split_key(key);
    remove_value(&mut root, &segments)?;

    write_yaml_atomic(&path, &root)?;
    Ok(true)
}

fn resolve_existing_yaml_path(file_name: &str) -> Option<PathBuf> {
    let production = Path::new("config").join(file_name);
    if production.exists() {
        return Some(production);
    }

    let development = Path::new("../../config").join(file_name);
    if development.exists() {
        return Some(development);
    }

    None
}

fn resolve_target_yaml_path(file_name: &str) -> PathBuf {
    resolve_existing_yaml_path(file_name).unwrap_or_else(|| Path::new("config").join(file_name))
}

fn split_key(key: &str) -> Vec<&str> {
    key.split('.')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn find_value<'a>(value: &'a Value, segments: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in segments {
        match current {
            Value::Mapping(map) => {
                current = map.get(&Value::String(segment.to_string()))?;
            }
            Value::Sequence(seq) => {
                let idx: usize = segment.parse().ok()?;
                current = seq.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}

fn insert_value(
    value: &mut Value,
    segments: &[&str],
    new_value: Value,
) -> Result<(), Box<dyn Error>> {
    if segments.is_empty() {
        return Err(Box::new(YamlControllerError::EmptyKey));
    }

    let mut current = value;
    let last_index = segments.len() - 1;

    for (i, segment) in segments.iter().enumerate() {
        match current {
            Value::Mapping(map) => {
                if i == last_index {
                    if map.contains_key(&Value::String(segment.to_string())) {
                        return Err(Box::new(KeyAlreadyExistsError(segment.to_string())));
                    }
                    map.insert(Value::String(segment.to_string()), new_value);
                    return Ok(());
                }

                current = map
                    .entry(Value::String(segment.to_string()))
                    .or_insert_with(|| Value::Mapping(Mapping::new()));
            }
            Value::Sequence(seq) => {
                let idx: usize = segment
                    .parse()
                    .map_err(|_| KeyNotFoundError(segment.to_string()))?;
                current = seq
                    .get_mut(idx)
                    .ok_or_else(|| KeyNotFoundError(segment.to_string()))?;
            }
            _ => return Err(Box::new(YamlControllerError::NonMappingRoot)),
        }
    }

    Ok(())
}

fn replace_value(
    value: &mut Value,
    segments: &[&str],
    new_value: Value,
) -> Result<(), Box<dyn Error>> {
    if segments.is_empty() {
        return Err(Box::new(YamlControllerError::EmptyKey));
    }

    let mut current = value;
    let last_index = segments.len() - 1;

    for (i, segment) in segments.iter().enumerate() {
        match current {
            Value::Mapping(map) => {
                if i == last_index {
                    let entry = map
                        .get_mut(&Value::String(segment.to_string()))
                        .ok_or_else(|| KeyNotFoundError(segment.to_string()))?;
                    *entry = new_value;
                    return Ok(());
                }
                current = map
                    .get_mut(&Value::String(segment.to_string()))
                    .ok_or_else(|| KeyNotFoundError(segment.to_string()))?;
            }
            Value::Sequence(seq) => {
                let idx: usize = segment
                    .parse()
                    .map_err(|_| KeyNotFoundError(segment.to_string()))?;
                current = seq
                    .get_mut(idx)
                    .ok_or_else(|| KeyNotFoundError(segment.to_string()))?;
            }
            _ => return Err(Box::new(YamlControllerError::NonMappingRoot)),
        }
    }

    Ok(())
}

fn remove_value(value: &mut Value, segments: &[&str]) -> Result<(), Box<dyn Error>> {
    if segments.is_empty() {
        return Err(Box::new(YamlControllerError::EmptyKey));
    }

    let mut current = value;
    let last_index = segments.len() - 1;

    for (i, segment) in segments.iter().enumerate() {
        match current {
            Value::Mapping(map) => {
                if i == last_index {
                    let existed = map.remove(&Value::String(segment.to_string()));
                    if existed.is_none() {
                        return Err(Box::new(KeyNotFoundError(segment.to_string())));
                    }
                    return Ok(());
                }
                current = map
                    .get_mut(&Value::String(segment.to_string()))
                    .ok_or_else(|| KeyNotFoundError(segment.to_string()))?;
            }
            Value::Sequence(seq) => {
                let idx: usize = segment
                    .parse()
                    .map_err(|_| KeyNotFoundError(segment.to_string()))?;
                current = seq
                    .get_mut(idx)
                    .ok_or_else(|| KeyNotFoundError(segment.to_string()))?;
            }
            _ => return Err(Box::new(YamlControllerError::NonMappingRoot)),
        }
    }

    Ok(())
}

fn write_yaml_atomic(path: &Path, value: &Value) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml_string = serde_yaml::to_string(value)?;
    let parent_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let mut tmp_file = NamedTempFile::new_in(parent_dir)?;
    tmp_file.write_all(yaml_string.as_bytes())?;
    tmp_file.flush()?;
    tmp_file.as_file().sync_all()?;

    let (_file, tmp_path) = tmp_file.keep()?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}
