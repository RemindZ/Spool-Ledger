use crate::AppError;
use crate::model::{ProfileId, SourceApp, SourceKind};
use crate::profiles::ProfileDocument;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CatalogRoot {
    pub path: PathBuf,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    priority: u8,
}

impl CatalogRoot {
    pub fn system(path: impl Into<PathBuf>, source_app: SourceApp) -> Self {
        Self {
            path: path.into(),
            source_app,
            source_kind: SourceKind::FactorySystem,
            priority: 10,
        }
    }

    pub fn vendor_system(path: impl Into<PathBuf>, source_app: SourceApp) -> Self {
        Self {
            path: path.into(),
            source_app,
            source_kind: SourceKind::FactorySystem,
            priority: 20,
        }
    }

    pub fn filament_library(path: impl Into<PathBuf>, source_app: SourceApp) -> Self {
        Self {
            path: path.into(),
            source_app,
            source_kind: SourceKind::FactorySystem,
            priority: 30,
        }
    }

    pub fn user(path: impl Into<PathBuf>, source_app: SourceApp) -> Self {
        Self {
            path: path.into(),
            source_app,
            source_kind: SourceKind::UserCustom,
            priority: 100,
        }
    }

    pub fn default_local_user(path: impl Into<PathBuf>, source_app: SourceApp) -> Self {
        Self {
            path: path.into(),
            source_app,
            source_kind: SourceKind::UserCustom,
            priority: 90,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileRecord {
    pub id: ProfileId,
    pub name: String,
    pub path: PathBuf,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    pub instantiable: bool,
    document: ProfileDocument,
    root_path: PathBuf,
    priority: u8,
}

#[derive(Debug, Clone)]
pub struct ValueProvenance {
    pub path: PathBuf,
}

impl ValueProvenance {
    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    }
}

#[derive(Debug, Clone)]
pub struct EffectiveProfile {
    pub id: ProfileId,
    pub name: String,
    pub values: BTreeMap<String, Value>,
    provenance: BTreeMap<String, ValueProvenance>,
    pub source_app: SourceApp,
    pub source_kind: SourceKind,
    pub source_path: PathBuf,
}

impl EffectiveProfile {
    pub fn string_values(&self, key: &str) -> Vec<&str> {
        match self.values.get(key) {
            Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
            Some(Value::String(value)) => vec![value],
            _ => Vec::new(),
        }
    }

    pub fn provenance(&self, key: &str) -> Option<&ValueProvenance> {
        self.provenance.get(key)
    }

    pub fn value(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProfileCatalog {
    profiles: BTreeMap<String, ProfileRecord>,
    profiles_by_root: BTreeMap<PathBuf, BTreeMap<String, ProfileRecord>>,
}

impl ProfileCatalog {
    pub fn load_roots(roots: &[CatalogRoot]) -> Result<Self, AppError> {
        Self::load_roots_with_progress(roots, || {})
    }

    pub fn load_roots_with_progress(
        roots: &[CatalogRoot],
        mut on_file: impl FnMut(),
    ) -> Result<Self, AppError> {
        let mut catalog = Self::default();
        for root in roots {
            if !root.path.exists() {
                return Err(AppError::InvalidProfile(format!(
                    "profile root does not exist: {}",
                    root.path.display()
                )));
            }
            let mut paths: Vec<_> = WalkDir::new(&root.path)
                .follow_links(false)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
                .map(|entry| entry.into_path())
                .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
                .filter(|path| {
                    path.file_name().and_then(|value| value.to_str()) != Some("BBL.json")
                })
                .collect();
            paths.sort();
            for path in paths {
                let document = ProfileDocument::load(&path)?;
                on_file();
                let Some(name) = document.name().map(str::to_owned) else {
                    continue;
                };
                if !is_filament_document(&document.value) {
                    continue;
                }
                let record = ProfileRecord {
                    id: ProfileId::new(name.clone()),
                    name: name.clone(),
                    path,
                    source_app: root.source_app,
                    source_kind: root.source_kind,
                    instantiable: document.is_instantiable(),
                    document,
                    root_path: root.path.clone(),
                    priority: root.priority,
                };
                if let Some(existing) = catalog
                    .profiles_by_root
                    .entry(root.path.clone())
                    .or_default()
                    .insert(name.clone(), record.clone())
                {
                    return Err(AppError::InvalidProfile(format!(
                        "ambiguous profile name {name}: {} and {}",
                        existing.path.display(),
                        record.path.display()
                    )));
                }
                match catalog.profiles.get(&name) {
                    Some(existing) if existing.priority == record.priority => {
                        return Err(AppError::InvalidProfile(format!(
                            "ambiguous profile name {name}: {} and {}",
                            existing.path.display(),
                            record.path.display()
                        )));
                    }
                    Some(existing) if existing.priority > record.priority => {}
                    _ => {
                        catalog.profiles.insert(name, record);
                    }
                }
            }
        }
        Ok(catalog)
    }

    pub fn selectable_profiles(&self) -> Vec<&ProfileRecord> {
        self.profiles
            .values()
            .filter(|profile| profile.instantiable)
            .collect()
    }

    pub fn profile(&self, name: &str) -> Option<&ProfileRecord> {
        self.profiles.get(name)
    }

    pub fn resolve_name(&self, name: &str) -> Result<EffectiveProfile, AppError> {
        let source = self
            .profiles
            .get(name)
            .ok_or_else(|| AppError::InvalidProfile(format!("profile not found: {name}")))?;
        let mut stack = Vec::new();
        let (values, provenance) = self.resolve_inner(name, &source.root_path, &mut stack)?;
        Ok(EffectiveProfile {
            id: source.id.clone(),
            name: source.name.clone(),
            values,
            provenance,
            source_app: source.source_app,
            source_kind: source.source_kind,
            source_path: source.path.clone(),
        })
    }

    fn resolve_inner(
        &self,
        name: &str,
        preferred_root: &Path,
        stack: &mut Vec<String>,
    ) -> Result<ResolvedValues, AppError> {
        if let Some(position) = stack.iter().position(|item| item == name) {
            let mut cycle = stack[position..].to_vec();
            cycle.push(name.to_owned());
            return Err(AppError::InvalidProfile(format!(
                "inheritance cycle: {}",
                cycle.join(" -> ")
            )));
        }
        let profile = self
            .profiles_by_root
            .get(preferred_root)
            .and_then(|profiles| profiles.get(name))
            .or_else(|| self.profiles.get(name))
            .ok_or_else(|| {
                AppError::InvalidProfile(format!("missing parent or include: {name}"))
            })?;
        let profile_root = profile.root_path.clone();
        stack.push(name.to_owned());
        let mut resolved = ResolvedAccumulator::default();

        if let Some(parent) = profile
            .document
            .inherits()
            .filter(|value| !value.is_empty())
        {
            resolved.merge(self.resolve_inner(parent, &profile_root, stack)?);
        }
        if let Some(includes) = profile
            .document
            .value
            .get("include")
            .and_then(Value::as_array)
        {
            for include in includes.iter().filter_map(Value::as_str) {
                resolved.merge(self.resolve_inner(include, &profile_root, stack)?);
            }
        }
        if let Some(object) = profile.document.value.as_object() {
            resolved.apply(object, &profile.path);
        }
        stack.pop();
        Ok((resolved.values, resolved.provenance))
    }

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

type ResolvedValues = (BTreeMap<String, Value>, BTreeMap<String, ValueProvenance>);

#[derive(Default)]
struct ResolvedAccumulator {
    values: BTreeMap<String, Value>,
    provenance: BTreeMap<String, ValueProvenance>,
}

impl ResolvedAccumulator {
    fn merge(&mut self, other: ResolvedValues) {
        for (key, value) in other.0 {
            self.values.insert(key, value);
        }
        for (key, value) in other.1 {
            self.provenance.insert(key, value);
        }
    }

    fn apply(&mut self, object: &Map<String, Value>, path: &Path) {
        for (key, value) in object {
            self.values.insert(key.clone(), value.clone());
            self.provenance.insert(
                key.clone(),
                ValueProvenance {
                    path: path.to_path_buf(),
                },
            );
        }
    }
}

fn is_filament_document(value: &Value) -> bool {
    let object = match value.as_object() {
        Some(object) => object,
        None => return false,
    };
    object.get("type").and_then(Value::as_str) == Some("filament")
        || object.contains_key("filament_settings_id")
        || object.contains_key("filament_type")
        || object.contains_key("filament_id")
        || object.contains_key("inherits")
}
