use crate::AppError;
use crate::field_policy::{FieldClass, FieldPolicyTable};
use crate::model::ProfileId;
use crate::naming::validate_name_component;
use crate::planner::{MigrationPlan, PlanAction, PlanOperation};
use crate::profiles::{InfoSidecar, SyncAction};
use crate::resolver::{EffectiveProfile, ProfileCatalog};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BambuAdapter {
    pub id: &'static str,
    pub profile_version: &'static str,
    field_policy: FieldPolicyTable,
}

impl BambuAdapter {
    pub fn v2_0_0_56() -> Self {
        Self {
            id: "bambu-2.0.0.56-v1",
            profile_version: "2.0.0.56",
            field_policy: FieldPolicyTable::bundled().expect("bundled field policy must be valid"),
        }
    }
}

pub struct WriterContext<'a> {
    pub sources: &'a ProfileCatalog,
    pub targets: &'a ProfileCatalog,
    pub target_profiles: BTreeMap<String, String>,
    pub adapter: BambuAdapter,
    pub updated_time: i64,
    pub existing_sidecars: BTreeMap<String, InfoSidecar>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedArtifactKind {
    NormalPreset,
    CustomProfile,
    CustomSidecar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub kind: GeneratedArtifactKind,
    pub operation_ids: Vec<String>,
    pub relative_path: PathBuf,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagedDeletion {
    pub operation_ids: Vec<String>,
    pub relative_path: PathBuf,
    pub expected_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagedRun {
    pub root: PathBuf,
    pub artifacts: Vec<GeneratedArtifact>,
    pub deletions: Vec<StagedDeletion>,
}

pub struct Writer;

impl Writer {
    pub fn stage(
        plan: &MigrationPlan,
        context: &WriterContext<'_>,
        staging_root: &Path,
    ) -> Result<StagedRun, AppError> {
        validate_staging_root(staging_root)?;
        let writable: Vec<_> = plan
            .operations
            .iter()
            .filter(|operation| is_writable(operation.action))
            .collect();
        let resolved = resolve_and_validate_sources(&writable, context)?;
        let targets = resolve_targets(&writable, context)?;
        let mut generated: BTreeMap<PathBuf, PendingArtifact> = BTreeMap::new();

        let mut normal_groups: BTreeMap<(ProfileId, String), Vec<&PlanOperation>> = BTreeMap::new();
        for operation in &writable {
            normal_groups
                .entry((operation.source_id.clone(), operation.preset_name.clone()))
                .or_default()
                .push(operation);
        }
        for ((source_id, preset_name), operations) in normal_groups {
            validate_output_name(&preset_name)?;
            let source = resolved.get(&source_id).ok_or_else(|| {
                AppError::InvalidProfile(format!("resolved source disappeared: {source_id}"))
            })?;
            let target = common_target(&operations, &targets)?;
            let value =
                build_normal_profile(source, target, &preset_name, &operations, &context.adapter)?;
            insert_pending(
                &mut generated,
                PathBuf::from("filament").join(format!("{preset_name}.json")),
                GeneratedArtifactKind::NormalPreset,
                operations.iter().map(|item| item.id.clone()).collect(),
                json_bytes(&value)?,
            )?;
        }

        for operation in writable {
            let source = resolved.get(&operation.source_id).ok_or_else(|| {
                AppError::InvalidProfile(format!(
                    "resolved source disappeared: {}",
                    operation.source_id
                ))
            })?;
            let target = targets.get(&operation.id).ok_or_else(|| {
                AppError::InvalidProfile(format!(
                    "resolved target disappeared: {}",
                    operation.printer_id
                ))
            })?;
            let profile_name = format!("{} @{}", operation.ams_name, operation.printer_preset_name);
            validate_output_name(&profile_name)?;
            let value =
                build_custom_profile(source, target, operation, &profile_name, &context.adapter)?;
            let base = PathBuf::from("filament/base");
            insert_pending(
                &mut generated,
                base.join(format!("{profile_name}.json")),
                GeneratedArtifactKind::CustomProfile,
                vec![operation.id.clone()],
                json_bytes(&value)?,
            )?;
            let sidecar = match operation.action {
                PlanAction::Create | PlanAction::AddTarget => {
                    InfoSidecar::pre_sync(context.updated_time, "", SyncAction::None)
                }
                PlanAction::Update | PlanAction::Rename | PlanAction::Replace => context
                    .existing_sidecars
                    .get(&operation.id)
                    .ok_or_else(|| {
                        AppError::Conflict(format!(
                            "existing sidecar is required for {}",
                            operation.action.as_str()
                        ))
                    })?
                    .transition(context.updated_time, SyncAction::Update),
                PlanAction::Skip | PlanAction::Block => unreachable!("filtered above"),
            };
            insert_pending(
                &mut generated,
                base.join(format!("{profile_name}.info")),
                GeneratedArtifactKind::CustomSidecar,
                vec![operation.id.clone()],
                sidecar.to_bytes(),
            )?;
        }

        write_staging_tree(staging_root, generated)
    }
}

pub fn effective_settings_fingerprint(profile: &EffectiveProfile) -> Result<String, AppError> {
    let bytes = serde_json::to_vec(&profile.values)
        .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

#[derive(Debug)]
struct PendingArtifact {
    kind: GeneratedArtifactKind,
    operation_ids: Vec<String>,
    bytes: Vec<u8>,
}

fn validate_output_name(value: &str) -> Result<(), AppError> {
    validate_name_component(value).map_err(|_| AppError::UnsafePath(PathBuf::from(value)))
}

fn validate_staging_root(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        return Ok(());
    }
    if !path.is_dir() {
        return Err(AppError::Conflict(format!(
            "staging path is not a directory: {}",
            path.display()
        )));
    }
    let mut entries = std::fs::read_dir(path).map_err(|error| AppError::io(path, error))?;
    if entries
        .next()
        .transpose()
        .map_err(|error| AppError::io(path, error))?
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "staging directory is not empty: {}",
            path.display()
        )));
    }
    Ok(())
}

fn resolve_and_validate_sources(
    operations: &[&PlanOperation],
    context: &WriterContext<'_>,
) -> Result<BTreeMap<ProfileId, EffectiveProfile>, AppError> {
    let mut result = BTreeMap::new();
    for operation in operations {
        if result.contains_key(&operation.source_id) {
            continue;
        }
        let effective = context.sources.resolve_name(&operation.source_id.0)?;
        context
            .adapter
            .field_policy
            .validate_cross_application(effective.values.keys().map(String::as_str))?;
        let rejected: Vec<_> = effective
            .values
            .keys()
            .filter(|key| context.adapter.field_policy.classify(key) == Some(FieldClass::Reject))
            .cloned()
            .collect();
        if !rejected.is_empty() {
            return Err(AppError::UnsupportedSchema(format!(
                "rejected source fields: {}",
                rejected.join(", ")
            )));
        }
        let actual = effective_settings_fingerprint(&effective)?;
        if actual != operation.source_settings_fingerprint {
            return Err(AppError::Conflict(format!(
                "source fingerprint changed for {}",
                operation.source_id
            )));
        }
        result.insert(operation.source_id.clone(), effective);
    }
    Ok(result)
}

fn resolve_targets(
    operations: &[&PlanOperation],
    context: &WriterContext<'_>,
) -> Result<BTreeMap<String, EffectiveProfile>, AppError> {
    let mut result = BTreeMap::new();
    for operation in operations {
        if result.contains_key(&operation.id) {
            continue;
        }
        let profile_name = context
            .target_profiles
            .get(&operation.id)
            .or_else(|| context.target_profiles.get(&operation.printer_id))
            .ok_or_else(|| {
                AppError::UnsupportedSchema(format!(
                    "no characterized target profile for {}",
                    operation.printer_id
                ))
            })?;
        let effective = context.targets.resolve_name(profile_name)?;
        context
            .adapter
            .field_policy
            .validate_cross_application(effective.values.keys().map(String::as_str))?;
        result.insert(operation.id.clone(), effective);
    }
    Ok(result)
}

fn common_target<'a>(
    operations: &[&PlanOperation],
    targets: &'a BTreeMap<String, EffectiveProfile>,
) -> Result<&'a EffectiveProfile, AppError> {
    let first = operations
        .first()
        .and_then(|operation| targets.get(&operation.id))
        .ok_or_else(|| AppError::InvalidProfile("target profile was not resolved".to_owned()))?;
    if operations.iter().any(|operation| {
        targets
            .get(&operation.id)
            .is_none_or(|target| target.name != first.name)
    }) {
        return Err(AppError::Conflict(
            "one slicing preset name resolves to multiple target profiles".to_owned(),
        ));
    }
    Ok(first)
}

fn build_normal_profile(
    source: &EffectiveProfile,
    target: &EffectiveProfile,
    name: &str,
    operations: &[&PlanOperation],
    adapter: &BambuAdapter,
) -> Result<Value, AppError> {
    let mut object = transfer_values(source, target, adapter)?;
    let mut compatible: Vec<_> = operations
        .iter()
        .map(|operation| operation.printer_preset_name.clone())
        .collect();
    compatible.sort();
    compatible.dedup();
    object.insert("compatible_printers".to_owned(), strings(compatible));
    object.insert(
        "filament_extruder_variant".to_owned(),
        target_variants(target)?,
    );
    object.insert("filament_settings_id".to_owned(), strings([name]));
    object.insert("from".to_owned(), Value::String("User".to_owned()));
    object.insert("inherits".to_owned(), Value::String(target.name.clone()));
    object.insert("name".to_owned(), Value::String(name.to_owned()));
    object.insert(
        "version".to_owned(),
        Value::String(adapter.profile_version.to_owned()),
    );
    object.remove("filament_id");
    Ok(Value::Object(object))
}

fn build_custom_profile(
    source: &EffectiveProfile,
    target: &EffectiveProfile,
    operation: &PlanOperation,
    profile_name: &str,
    adapter: &BambuAdapter,
) -> Result<Value, AppError> {
    let mut object = transfer_values(source, target, adapter)?;
    object.insert(
        "compatible_printers".to_owned(),
        strings([operation.printer_preset_name.as_str()]),
    );
    object.insert(
        "filament_extruder_variant".to_owned(),
        target_variants(target)?,
    );
    object.insert(
        "filament_id".to_owned(),
        Value::String(operation.filament_id.clone()),
    );
    object.insert("filament_settings_id".to_owned(), strings([profile_name]));
    object.insert("from".to_owned(), Value::String("User".to_owned()));
    object.insert("inherits".to_owned(), Value::String(String::new()));
    object.insert("name".to_owned(), Value::String(profile_name.to_owned()));
    object.insert(
        "version".to_owned(),
        Value::String(adapter.profile_version.to_owned()),
    );
    Ok(Value::Object(object))
}

fn transfer_values(
    source: &EffectiveProfile,
    target: &EffectiveProfile,
    adapter: &BambuAdapter,
) -> Result<Map<String, Value>, AppError> {
    let mut output = Map::new();
    for (key, value) in &target.values {
        if matches!(
            adapter.field_policy.classify(key),
            Some(FieldClass::SourceMaterial | FieldClass::TargetMachine)
        ) {
            output.insert(key.clone(), value.clone());
        }
    }
    for (key, value) in &source.values {
        if adapter.field_policy.classify(key) == Some(FieldClass::SourceMaterial) {
            output.insert(key.clone(), value.clone());
        }
    }
    let mapped_keys: BTreeSet<_> = source
        .values
        .keys()
        .chain(target.values.keys())
        .filter(|key| adapter.field_policy.classify(key) == Some(FieldClass::Mapped))
        .cloned()
        .collect();
    for key in mapped_keys {
        if let Some(value) = normalize_mapped_vector(&key, source, target)? {
            output.insert(key, value);
        }
    }
    Ok(output)
}

fn normalize_mapped_vector(
    key: &str,
    source: &EffectiveProfile,
    target: &EffectiveProfile,
) -> Result<Option<Value>, AppError> {
    let target_variants =
        string_array(target.value("filament_extruder_variant"), "target variants")?;
    if target_variants.is_empty() {
        return Err(AppError::UnsupportedSchema(
            "target profile has no extruder variants".to_owned(),
        ));
    }
    let Some(target_value) = target.value(key) else {
        return Ok(source.value(key).cloned());
    };
    let target_values = string_array(Some(target_value), key)?;
    let source_values = match source.value(key) {
        Some(value) => string_array(Some(value), key)?,
        None => Vec::new(),
    };
    let source_variants = match source.value("filament_extruder_variant") {
        Some(value) => string_array(Some(value), "source variants")?,
        None => Vec::new(),
    };
    let mut mapped = Vec::with_capacity(target_variants.len());
    for (index, variant) in target_variants.iter().enumerate() {
        let exact_source = source_variants
            .iter()
            .position(|candidate| candidate == variant)
            .and_then(|position| source_values.get(position));
        let source_standard = if matches!(
            variant.as_str(),
            "Direct Drive Standard" | "Direct Drive High Flow"
        ) {
            source_values.first()
        } else {
            None
        };
        let target_default = target_values
            .get(index)
            .or_else(|| (target_values.len() == 1).then(|| &target_values[0]));
        let value = exact_source
            .or(source_standard)
            .or(target_default)
            .ok_or_else(|| {
                AppError::UnsupportedSchema(format!(
                    "no characterized {key} mapping for target variant {variant}"
                ))
            })?;
        mapped.push(Value::String(value.clone()));
    }
    Ok(Some(Value::Array(mapped)))
}

fn target_variants(target: &EffectiveProfile) -> Result<Value, AppError> {
    target
        .value("filament_extruder_variant")
        .cloned()
        .ok_or_else(|| {
            AppError::UnsupportedSchema("target profile has no extruder variants".to_owned())
        })
}

fn string_array(value: Option<&Value>, label: &str) -> Result<Vec<String>, AppError> {
    let value =
        value.ok_or_else(|| AppError::UnsupportedSchema(format!("missing array for {label}")))?;
    let values = value
        .as_array()
        .ok_or_else(|| AppError::UnsupportedSchema(format!("{label} is not an array")))?;
    values
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                AppError::UnsupportedSchema(format!("{label} contains a non-string value"))
            })
        })
        .collect()
}

fn strings(values: impl IntoIterator<Item = impl AsRef<str>>) -> Value {
    Value::Array(
        values
            .into_iter()
            .map(|value| Value::String(value.as_ref().to_owned()))
            .collect(),
    )
}

fn insert_pending(
    generated: &mut BTreeMap<PathBuf, PendingArtifact>,
    relative_path: PathBuf,
    kind: GeneratedArtifactKind,
    operation_ids: Vec<String>,
    bytes: Vec<u8>,
) -> Result<(), AppError> {
    if generated
        .insert(
            relative_path.clone(),
            PendingArtifact {
                kind,
                operation_ids,
                bytes,
            },
        )
        .is_some()
    {
        return Err(AppError::Conflict(format!(
            "duplicate generated path: {}",
            relative_path.display()
        )));
    }
    Ok(())
}

fn write_staging_tree(
    staging_root: &Path,
    generated: BTreeMap<PathBuf, PendingArtifact>,
) -> Result<StagedRun, AppError> {
    let root_existed = staging_root.exists();
    std::fs::create_dir_all(staging_root).map_err(|error| AppError::io(staging_root, error))?;
    let result = (|| {
        let mut artifacts = Vec::with_capacity(generated.len());
        for (relative_path, pending) in generated {
            let path = staging_root.join(&relative_path);
            let parent = path.parent().ok_or_else(|| {
                AppError::InvalidProfile(format!(
                    "generated path has no parent: {}",
                    path.display()
                ))
            })?;
            std::fs::create_dir_all(parent).map_err(|error| AppError::io(parent, error))?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|error| AppError::io(&path, error))?;
            file.write_all(&pending.bytes)
                .map_err(|error| AppError::io(&path, error))?;
            file.sync_all()
                .map_err(|error| AppError::io(&path, error))?;
            artifacts.push(GeneratedArtifact {
                kind: pending.kind,
                operation_ids: pending.operation_ids,
                relative_path,
                sha256: format!("{:x}", Sha256::digest(&pending.bytes)),
                size: pending.bytes.len() as u64,
            });
        }
        Ok(StagedRun {
            root: staging_root.to_path_buf(),
            artifacts,
            deletions: Vec::new(),
        })
    })();
    if result.is_err() {
        if root_existed {
            let _ = remove_staged_contents(staging_root);
        } else {
            let _ = std::fs::remove_dir_all(staging_root);
        }
    }
    result
}

fn remove_staged_contents(root: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn json_bytes(value: &Value) -> Result<Vec<u8>, AppError> {
    let mut bytes = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, formatter);
    value
        .serialize(&mut serializer)
        .map_err(|error| AppError::InvalidProfile(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn is_writable(action: PlanAction) -> bool {
    !matches!(action, PlanAction::Skip | PlanAction::Block)
}
