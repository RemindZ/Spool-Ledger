use bambu_filament_migrator::model::{ProfileId, SourceApp, SourceKind};
use bambu_filament_migrator::naming::{
    ConditionField, NamingContext, NamingTemplate, ReplacementRule, ReplacementRuleSpec,
    RuleCondition, RuleConditionSpec, RulePatternKind,
};
use bambu_filament_migrator::planner::{
    ConflictChoice, ConflictDecision, DestinationIndex, ExistingDestination, FilterQuery,
    IdentityFingerprint, MaterialFingerprintIndex, MigrationRequest, MigrationSource, NameOverride,
    NamingOptions, Planner, TargetSelection,
};

fn source(name: &str) -> MigrationSource {
    MigrationSource {
        id: ProfileId::new(name),
        name: name.to_owned(),
        vendor: "Polymaker".to_owned(),
        material: "PLA".to_owned(),
        family: "Panchroma".to_owned(),
        compatible_printers: std::collections::BTreeSet::from(["Bambu Lab X1 Carbon".to_owned()]),
        migration_status: bambu_filament_migrator::planner::MigrationStatus::New,
        variant: "Satin".to_owned(),
        source_app: SourceApp::OrcaSlicer,
        source_kind: SourceKind::FactorySystem,
        source_precondition_fingerprint: "settings-a".to_owned(),
        existing_filament_id: None,
    }
}

fn target(nozzle: &str) -> TargetSelection {
    TargetSelection {
        printer_id: "official:H2C".to_owned(),
        printer_name: "Bambu Lab H2C".to_owned(),
        printer_code: "H2C".to_owned(),
        nozzle: nozzle.to_owned(),
        printer_preset_name: format!("Bambu Lab H2C {nozzle} nozzle"),
        custom_unverified: false,
    }
}

#[test]
fn panchroma_default_name_does_not_duplicate_pla() {
    let context = NamingContext::new("Panchroma PLA Satin", "Polymaker", "PLA");
    let result = NamingTemplate::parse("{vendor} {material} {clean_name}")
        .unwrap()
        .render(&context)
        .unwrap();
    assert_eq!(result, "Polymaker PLA Panchroma Satin");
}

#[test]
fn regex_and_wildcard_rules_apply_in_order() {
    let rules = vec![
        ReplacementRule::wildcard("Panchroma PLA *", "Panchroma $1", false).unwrap(),
        ReplacementRule::regex("(?i)satin", "Satin Finish").unwrap(),
    ];
    let result = NamingTemplate::apply_rules("Panchroma PLA satin", &rules).unwrap();
    assert_eq!(result, "Panchroma Satin Finish");
}

#[test]
fn invalid_regex_is_blocking() {
    let error = ReplacementRule::regex("[", "x").unwrap_err();
    assert!(error.to_string().contains("regex"));
}

#[test]
fn conditional_rule_only_changes_matching_sources() {
    let rule = ReplacementRule::regex("Satin", "Matte")
        .unwrap()
        .when(RuleCondition::is(ConditionField::Material, "PLA"));
    let pla =
        NamingContext::new("Panchroma PLA Satin", "Polymaker", "PLA").with("variant", "Satin");
    let petg =
        NamingContext::new("Panchroma PETG Satin", "Polymaker", "PETG").with("variant", "Satin");
    assert_eq!(
        NamingTemplate::apply_rules_with_context(
            "Panchroma Satin",
            std::slice::from_ref(&rule),
            &pla,
        )
        .unwrap(),
        "Panchroma Matte"
    );
    assert_eq!(
        NamingTemplate::apply_rules_with_context("Panchroma Satin", &[rule], &petg).unwrap(),
        "Panchroma Satin"
    );
}

#[test]
fn serialized_rule_specs_preserve_case_and_conditions() {
    let spec = ReplacementRuleSpec {
        kind: RulePatternKind::Wildcard,
        pattern: "Polymaker PLA *".to_owned(),
        replacement: "$1".to_owned(),
        case_sensitive: false,
        condition: Some(RuleConditionSpec {
            field: ConditionField::Family,
            value: "Panchroma".to_owned(),
        }),
    };
    let encoded = serde_json::to_value(&spec).unwrap();
    assert_eq!(encoded["kind"], "wildcard");
    assert_eq!(encoded["condition"]["field"], "family");
    let context =
        NamingContext::new("Panchroma PLA Satin", "Polymaker", "PLA").with("family", "Panchroma");
    let rule = spec.compile().unwrap();
    assert_eq!(
        NamingTemplate::apply_rules_with_context("polymaker pla Satin", &[rule], &context).unwrap(),
        "Satin"
    );
}

#[test]
fn planner_applies_ordered_rules_then_exact_row_override() {
    let request = MigrationRequest {
        sources: vec![source("Panchroma PLA Satin")],
        targets: vec![target("0.4"), target("0.6")],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let options = NamingOptions {
        preset_rules: vec![ReplacementRuleSpec {
            kind: RulePatternKind::Regex,
            pattern: "Panchroma".to_owned(),
            replacement: "PolyTerra".to_owned(),
            case_sensitive: true,
            condition: None,
        }],
        ams_rules: vec![ReplacementRuleSpec {
            kind: RulePatternKind::Wildcard,
            pattern: "Polymaker PLA *".to_owned(),
            replacement: "$1".to_owned(),
            case_sensitive: false,
            condition: Some(RuleConditionSpec {
                field: ConditionField::SourceApp,
                value: "orca_slicer".to_owned(),
            }),
        }],
        overrides: vec![NameOverride {
            source_id: ProfileId::new("Panchroma PLA Satin"),
            printer_id: "official:H2C".to_owned(),
            nozzle: "0.6".to_owned(),
            preset_name: Some("Hand tuned 0.6".to_owned()),
            ams_name: None,
        }],
        conflict_decisions: vec![],
    };
    let plan =
        Planner::build_with_naming(&request, &DestinationIndex::default(), &options).unwrap();
    assert_eq!(plan.operations[0].preset_name, "PolyTerra Satin - H2C");
    assert_eq!(plan.operations[0].ams_name, "Panchroma Satin");
    assert_eq!(plan.operations[1].preset_name, "Hand tuned 0.6");
    assert_eq!(plan.operations[1].ams_name, "Panchroma Satin");
}

#[test]
fn target_placeholders_and_reserved_names_are_validated() {
    let context = NamingContext::new("PLA", "Acme", "PLA")
        .with("printer_code", "H2C")
        .with("nozzle", "0.4");
    assert_eq!(
        NamingTemplate::parse("{printer_code} {nozzle}")
            .unwrap()
            .render(&context)
            .unwrap(),
        "H2C 0.4"
    );
    assert!(
        NamingTemplate::parse("CON")
            .unwrap()
            .render(&context)
            .is_err()
    );
    assert!(
        NamingTemplate::parse("bad/name")
            .unwrap()
            .render(&context)
            .is_err()
    );
}

#[test]
fn filter_query_combines_facets_without_losing_selection() {
    let mut selected = std::collections::BTreeSet::from([
        ProfileId::new("selected-hidden"),
        ProfileId::new("selected-visible"),
    ]);
    let mut other = source("Other PLA");
    other.family = "Other".to_owned();
    let sources = vec![source("Panchroma PLA Satin"), other];
    let query = FilterQuery {
        search: "panchroma".to_owned(),
        vendors: std::collections::BTreeSet::from(["Polymaker".to_owned()]),
        selected_only: false,
        ..Default::default()
    };
    let visible = query.apply(&sources, &selected);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "Panchroma PLA Satin");
    selected.remove(&ProfileId::new("selected-visible"));
    assert!(selected.contains(&ProfileId::new("selected-hidden")));
}

#[test]
fn identity_fingerprint_normalizes_case_and_whitespace() {
    let left = IdentityFingerprint::new(" Polymaker  PLA ", "H2C", "0.4");
    let right = IdentityFingerprint::new("polymaker pla", "h2c", "0.4");
    assert_eq!(left, right);
}

#[test]
fn planner_is_deterministic_and_selects_only_requested_nozzles() {
    let request = MigrationRequest {
        sources: vec![source("Panchroma PLA Satin")],
        targets: vec![target("0.8"), target("0.4")],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let first = Planner::build(&request, &DestinationIndex::default()).unwrap();
    let second = Planner::build(&request, &DestinationIndex::default()).unwrap();
    assert_eq!(first.id, second.id);
    let nozzles: Vec<_> = first
        .operations
        .iter()
        .map(|item| item.nozzle.as_str())
        .collect();
    assert_eq!(nozzles, vec!["0.4", "0.8"]);
    assert!(
        first
            .operations
            .iter()
            .all(|item| item.ams_name == "Polymaker PLA Panchroma Satin")
    );
}

#[test]
fn case_only_identity_collision_blocks_but_missing_target_is_created() {
    let request = MigrationRequest {
        sources: vec![source("Panchroma PLA Satin")],
        targets: vec![target("0.4"), target("0.6")],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let preview = Planner::build(&request, &DestinationIndex::default()).unwrap();
    let existing = DestinationIndex::from_existing([ExistingDestination {
        name: preview.operations[0].ams_name.to_ascii_lowercase(),
        material_settings_fingerprint: "settings-a".to_owned(),
        filament_id: preview.operations[0].filament_id.clone(),
        printer_preset_name: preview.operations[0].printer_preset_name.clone(),
    }]);
    let case_plan = Planner::build(&request, &existing).unwrap();
    assert_eq!(case_plan.operations[0].action.as_str(), "block");

    let exact = DestinationIndex::from_existing([ExistingDestination {
        name: preview.operations[0].ams_name.clone(),
        material_settings_fingerprint: "settings-a".to_owned(),
        filament_id: preview.operations[0].filament_id.clone(),
        printer_preset_name: preview.operations[0].printer_preset_name.clone(),
    }]);
    let target_plan = Planner::build(&request, &exact).unwrap();
    assert_eq!(target_plan.operations[0].action.as_str(), "skip");
    assert_eq!(target_plan.operations[1].nozzle, "0.6");
    assert_eq!(target_plan.operations[1].action.as_str(), "add_target");
}

#[test]
fn generated_id_collision_blocks_every_conflicting_identity() {
    let mut first = source("Panchroma PLA Satin");
    first.existing_filament_id = Some("P7654321".to_owned());
    let mut second = source("Panchroma PLA Matte");
    second.variant = "Matte".to_owned();
    second.existing_filament_id = Some("P7654321".to_owned());
    let request = MigrationRequest {
        sources: vec![first, second],
        targets: vec![target("0.4")],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let plan = Planner::build(&request, &DestinationIndex::default()).unwrap();
    assert_eq!(plan.operations.len(), 2);
    assert!(
        plan.operations
            .iter()
            .all(|operation| operation.action.as_str() == "block")
    );
    assert!(plan.operations.iter().all(|operation| {
        operation
            .conflict
            .as_ref()
            .is_some_and(|conflict| conflict.message.contains("filament id"))
    }));
}

#[test]
fn one_identity_allows_target_specific_material_fingerprints() {
    let source = source("Panchroma PLA Satin");
    let first_target = target("0.4");
    let second_target = target("0.6");
    let fingerprints = MaterialFingerprintIndex::from([
        (
            (source.id.clone(), first_target.printer_preset_name.clone()),
            "target-settings-0.4".to_owned(),
        ),
        (
            (source.id.clone(), second_target.printer_preset_name.clone()),
            "target-settings-0.6".to_owned(),
        ),
    ]);
    let request = MigrationRequest {
        sources: vec![source],
        targets: vec![first_target, second_target],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };

    let plan = Planner::build_with_context(
        &request,
        &DestinationIndex::default(),
        &NamingOptions::default(),
        &fingerprints,
    )
    .unwrap();

    assert_eq!(plan.operations.len(), 2);
    assert!(
        plan.operations
            .iter()
            .all(|operation| operation.action.as_str() == "create")
    );
    assert_eq!(
        plan.operations[0].filament_id,
        plan.operations[1].filament_id
    );
}

#[test]
fn equivalent_destination_is_skipped_but_different_settings_conflict() {
    let request = MigrationRequest {
        sources: vec![source("Panchroma PLA Satin")],
        targets: vec![target("0.4")],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let preview = Planner::build(&request, &DestinationIndex::default()).unwrap();
    let operation = &preview.operations[0];
    let equivalent = DestinationIndex::from_existing([ExistingDestination {
        name: operation.ams_name.clone(),
        material_settings_fingerprint: "settings-a".to_owned(),
        filament_id: operation.filament_id.clone(),
        printer_preset_name: operation.printer_preset_name.clone(),
    }]);
    let skipped = Planner::build(&request, &equivalent).unwrap();
    assert_eq!(skipped.operations[0].action.as_str(), "skip");

    let conflict = DestinationIndex::from_existing([ExistingDestination {
        name: operation.ams_name.clone(),
        material_settings_fingerprint: "settings-b".to_owned(),
        filament_id: operation.filament_id.clone(),
        printer_preset_name: operation.printer_preset_name.clone(),
    }]);
    let blocked = Planner::build(&request, &conflict).unwrap();
    assert_eq!(blocked.operations[0].action.as_str(), "block");
    assert!(blocked.operations[0].conflict.is_some());
}

#[test]
fn explicit_conflict_decisions_update_or_skip_settings_mismatches() {
    let request = MigrationRequest {
        sources: vec![source("Panchroma PLA Satin")],
        targets: vec![target("0.4")],
        preset_template: "{clean_name} - {printer_code}".to_owned(),
        ams_template: "{vendor} {material} {clean_name}".to_owned(),
        user_id: "0000000000".to_owned(),
    };
    let preview = Planner::build(&request, &DestinationIndex::default()).unwrap();
    let operation = &preview.operations[0];
    let destinations = DestinationIndex::from_existing([ExistingDestination {
        name: operation.ams_name.clone(),
        material_settings_fingerprint: "changed-settings".to_owned(),
        filament_id: operation.filament_id.clone(),
        printer_preset_name: operation.printer_preset_name.clone(),
    }]);
    let decision = |choice| NamingOptions {
        conflict_decisions: vec![ConflictDecision {
            source_id: operation.source_id.clone(),
            printer_id: operation.printer_id.clone(),
            nozzle: operation.nozzle.clone(),
            choice,
        }],
        ..NamingOptions::default()
    };

    let updated =
        Planner::build_with_naming(&request, &destinations, &decision(ConflictChoice::Update))
            .unwrap();
    assert_eq!(updated.operations[0].action.as_str(), "update");
    assert!(updated.operations[0].conflict.is_none());

    let skipped =
        Planner::build_with_naming(&request, &destinations, &decision(ConflictChoice::Skip))
            .unwrap();
    assert_eq!(skipped.operations[0].action.as_str(), "skip");
    assert!(skipped.operations[0].conflict.is_none());
}
