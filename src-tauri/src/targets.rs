use crate::AppError;
use crate::model::{NozzleTarget, PrinterPresetKind, PrinterTarget};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(default)]
    machine_model_list: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct ManifestEntry {
    name: String,
    sub_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct TargetCatalog {
    printers: Vec<PrinterTarget>,
}

impl TargetCatalog {
    pub fn load(manifest_path: &Path, custom_machine_dir: Option<&Path>) -> Result<Self, AppError> {
        let bytes =
            std::fs::read(manifest_path).map_err(|error| AppError::io(manifest_path, error))?;
        let manifest: Manifest =
            serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
                path: manifest_path.to_path_buf(),
                source,
            })?;
        let root = manifest_path.parent().ok_or_else(|| {
            AppError::InvalidProfile("manifest has no parent directory".to_owned())
        })?;
        let compatibility = scan_compatible_printers(&root.join("filament"))?;
        let variants = scan_extruder_variants(&root.join("filament"))?;
        let mut printers = Vec::new();
        for entry in manifest.machine_model_list {
            let machine_path = root.join(&entry.sub_path);
            let machine = load_json(&machine_path)?;
            let mut nozzles = nozzle_values(&machine);
            if let Some(found) = compatibility.get(&entry.name) {
                nozzles.extend(found.iter().cloned());
            }
            nozzles.sort_by(|left, right| numeric_cmp(left, right));
            nozzles.dedup();
            if nozzles.is_empty() {
                continue;
            }
            let code = printer_code(&entry.name);
            printers.push(PrinterTarget {
                id: format!("official:{code}"),
                name: entry.name.clone(),
                code: code.clone(),
                kind: PrinterPresetKind::Official,
                verified: true,
                extruder_variants: variants
                    .get(&entry.name)
                    .cloned()
                    .unwrap_or_else(|| vec!["Direct Drive Standard".to_owned()]),
                nozzles: nozzles
                    .into_iter()
                    .map(|diameter| NozzleTarget {
                        id: format!("{code}:{diameter}"),
                        printer_preset_name: format!("{} {} nozzle", entry.name, diameter),
                        diameter,
                        selected: false,
                        supported: true,
                    })
                    .collect(),
            });
        }
        if let Some(dir) = custom_machine_dir.filter(|dir| dir.is_dir()) {
            printers.extend(load_custom_printers(dir)?);
        }
        printers.sort_by_key(|printer| {
            (
                if printer.kind == PrinterPresetKind::Official {
                    0
                } else {
                    1
                },
                printer.name.clone(),
            )
        });
        Ok(Self { printers })
    }

    pub fn printers(&self) -> &[PrinterTarget] {
        &self.printers
    }

    pub fn visible(&self, show_custom: bool) -> Vec<&PrinterTarget> {
        self.printers
            .iter()
            .filter(|printer| show_custom || printer.kind == PrinterPresetKind::Official)
            .collect()
    }
}

fn load_custom_printers(dir: &Path) -> Result<Vec<PrinterTarget>, AppError> {
    let nozzle_re =
        Regex::new(r"^(?P<name>.+?) (?P<nozzle>\d+(?:\.\d+)?) nozzle$").expect("constant regex");
    let mut grouped: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut paths: Vec<PathBuf> = WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect();
    paths.sort();
    for path in paths {
        let value = load_json(&path)?;
        let Some(raw_name) = value.get("name").and_then(Value::as_str) else {
            continue;
        };
        if let Some(captures) = nozzle_re.captures(raw_name) {
            grouped
                .entry(captures["name"].to_owned())
                .or_default()
                .insert(captures["nozzle"].to_owned());
        } else {
            let nozzles = nozzle_values(&value);
            grouped
                .entry(raw_name.to_owned())
                .or_default()
                .extend(nozzles);
        }
    }
    Ok(grouped
        .into_iter()
        .map(|(name, nozzles)| {
            let code = printer_code(&name);
            PrinterTarget {
                id: format!("custom:{name}"),
                name: name.clone(),
                code: code.clone(),
                kind: PrinterPresetKind::Custom,
                verified: false,
                extruder_variants: vec!["Unknown custom target".to_owned()],
                nozzles: nozzles
                    .into_iter()
                    .map(|diameter| NozzleTarget {
                        id: format!("custom:{name}:{diameter}"),
                        printer_preset_name: format!("{name} {diameter} nozzle"),
                        diameter,
                        selected: false,
                        supported: false,
                    })
                    .collect(),
            }
        })
        .collect())
}

fn scan_compatible_printers(dir: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, AppError> {
    let re =
        Regex::new(r"^(?P<name>.+?) (?P<nozzle>\d+(?:\.\d+)?) nozzle$").expect("constant regex");
    let mut result: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(result);
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let value = load_json(entry.path())?;
        for printer in value
            .get("compatible_printers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if let Some(captures) = re.captures(printer) {
                result
                    .entry(captures["name"].to_owned())
                    .or_default()
                    .insert(captures["nozzle"].to_owned());
            }
        }
    }
    Ok(result)
}

fn scan_extruder_variants(dir: &Path) -> Result<BTreeMap<String, Vec<String>>, AppError> {
    let re = Regex::new(r"^(?P<name>.+?) \d+(?:\.\d+)? nozzle$").expect("constant regex");
    let mut result: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if !dir.is_dir() {
        return Ok(BTreeMap::new());
    }
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
        {
            continue;
        }
        let value = load_json(entry.path())?;
        let variants: Vec<_> = value
            .get("filament_extruder_variant")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        if variants.is_empty() {
            continue;
        }
        for printer in value
            .get("compatible_printers")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if let Some(captures) = re.captures(printer) {
                result
                    .entry(captures["name"].to_owned())
                    .or_default()
                    .extend(variants.iter().cloned());
            }
        }
    }
    Ok(result
        .into_iter()
        .map(|(key, values)| (key, values.into_iter().collect()))
        .collect())
}

fn load_json(path: &Path) -> Result<Value, AppError> {
    let bytes = std::fs::read(path).map_err(|error| AppError::io(path, error))?;
    serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn nozzle_values(value: &Value) -> Vec<String> {
    value
        .get("nozzle_diameter")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn numeric_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    left.parse::<f32>()
        .unwrap_or_default()
        .partial_cmp(&right.parse::<f32>().unwrap_or_default())
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn printer_code(name: &str) -> String {
    let short = name.strip_prefix("Bambu Lab ").unwrap_or(name);
    match short {
        "X1 Carbon" => "X1C".to_owned(),
        "A1 mini" => "A1M".to_owned(),
        "H2D Pro" => "H2DP".to_owned(),
        _ if !short.contains(' ') => short.to_owned(),
        _ => short
            .split_whitespace()
            .filter_map(|part| part.chars().next())
            .collect::<String>()
            .to_ascii_uppercase(),
    }
}
