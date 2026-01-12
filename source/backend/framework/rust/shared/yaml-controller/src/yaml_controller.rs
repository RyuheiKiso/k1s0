use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
struct KeyNotFoundError(String);
impl fmt::Display for KeyNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key not found: {}", self.0)
    }
}
impl Error for KeyNotFoundError {}

#[derive(Debug)]
struct KeyAlreadyExistsError(String);
impl fmt::Display for KeyAlreadyExistsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key already exists: {}", self.0)
    }
}
impl Error for KeyAlreadyExistsError {}

fn split_path(key: &str) -> Vec<&str> {
    key.split('.').collect()
}

fn ensure_mapping_for_path<'a>(root: &'a mut Value, path: &[&str]) -> &'a mut Mapping {
    let mut cur = root;
    for seg in path {
        if !cur.is_mapping() {
            *cur = Value::Mapping(Mapping::new());
        }
        let map = cur.as_mapping_mut().unwrap();
        let key = Value::String(seg.to_string());
        if !map.contains_key(&key) {
            map.insert(key.clone(), Value::Mapping(Mapping::new()));
        }
        cur = map.get_mut(&key).unwrap();
    }
    cur.as_mapping_mut().unwrap()
}

fn get_value<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = root;
    for seg in path {
        match cur {
            Value::Mapping(map) => {
                let key = Value::String(seg.to_string());
                cur = map.get(&key)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn get_value_mut<'a>(root: &'a mut Value, path: &[&str]) -> Option<&'a mut Value> {
    let mut cur = root;
    for seg in path {
        match cur {
            Value::Mapping(map) => {
                let key = Value::String(seg.to_string());
                cur = map.get_mut(&key)?;
            }
            _ => return None,
        }
    }
    Some(cur)
}

fn atomic_write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(content.as_bytes())?;
    tmp.flush()?;
    // Persist (atomic rename)
    tmp.persist(path)?;
    Ok(())
}

/// Read YAML file and optionally return a nested key (dot-separated)
pub fn read_yaml(file_name: &str, key: Option<&str>) -> Result<Value, Box<dyn Error>> {
    let s = fs::read_to_string(file_name)?;
    let root: Value = serde_yaml::from_str(&s)?;
    if let Some(k) = key {
        let path = split_path(k);
        match get_value(&root, &path) {
            Some(v) => Ok(v.clone()),
            None => Err(Box::new(KeyNotFoundError(k.to_string()))),
        }
    } else {
        Ok(root)
    }
}

/// Add a new key (dot-separated) with the given value. Fails if key already exists.
pub fn add_key_value<T: Serialize>(file_name: &str, key: &str, value: T) -> Result<bool, Box<dyn Error>> {
    let path = Path::new(file_name);
    let mut root = if path.exists() {
        let s = fs::read_to_string(path)?;
        serde_yaml::from_str(&s)?
    } else {
        Value::Mapping(Mapping::new())
    };

    let parts = split_path(key);
    if parts.is_empty() {
        return Err(Box::new(KeyAlreadyExistsError(key.to_string())));
    }
    let (parent_parts, last) = parts.split_at(parts.len() - 1);

    let map = ensure_mapping_for_path(&mut root, parent_parts);
    let last_key = Value::String(last[0].to_string());
    if map.contains_key(&last_key) {
        return Err(Box::new(KeyAlreadyExistsError(key.to_string())));
    }
    let v = serde_yaml::to_value(value)?;
    map.insert(last_key, v);

    let s = serde_yaml::to_string(&root)?;
    atomic_write(path, &s)?;
    Ok(true)
}

/// Update an existing key (dot-separated) with a new value. Fails if key not found.
pub fn update_key_value<T: Serialize>(file_name: &str, key: &str, new_value: T) -> Result<bool, Box<dyn Error>> {
    let path = Path::new(file_name);
    if !path.exists() {
        return Err(Box::new(KeyNotFoundError(key.to_string())));
    }
    let mut root: Value = serde_yaml::from_str(&fs::read_to_string(path)?)?;
    let parts = split_path(key);
    if parts.is_empty() {
        return Err(Box::new(KeyNotFoundError(key.to_string())));
    }
    match get_value_mut(&mut root, &parts) {
        Some(target) => {
            *target = serde_yaml::to_value(new_value)?;
            let s = serde_yaml::to_string(&root)?;
            atomic_write(path, &s)?;
            Ok(true)
        }
        None => Err(Box::new(KeyNotFoundError(key.to_string()))),
    }
}

/// Delete a key (dot-separated). Fails if key not found.
pub fn delete_key(file_name: &str, key: &str) -> Result<bool, Box<dyn Error>> {
    let path = Path::new(file_name);
    if !path.exists() {
        return Err(Box::new(KeyNotFoundError(key.to_string())));
    }
    let mut root: Value = serde_yaml::from_str(&fs::read_to_string(path)?)?;
    let parts = split_path(key);
    if parts.is_empty() { return Err(Box::new(KeyNotFoundError(key.to_string()))); }
    let (parent_parts, last) = parts.split_at(parts.len() - 1);
    let mut cur = &mut root;
    for seg in parent_parts {
        if !cur.is_mapping() { return Err(Box::new(KeyNotFoundError(key.to_string()))); }
        let map = cur.as_mapping_mut().unwrap();
        let k = Value::String(seg.to_string());
        cur = map.get_mut(&k).ok_or_else(|| Box::new(KeyNotFoundError(key.to_string())))?;
    }
    if !cur.is_mapping() { return Err(Box::new(KeyNotFoundError(key.to_string()))); }
    let map = cur.as_mapping_mut().unwrap();
    let last_key = Value::String(last[0].to_string());
    if map.remove(&last_key).is_some() {
        let s = serde_yaml::to_string(&root)?;
        atomic_write(path, &s)?;
        Ok(true)
    } else {
        Err(Box::new(KeyNotFoundError(key.to_string())))
    }
}
