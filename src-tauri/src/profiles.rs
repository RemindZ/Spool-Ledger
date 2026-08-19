use crate::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const REQUIRED_INFO_FIELDS: [&str; 5] = [
    "sync_info",
    "user_id",
    "setting_id",
    "base_id",
    "updated_time",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncAction {
    None,
    Create,
    Update,
    Delete,
    Hold,
}

impl SyncAction {
    fn parse(value: &str) -> Result<Self, AppError> {
        match value.trim() {
            "" => Ok(Self::None),
            "create" => Ok(Self::Create),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            "hold" => Ok(Self::Hold),
            value => Err(AppError::InvalidProfile(format!(
                "unsupported sync_info value: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoSidecar {
    raw: Vec<u8>,
    values: BTreeMap<String, String>,
}

impl InfoSidecar {
    pub fn parse(bytes: &[u8]) -> Result<Self, AppError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|error| AppError::InvalidProfile(format!("sidecar is not UTF-8: {error}")))?;
        let mut values = BTreeMap::new();
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once(" =") else {
                return Err(AppError::InvalidProfile(format!(
                    "invalid sidecar line: {line}"
                )));
            };
            let value = value.strip_prefix(' ').unwrap_or(value);
            if values.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(AppError::InvalidProfile(format!(
                    "duplicate sidecar field: {key}"
                )));
            }
        }
        for key in REQUIRED_INFO_FIELDS {
            if !values.contains_key(key) {
                return Err(AppError::InvalidProfile(format!(
                    "missing sidecar field: {key}"
                )));
            }
        }
        SyncAction::parse(&values["sync_info"])?;
        values["updated_time"]
            .parse::<i64>()
            .map_err(|_| AppError::InvalidProfile("updated_time must be an integer".to_owned()))?;
        Ok(Self {
            raw: bytes.to_vec(),
            values,
        })
    }

    pub fn pre_sync(updated_time: i64, user_id: &str, action: SyncAction) -> Self {
        let action = match action {
            SyncAction::None => "",
            SyncAction::Create => "create",
            SyncAction::Update => "update",
            SyncAction::Delete => "delete",
            SyncAction::Hold => "hold",
        };
        let raw = format!(
            "sync_info = {action}\nuser_id = {user_id}\nsetting_id = \nbase_id = \nupdated_time = {updated_time}\n"
        )
        .into_bytes();
        Self::parse(&raw).expect("generated sidecar must be valid")
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.raw.clone()
    }

    pub fn sync_action(&self) -> SyncAction {
        SyncAction::parse(&self.values["sync_info"]).expect("validated during parse")
    }

    pub fn user_id(&self) -> &str {
        &self.values["user_id"]
    }

    pub fn setting_id(&self) -> &str {
        &self.values["setting_id"]
    }

    pub fn base_id(&self) -> &str {
        &self.values["base_id"]
    }

    pub fn updated_time(&self) -> i64 {
        self.values["updated_time"]
            .parse()
            .expect("validated during parse")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDocument {
    pub path: PathBuf,
    pub value: Value,
}

impl ProfileDocument {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AppError> {
        let path = path.as_ref().to_path_buf();
        let bytes = std::fs::read(&path).map_err(|error| AppError::io(&path, error))?;
        let value = serde_json::from_slice(&bytes).map_err(|source| AppError::Json {
            path: path.clone(),
            source,
        })?;
        Ok(Self { path, value })
    }

    pub fn name(&self) -> Option<&str> {
        self.value.get("name").and_then(Value::as_str)
    }

    pub fn inherits(&self) -> Option<&str> {
        self.value.get("inherits").and_then(Value::as_str)
    }

    pub fn is_instantiable(&self) -> bool {
        self.value
            .get("instantiation")
            .and_then(Value::as_str)
            .is_none_or(|value| value == "true")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaFingerprint {
    pub application_version: String,
    pub profile_version: String,
    pub manifest_version: String,
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactContract {
    pub id: String,
    pub schema: SchemaFingerprint,
    pub sidecar_fields: Vec<String>,
    pub supported_sync_actions: Vec<SyncAction>,
}
