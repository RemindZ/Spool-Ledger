use bambu_filament_migrator::material::{MaterialClass, classify_material};

fn class(filament_type: &str, name: &str, vendor: &str) -> MaterialClass {
    classify_material(filament_type, name, vendor)
}

fn assert_class(filament_type: &str, name: &str, vendor: &str, archetype: &str, kind: &str) {
    assert_eq!(
        class(filament_type, name, vendor),
        MaterialClass {
            archetype: archetype.to_owned(),
            material_type: kind.to_owned(),
        },
        "{vendor} {name} ({filament_type})"
    );
}

#[test]
fn plain_presets_fall_back_to_standard_under_their_material() {
    assert_class("PLA", "Fiberlogy Easy PLA", "Fiberlogy", "PLA", "Standard");
    assert_class("PETG", "Overture PETG", "Overture", "PETG", "Standard");
    assert_class("PLA", "Bambu PLA Basic", "Bambu Lab", "PLA", "Standard");
    assert_class("PC", "Generic PC Blend", "Generic", "PC", "Standard");
}

#[test]
fn filament_type_variants_collapse_to_their_archetype() {
    assert_class("PA6-CF", "Bambu PA6-CF", "Bambu Lab", "PA", "Carbon fiber");
    assert_class(
        "PAHT-CF",
        "Bambu PAHT-CF",
        "Bambu Lab",
        "PA",
        "Carbon fiber",
    );
    assert_class("PA12", "Fiberlogy PA12", "Fiberlogy", "PA", "Standard");
    assert_class("PLA-CF", "Bambu PLA-CF", "Bambu Lab", "PLA", "Carbon fiber");
    assert_class(
        "TPU-AMS",
        "Bambu TPU for AMS",
        "Bambu Lab",
        "TPU",
        "For AMS",
    );
    assert_class("pla", "Overture PLA", "Overture", "PLA", "Standard");
}

#[test]
fn name_keywords_choose_the_material_type() {
    assert_class("PLA", "Overture Matte PLA", "Overture", "PLA", "Matte");
    assert_class("PLA", "Overture ePLA-Matte", "Overture", "PLA", "Matte");
    assert_class("PLA", "Fiberlogy eSilk PLA", "Fiberlogy", "PLA", "Silk");
    assert_class(
        "PLA",
        "Fiberlogy Rapid PLA+",
        "Fiberlogy",
        "PLA",
        "High speed",
    );
    assert_class(
        "PLA",
        "Fiberlogy Hyper PLA",
        "Fiberlogy",
        "PLA",
        "High speed",
    );
    assert_class("PLA", "Overture PLA+HS", "Overture", "PLA", "High speed");
    assert_class("PETG", "Bambu PETG HF", "Bambu Lab", "PETG", "High speed");
    assert_class(
        "PETG",
        "Overture PETG-rCF08",
        "Overture",
        "PETG",
        "Carbon fiber",
    );
    assert_class(
        "PA6",
        "Fiberlogy PA6-GF25",
        "Fiberlogy",
        "PA",
        "Glass fiber",
    );
    assert_class("ABS", "Bambu ABS-GF", "Bambu Lab", "ABS", "Glass fiber");
    assert_class("ASA", "Bambu ASA Aero", "Bambu Lab", "ASA", "Aero");
    assert_class(
        "TPU",
        "Overture PolyFlex TPU95",
        "Overture",
        "TPU",
        "Shore 95A",
    );
    assert_class("TPU", "Bambu TPU 95A HF", "Bambu Lab", "TPU", "Shore 95A");
    assert_class("PLA", "Overture PLA Pro", "Overture", "PLA", "PLA+ & Pro");
    assert_class("PLA", "Fiberlogy PLA+", "Fiberlogy", "PLA", "PLA+ & Pro");
    assert_class(
        "PETG",
        "Overture PETG Plus",
        "Overture",
        "PETG",
        "PETG+ & Pro",
    );
}

#[test]
fn keyword_precedence_prefers_the_most_specific_property() {
    assert_class("PLA", "Fiberlogy Silk PLA+", "Fiberlogy", "PLA", "Silk");
    assert_class(
        "PLA",
        "Overture PLA Matte (dry box)",
        "Overture",
        "PLA",
        "Matte",
    );
    assert_class(
        "PLA",
        "Overture PLA-CF Matte",
        "Overture",
        "PLA",
        "Carbon fiber",
    );
}

#[test]
fn vendor_names_never_supply_type_keywords() {
    assert_class(
        "PLA",
        "Matterhackers PLA",
        "Matterhackers",
        "PLA",
        "Standard",
    );
    assert_class("PLA", "SilkWorks PLA", "SilkWorks", "PLA", "Standard");
    assert_class("PETG", "Prusament PETG", "Prusament", "PETG", "Standard");
}

#[test]
fn unknown_or_missing_materials_are_never_guessed() {
    assert_class("", "Overture Mystery", "Overture", "Other", "Standard");
    assert_class(
        "   ",
        "Overture PLA lookalike",
        "Overture",
        "Other",
        "Standard",
    );
    assert_class("PVA", "Overture PVA", "Overture", "PVA", "Standard");
    assert_class(
        "PPS-CF",
        "Overture PPS-CF",
        "Overture",
        "PPS",
        "Carbon fiber",
    );
}
