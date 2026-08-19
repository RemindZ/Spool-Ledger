export type SourceApp = "orca_slicer" | "bambu_studio";
export type SourceKind = "factory_system" | "user_custom";
export type MigrationStatus =
  "new" | "already_migrated" | "incomplete" | "conflicting" | "unsupported";
export type AccountEligibility =
  "eligible" | "empty" | "stale" | "default_local" | "backup" | "unsupported";
export type EvidenceLevel =
  "created_local" | "loaded_by_bambu" | "cloud_id_assigned" | "ams_verified";
export type OperationState =
  | "planned"
  | "created_local"
  | "updated_local"
  | "skipped_existing"
  | "skipped_by_user"
  | "blocked_invalid_source"
  | "blocked_missing_parent"
  | "blocked_unsupported_schema"
  | "blocked_conflict"
  | "rolled_back"
  | "created_local_unsynchronized"
  | "loaded_by_bambu"
  | "cloud_id_assigned"
  | "ams_verified";

export interface AccountRoot {
  id: string;
  eligibility: AccountEligibility;
  filament_profile_count: number;
}

export interface DiscoveryResponse {
  source_root_ids: string[];
  target_catalog_ids: string[];
  accounts: AccountRoot[];
}

export interface CatalogSource {
  id: string;
  name: string;
  vendor: string;
  material: string;
  family: string;
  variant: string;
  source_app: SourceApp;
  source_kind: SourceKind;
  compatible_printers: string[];
  migration_status: MigrationStatus;
  warnings: string[];
}

export interface NozzleTarget {
  id: string;
  diameter: string;
  printer_preset_name: string;
  selected: boolean;
  supported: boolean;
}

export interface PrinterTarget {
  id: string;
  name: string;
  code: string;
  kind: "official" | "custom";
  verified: boolean;
  extruder_variants: string[];
  nozzles: NozzleTarget[];
}

export type PlanAction =
  "create" | "add_target" | "update" | "rename" | "replace" | "skip" | "block";

export interface Conflict {
  kind: "case_only_name" | "identity_collision" | "settings_mismatch";
  message: string;
}

export interface IdentityFingerprint {
  name: string;
  printer: string;
  nozzle: string;
}

export interface PlanOperation {
  id: string;
  source_id: string;
  source_name: string;
  preset_name: string;
  ams_name: string;
  filament_id: string;
  printer_id: string;
  printer_name: string;
  printer_preset_name: string;
  nozzle: string;
  custom_unverified: boolean;
  source_precondition_fingerprint: string;
  material_settings_fingerprint: string;
  precondition_fingerprint: string | null;
  identity_fingerprint: IdentityFingerprint;
  action: PlanAction;
  conflict: Conflict | null;
}

export interface MigrationPlan {
  id: string;
  operations: PlanOperation[];
}

export interface NozzleSelection {
  printer_id: string;
  diameters: string[];
}

export type RulePatternKind = "wildcard" | "regex";
export type RuleConditionField =
  "source_app" | "source_kind" | "vendor" | "material" | "family";

export interface RuleConditionSpec {
  field: RuleConditionField;
  value: string;
}

export interface NamingRule {
  id: string;
  kind: RulePatternKind;
  pattern: string;
  replacement: string;
  case_sensitive: boolean;
  condition: RuleConditionSpec | null;
}

export interface NamingRuleSpec {
  kind: RulePatternKind;
  pattern: string;
  replacement: string;
  case_sensitive: boolean;
  condition: RuleConditionSpec | null;
}

export interface NameOverride {
  source_id: string;
  printer_id: string;
  nozzle: string;
  preset_name: string | null;
  ams_name: string | null;
}

export interface NamingPreset {
  id: string;
  name: string;
  preset_template: string;
  ams_template: string;
  preset_rules: NamingRule[];
  ams_rules: NamingRule[];
}

export interface NamePreview {
  preset_before: string;
  preset_name: string;
  ams_before: string;
  ams_name: string;
}

export interface LocalRunResult {
  run_id: string;
  plan_id: string;
  committed_files: number;
  receipt_path: string;
}

export interface RestorePreviewPath {
  path: string;
  action: "create" | "update" | "delete";
  safe_to_restore: boolean;
  current_sha256: string | null;
  expected_committed_sha256: string | null;
}

export interface RestorePreview {
  paths: RestorePreviewPath[];
}

export interface RollbackOutcome {
  rolled_back: number;
  external_conflicts: number;
  failed: number;
}
