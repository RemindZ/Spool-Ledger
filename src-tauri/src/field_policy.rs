use crate::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldClass {
    SourceMaterial,
    TargetMachine,
    Mapped,
    Derived,
    Metadata,
    Reject,
}

#[derive(Debug, Clone)]
pub struct FieldPolicyTable {
    fields: BTreeMap<String, FieldClass>,
}

impl FieldPolicyTable {
    pub fn bundled() -> Result<Self, AppError> {
        Self::from_json(include_str!("../resources/field-policy-v1.json"))
    }

    pub fn from_json(json: &str) -> Result<Self, AppError> {
        let fields = serde_json::from_str(json).map_err(|error| {
            AppError::InvalidProfile(format!("invalid field policy JSON: {error}"))
        })?;
        Ok(Self { fields })
    }

    pub fn classify(&self, key: &str) -> Option<FieldClass> {
        self.fields.get(key).copied()
    }

    pub fn validate_cross_application<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), AppError> {
        let unknown: Vec<_> = keys
            .into_iter()
            .filter(|key| !self.fields.contains_key(*key))
            .collect();
        if unknown.is_empty() {
            Ok(())
        } else {
            Err(AppError::UnsupportedSchema(format!(
                "unclassified fields: {}",
                unknown.join(", ")
            )))
        }
    }

    pub fn transferable(&self, key: &str) -> bool {
        matches!(
            self.classify(key),
            Some(FieldClass::SourceMaterial | FieldClass::Mapped | FieldClass::Derived)
        )
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}
