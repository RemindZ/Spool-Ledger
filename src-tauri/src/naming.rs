use crate::AppError;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const PLACEHOLDERS: [&str; 11] = [
    "source_name",
    "clean_name",
    "vendor",
    "material",
    "family",
    "variant",
    "source_app",
    "source_kind",
    "printer",
    "printer_code",
    "nozzle",
];

#[derive(Debug, Clone)]
pub struct NamingContext {
    values: BTreeMap<&'static str, String>,
}

impl NamingContext {
    pub fn new(source_name: &str, vendor: &str, material: &str) -> Self {
        let mut values = BTreeMap::new();
        values.insert("source_name", source_name.to_owned());
        values.insert("clean_name", clean_name(source_name, vendor, material));
        values.insert("vendor", vendor.to_owned());
        values.insert("material", material.to_owned());
        values.insert("family", String::new());
        values.insert("variant", String::new());
        values.insert("source_app", String::new());
        values.insert("source_kind", String::new());
        values.insert("printer", String::new());
        values.insert("printer_code", String::new());
        values.insert("nozzle", String::new());
        Self { values }
    }

    pub fn with(mut self, key: &'static str, value: impl Into<String>) -> Self {
        if PLACEHOLDERS.contains(&key) {
            self.values.insert(key, value.into());
        }
        self
    }

    fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

#[derive(Debug, Clone)]
pub struct NamingTemplate {
    template: String,
    placeholders: Vec<String>,
}

impl NamingTemplate {
    pub fn parse(template: impl Into<String>) -> Result<Self, AppError> {
        let template = template.into();
        let placeholder_re = Regex::new(r"\{([a-z_]+)\}").expect("constant regex");
        let placeholders: Vec<String> = placeholder_re
            .captures_iter(&template)
            .map(|capture| capture[1].to_owned())
            .collect();
        for placeholder in &placeholders {
            if !PLACEHOLDERS.contains(&placeholder.as_str()) {
                return Err(AppError::InvalidProfile(format!(
                    "unknown naming placeholder: {{{placeholder}}}"
                )));
            }
        }
        let stripped = placeholder_re.replace_all(&template, "");
        if stripped.contains('{') || stripped.contains('}') {
            return Err(AppError::InvalidProfile(
                "invalid naming placeholder syntax".to_owned(),
            ));
        }
        Ok(Self {
            template,
            placeholders,
        })
    }

    pub fn render(&self, context: &NamingContext) -> Result<String, AppError> {
        let mut result = self.template.clone();
        for placeholder in &self.placeholders {
            let value = context.value(placeholder).unwrap_or("");
            result = result.replace(&format!("{{{placeholder}}}"), value);
        }
        let result = normalize_spaces(&result);
        validate_name_component(&result)?;
        Ok(result)
    }

    pub fn apply_rules(value: &str, rules: &[ReplacementRule]) -> Result<String, AppError> {
        let mut result = value.to_owned();
        for rule in rules.iter().filter(|rule| rule.condition.is_none()) {
            result = rule.apply(&result);
        }
        let result = normalize_spaces(&result);
        validate_name_component(&result)?;
        Ok(result)
    }

    pub fn apply_rules_with_context(
        value: &str,
        rules: &[ReplacementRule],
        context: &NamingContext,
    ) -> Result<String, AppError> {
        let mut result = value.to_owned();
        for rule in rules.iter().filter(|rule| rule.matches(context)) {
            result = rule.apply(&result);
        }
        let result = normalize_spaces(&result);
        validate_name_component(&result)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionField {
    SourceApp,
    SourceKind,
    Vendor,
    Material,
    Family,
}

impl ConditionField {
    fn key(self) -> &'static str {
        match self {
            Self::SourceApp => "source_app",
            Self::SourceKind => "source_kind",
            Self::Vendor => "vendor",
            Self::Material => "material",
            Self::Family => "family",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RulePatternKind {
    Wildcard,
    Regex,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleConditionSpec {
    pub field: ConditionField,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplacementRuleSpec {
    pub kind: RulePatternKind,
    pub pattern: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub condition: Option<RuleConditionSpec>,
}

impl ReplacementRuleSpec {
    pub fn compile(&self) -> Result<ReplacementRule, AppError> {
        if self.pattern.is_empty() {
            return Err(AppError::InvalidProfile(
                "naming rule pattern is empty".to_owned(),
            ));
        }
        let mut rule = match self.kind {
            RulePatternKind::Wildcard => {
                ReplacementRule::wildcard(&self.pattern, &self.replacement, self.case_sensitive)?
            }
            RulePatternKind::Regex => ReplacementRule::regex_with_case(
                &self.pattern,
                &self.replacement,
                self.case_sensitive,
            )?,
        };
        if let Some(condition) = &self.condition {
            if condition.value.trim().is_empty() {
                return Err(AppError::InvalidProfile(
                    "naming rule condition value is empty".to_owned(),
                ));
            }
            rule = rule.when(RuleCondition::is(condition.field, &condition.value));
        }
        Ok(rule)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleCondition {
    field: ConditionField,
    value: String,
}

impl RuleCondition {
    pub fn is(field: ConditionField, value: impl Into<String>) -> Self {
        Self {
            field,
            value: value.into(),
        }
    }

    fn matches(&self, context: &NamingContext) -> bool {
        context
            .value(self.field.key())
            .is_some_and(|value| normalize_condition(value) == normalize_condition(&self.value))
    }
}

#[derive(Debug, Clone)]
pub struct ReplacementRule {
    regex: Regex,
    replacement: String,
    condition: Option<RuleCondition>,
}

impl ReplacementRule {
    pub fn regex(pattern: &str, replacement: &str) -> Result<Self, AppError> {
        Self::regex_with_case(pattern, replacement, true)
    }

    pub fn regex_with_case(
        pattern: &str,
        replacement: &str,
        case_sensitive: bool,
    ) -> Result<Self, AppError> {
        let regex = RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|error| AppError::InvalidProfile(format!("invalid regex: {error}")))?;
        Ok(Self {
            regex,
            replacement: replacement.to_owned(),
            condition: None,
        })
    }

    pub fn wildcard(
        pattern: &str,
        replacement: &str,
        case_sensitive: bool,
    ) -> Result<Self, AppError> {
        let expression = format!("^{}$", regex::escape(pattern).replace(r"\*", "(.*)"));
        let regex = RegexBuilder::new(&expression)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|error| AppError::InvalidProfile(format!("invalid wildcard: {error}")))?;
        Ok(Self {
            regex,
            replacement: replacement.to_owned(),
            condition: None,
        })
    }

    pub fn when(mut self, condition: RuleCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    fn matches(&self, context: &NamingContext) -> bool {
        self.condition
            .as_ref()
            .is_none_or(|condition| condition.matches(context))
    }

    pub fn apply(&self, value: &str) -> String {
        self.regex
            .replace_all(value, self.replacement.as_str())
            .into_owned()
    }
}

pub fn validate_name_component(value: &str) -> Result<(), AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidProfile(
            "generated name is empty".to_owned(),
        ));
    }
    if trimmed.ends_with('.') || trimmed.ends_with(' ') {
        return Err(AppError::InvalidProfile(format!(
            "generated name has an invalid ending: {value}"
        )));
    }
    if trimmed.chars().any(|character| {
        matches!(
            character,
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
        )
    }) {
        return Err(AppError::InvalidProfile(format!(
            "generated name contains a reserved character: {value}"
        )));
    }
    let stem = trimmed
        .split('.')
        .next()
        .unwrap_or(trimmed)
        .to_ascii_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&stem.as_str()) {
        return Err(AppError::InvalidProfile(format!(
            "generated name is reserved: {value}"
        )));
    }
    Ok(())
}

fn clean_name(source_name: &str, vendor: &str, material: &str) -> String {
    let mut value = source_name.trim().to_owned();
    if value
        .get(..vendor.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(vendor))
    {
        value = value[vendor.len()..].trim().to_owned();
    }
    let material_tokens: Vec<_> = material.split_whitespace().collect();
    let tokens: Vec<_> = value
        .split_whitespace()
        .filter(|token| {
            !material_tokens
                .iter()
                .any(|material| token.eq_ignore_ascii_case(material))
        })
        .collect();
    normalize_spaces(&tokens.join(" "))
}

fn normalize_condition(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_spaces(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
