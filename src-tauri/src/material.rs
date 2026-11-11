//! Material archetype and type classification for the source picker.
//!
//! The archetype comes from the resolved `filament_type`. The type comes from
//! keywords in the preset name after the vendor prefix is removed. Presets
//! without a recognised keyword stay under their material as "Standard", and
//! presets without a material go under "Other". Nothing is guessed.

use serde::{Deserialize, Serialize};

pub const OTHER_MATERIAL: &str = "Other";
pub const STANDARD_TYPE: &str = "Standard";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialClass {
    pub archetype: String,
    pub material_type: String,
}

pub fn classify_material(filament_type: &str, name: &str, vendor: &str) -> MaterialClass {
    let archetype = material_archetype(filament_type);
    let material_type = if archetype == OTHER_MATERIAL {
        STANDARD_TYPE.to_owned()
    } else {
        material_type(&archetype, filament_type, without_vendor(name, vendor))
    };
    MaterialClass {
        archetype,
        material_type,
    }
}

fn material_archetype(filament_type: &str) -> String {
    let normalized = filament_type.trim().to_ascii_uppercase();
    let base = normalized
        .split(|character: char| !character.is_ascii_alphanumeric())
        .find(|token| !token.is_empty())
        .unwrap_or("");
    if base.is_empty() {
        return OTHER_MATERIAL.to_owned();
    }
    if base == "PAHT" || (base.starts_with("PA") && base[2..].chars().all(|c| c.is_ascii_digit())) {
        return "PA".to_owned();
    }
    if base.starts_with("TPU") {
        return "TPU".to_owned();
    }
    base.to_owned()
}

fn without_vendor<'a>(name: &'a str, vendor: &str) -> &'a str {
    let name = name.trim();
    let vendor = vendor.trim();
    if !vendor.is_empty()
        && name
            .get(..vendor.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(vendor))
    {
        return name[vendor.len()..].trim();
    }
    name
}

fn material_type(archetype: &str, filament_type: &str, product: &str) -> String {
    let text = format!("{filament_type} {product}").to_ascii_lowercase();
    let tokens: Vec<&str> = text
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();
    let has = |predicate: &dyn Fn(&str) -> bool| tokens.iter().any(|token| predicate(token));

    if has(&|token| fiber_token(token, "cf") || token == "carbon") {
        return "Carbon fiber".to_owned();
    }
    if has(&|token| fiber_token(token, "gf") || token == "glass") {
        return "Glass fiber".to_owned();
    }
    if has(&|token| token == "aero") {
        return "Aero".to_owned();
    }
    if has(&|token| token == "ams") {
        return "For AMS".to_owned();
    }
    if archetype == "TPU"
        && let Some(hardness) = tokens.iter().find_map(|token| shore_hardness(token))
    {
        return format!("Shore {hardness}A");
    }
    if has(&|token| token.contains("silk")) {
        return "Silk".to_owned();
    }
    if has(&|token| token == "matte" || token == "matt") {
        return "Matte".to_owned();
    }
    if has(&|token| {
        matches!(
            token,
            "hs" | "hf" | "hyper" | "rapid" | "speed" | "highspeed"
        )
    }) {
        return "High speed".to_owned();
    }
    if product.contains('+') || has(&|token| token == "pro" || token == "plus") {
        return format!("{archetype}+ & Pro");
    }
    STANDARD_TYPE.to_owned()
}

/// Matches `cf`, `cf20`, and recycled `rcf08` style fiber tokens.
fn fiber_token(token: &str, fiber: &str) -> bool {
    let token = token.strip_prefix('r').unwrap_or(token);
    token
        .strip_prefix(fiber)
        .is_some_and(|rest| rest.chars().all(|c| c.is_ascii_digit()))
}

/// Reads a two-digit Shore A hardness from `95a` or `tpu95` style tokens.
fn shore_hardness(token: &str) -> Option<&str> {
    let digits = token
        .strip_suffix('a')
        .filter(|rest| rest.len() == 2)
        .or_else(|| {
            token
                .strip_prefix("tpu")
                .map(|rest| rest.strip_suffix('a').unwrap_or(rest))
                .filter(|rest| rest.len() == 2)
        })?;
    digits.chars().all(|c| c.is_ascii_digit()).then_some(digits)
}
