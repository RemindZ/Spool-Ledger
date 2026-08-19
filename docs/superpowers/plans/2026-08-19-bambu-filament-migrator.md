# Bambu Filament Migrator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and verify the cross-platform Tauri application specified by `SCOPE.md`, beginning with a characterized Windows proof and never writing to live slicer profile roots during development or testing.

**Architecture:** A Rust library owns discovery, parsing, inheritance, field policy, planning, generation, transactions, receipts, process handling, and sync evidence. A Tauri command layer exposes typed summaries and operations to a Svelte 5 frontend. All tests use synthetic fixtures or copied temporary slicer roots; the live Bambu/Orca roots are read-only characterization inputs.

**Tech Stack:** Rust 1.94+, Tauri 2, Svelte 5, TypeScript 5, Vite, Vitest, Testing Library, serde, serde_json, regex, sha2, md5, zip, chrono, directories, sysinfo, tempfile, GitHub Actions.

**Spec:** `SCOPE.md`

## Global Constraints

- License is AGPL-3.0.
- Public name is **Bambu Filament Migrator**; repository slug is `bambu-filament-migrator`.
- Direct profile generation is blocked until Phase 0 characterization passes for the frozen Windows x64 matrix row.
- Never write to installed slicer resources or the user's live profile roots in tests or acceptance automation.
- Never store credentials or call undocumented Bambu cloud APIs.
- Never force-close Bambu Studio.
- Automated evidence stops at `cloud_id_assigned`; only an operator can record `ams_verified`.
- Unknown cross-application fields block writes unless a reviewed field policy classifies them.
- Rollback touches only journal-owned paths whose hashes still match committed hashes.
- Official printers are manifest-discovered; custom printers remain hidden behind the **Show custom printers** toggle and marked unverified.
- UI uses Bambu green/charcoal foundations with restrained Orca orange accents, complete light/dark/system themes, keyboard access, and no color-only states.
- One source tree targets Windows, macOS, and Linux; release claims remain conditional on compatibility-matrix acceptance.

---

## File structure

```text
.
├── LICENSE
├── README.md
├── SCOPE.md
├── package.json
├── package-lock.json
├── index.html
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── tsconfig.node.json
├── src/
│   ├── main.ts
│   ├── App.svelte
│   ├── app.css
│   └── lib/
│       ├── api.ts
│       ├── types.ts
│       ├── state.ts
│       ├── components/
│       │   ├── DiscoveryBar.svelte
│       │   ├── FilterPanel.svelte
│       │   ├── SourceTable.svelte
│       │   ├── TargetPanel.svelte
│       │   ├── NamingPanel.svelte
│       │   ├── PlanTable.svelte
│       │   ├── RunProgress.svelte
│       │   └── ResultSummary.svelte
│       └── __tests__/
│           ├── state.test.ts
│           ├── filters.test.ts
│           └── workflow.test.ts
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── resources/field-policy-v1.json
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands.rs
│   │   ├── model.rs
│   │   ├── error.rs
│   │   ├── discovery.rs
│   │   ├── profiles.rs
│   │   ├── resolver.rs
│   │   ├── field_policy.rs
│   │   ├── targets.rs
│   │   ├── naming.rs
│   │   ├── planner.rs
│   │   ├── writer.rs
│   │   ├── transaction.rs
│   │   ├── receipt.rs
│   │   ├── sync.rs
│   │   └── platform.rs
│   └── tests/
│       ├── fixtures.rs
│       ├── resolver.rs
│       ├── planner.rs
│       ├── writer.rs
│       ├── transaction.rs
│       └── copied_roots.rs
├── fixtures/
│   ├── synthetic/
│   │   ├── bambu-system/
│   │   ├── bambu-user-pre-sync/
│   │   ├── bambu-user-post-sync/
│   │   └── orca-system/
│   └── expected/
├── docs/
│   ├── compatibility/README.md
│   ├── compatibility/windows-x64-baseline.md
│   ├── field-policy.md
│   ├── receipts.md
│   └── superpowers/plans/2026-08-19-bambu-filament-migrator.md
└── .github/workflows/ci.yml
```

---

### Task 1: Repository and testable Rust/Tauri foundation

**Files:**
- Create: `LICENSE`
- Create: `README.md`
- Create: `package.json`, `package-lock.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `tsconfig.node.json`, `index.html`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/{main,lib,error,model}.rs`
- Create: `src/main.ts`, `src/App.svelte`, `src/app.css`
- Test: `src-tauri/src/model.rs` unit tests

**Interfaces:**
- Produces `AppError`, `SourceApp`, `SourceKind`, `ProfileId`, `SourceProfileSummary`, `PrinterTarget`, `NozzleTarget`, `EvidenceLevel`, and `OperationState`.
- Produces a buildable Tauri shell and a callable Rust library crate named `bambu_filament_migrator`.

- [ ] **Step 1: Write the failing model serialization test**

```rust
#[test]
fn evidence_levels_use_stable_snake_case_values() {
    assert_eq!(serde_json::to_string(&EvidenceLevel::CloudIdAssigned).unwrap(), "\"cloud_id_assigned\"");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml evidence_levels_use_stable_snake_case_values`
Expected: FAIL because the crate/model does not exist.

- [ ] **Step 3: Scaffold the minimal Tauri/Svelte project and typed models**

Use serde-tagged enums and newtype IDs; do not add state management or plugins beyond Tauri core yet.

- [ ] **Step 4: Verify GREEN and build both halves**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm install
npm run check
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all exit 0.

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "chore: scaffold Tauri application"
```

---

### Task 2: Phase 0 characterization and compatibility gate

**Files:**
- Create: `docs/compatibility/README.md`
- Create: `docs/compatibility/windows-x64-baseline.md`
- Create: `fixtures/synthetic/**`
- Create: `fixtures/expected/**`
- Create: `src-tauri/src/profiles.rs`
- Test: `src-tauri/tests/fixtures.rs`
- Create: `.gitignore` entries for `.local-characterization/`, real profile copies, Tauri targets, and frontend build output

**Interfaces:**
- Produces `ProfileDocument`, `InfoSidecar`, `ArtifactContract`, `SchemaFingerprint`, and fixture loaders.
- Consumes live roots read-only only to generate local ignored evidence; committed fixtures are synthetic/sanitized.

- [ ] **Step 1: Write failing byte-contract tests**

```rust
#[test]
fn pre_sync_sidecar_round_trips_byte_for_byte() {
    let bytes = fixture("synthetic/bambu-user-pre-sync/custom.info");
    let parsed = InfoSidecar::parse(&bytes).unwrap();
    assert_eq!(parsed.to_bytes(), bytes);
    assert!(parsed.setting_id.is_empty());
}
```

Also add tests for post-sync, create, additional target, rename, update, delete, and hold fixture shapes.

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test fixtures`
Expected: FAIL because fixture parsing/contracts are missing.

- [ ] **Step 3: Capture the frozen Windows baseline without writes**

Record exact installed app versions, profile schema versions, relevant source hashes, source-code references, current post-sync artifacts, and backup-derived pre-sync differences in `.local-characterization/`. Build redistribution-safe synthetic fixtures with fictional vendor/material names and structurally exact fields.

- [ ] **Step 4: Implement strict JSON and INI-like sidecar parsing**

Parsing preserves unknown JSON values and the version-specific paired `.info` sidecar's field order, line endings, blank values, and integer timestamps. `ArtifactContract` identifies the exact matrix row and allowed initial/transition shapes.

- [ ] **Step 5: Verify Phase 0 fixture gate**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test fixtures`
Expected: all fixture contracts pass.

- [ ] **Step 6: Commit**

```bash
git add .gitignore docs/compatibility fixtures src-tauri/src/profiles.rs src-tauri/tests/fixtures.rs
git commit -m "test: characterize Bambu profile artifacts"
```

---

### Task 3: Inheritance resolver and versioned field policy

**Files:**
- Create: `src-tauri/src/resolver.rs`
- Create: `src-tauri/src/field_policy.rs`
- Create: `src-tauri/resources/field-policy-v1.json`
- Create: `docs/field-policy.md`
- Test: `src-tauri/tests/resolver.rs`

**Interfaces:**
- Produces `ProfileCatalog::load`, `Resolver::resolve`, `EffectiveProfile`, `ValueProvenance`, `FieldClass`, and `FieldPolicyTable::classify`.
- Resolver signature: `resolve(&self, id: &ProfileId) -> Result<EffectiveProfile, AppError>`.

- [ ] **Step 1: Write failing resolver tests**

Cover three-level inheritance, includes, child precedence, user/system shadowing, cycles, missing parents, ambiguous parents, `nil`, null, empty vectors, and non-instantiable bases.

```rust
#[test]
fn child_override_wins_and_keeps_provenance() {
    let resolved = catalog().resolver().resolve(&id("User PLA")).unwrap();
    assert_eq!(resolved.string_values("filament_flow_ratio"), ["0.97"]);
    assert_eq!(resolved.provenance("filament_flow_ratio").file_name(), "User PLA.json");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test resolver`
Expected: FAIL because resolver/policy do not exist.

- [ ] **Step 3: Implement graph resolution and condition parsing**

Implement deterministic root precedence, DFS cycle detection, ordered includes/inheritance, typed merge semantics, and a small expression evaluator for observed compatibility-condition grammar. Unsupported expressions return a blocking error.

- [ ] **Step 4: Implement exhaustive policy classification**

Each known field is `source_material`, `target_machine`, `mapped`, `derived`, `metadata`, or `reject`. Any unknown Orca-to-Bambu key blocks planning and is surfaced with source path.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test resolver`
Expected: all resolver/policy tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/resolver.rs src-tauri/src/field_policy.rs src-tauri/resources docs/field-policy.md src-tauri/tests/resolver.rs
git commit -m "feat: resolve profile inheritance safely"
```

---

### Task 4: Cross-platform discovery and target catalog

**Files:**
- Create: `src-tauri/src/discovery.rs`
- Create: `src-tauri/src/platform.rs`
- Create: `src-tauri/src/targets.rs`
- Test: unit tests in each module

**Interfaces:**
- Produces `DiscoveryService::scan`, `DiscoverySnapshot`, `Installation`, `AccountRoot`, `AccountEligibility`, `TargetCatalog::from_manifest`, and `PrinterPresetKind`.
- Platform trait exposes default roots, process names, executable candidates, and canonical path guards.

- [ ] **Step 1: Write failing discovery tests using temporary roots**

```rust
#[test]
fn empty_account_is_not_writable_by_default() {
    let account = inspect_account(temp_empty_account()).unwrap();
    assert_eq!(account.eligibility, AccountEligibility::Empty);
    assert!(!account.writable_by_default());
}
```

Add manifest tests proving official printers are visible and custom printers are tagged/hidden by default, with independent nozzle choices.

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml discovery`
Expected: FAIL because discovery/platform/targets are missing.

- [ ] **Step 3: Implement platform adapters and manual overrides**

Use `directories` and explicit candidate tables per OS. Discovery never writes. Canonicalize manual paths and classify default/local, stale, backup, empty, unsupported, and eligible account roots.

- [ ] **Step 4: Implement manifest-driven target discovery**

Read installed manifests/profile files, derive official/custom kind, printer code, nozzles, extruder variants, and adapter support state without a fixed printer model list.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml discovery targets platform`
Expected: all pass on the host OS; platform-path unit tests cover all OS modules.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/{discovery,platform,targets}.rs
git commit -m "feat: discover slicers and target printers"
```

---

### Task 5: Naming, filtering, fingerprints, and deterministic planning

**Files:**
- Create: `src-tauri/src/naming.rs`
- Create: `src-tauri/src/planner.rs`
- Test: `src-tauri/tests/planner.rs`

**Interfaces:**
- Produces `NamingTemplate`, `ReplacementRule`, `FilterQuery`, `IdentityFingerprint`, `MigrationRequest`, `MigrationPlan`, `PlanOperation`, `Conflict`, and `Planner::build`.
- Planner is pure: `build(snapshot, request) -> Result<MigrationPlan, Vec<PlanIssue>>` and performs no writes.

- [ ] **Step 1: Write failing naming/planner tests**

Cover placeholders, clean-name deduplication, wildcard matching, regex captures, invalid regex, conditional rules, filename validation, case-only collisions, persisted hidden selections, same-settings skip, missing-target add, identity collision, and deterministic plan ordering.

```rust
#[test]
fn panchroma_default_name_does_not_duplicate_pla() {
    let result = template("{vendor} {material} {clean_name}")
        .render(context("Polymaker", "PLA", "Panchroma PLA Satin"))
        .unwrap();
    assert_eq!(result, "Polymaker PLA Panchroma Satin");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test planner`
Expected: FAIL because naming/planner do not exist.

- [ ] **Step 3: Implement naming and fingerprinting**

Use explicit placeholders, ordered wildcard/regex rules, cross-platform reserved-name checks, canonical identity tuple, settings fingerprint, and adapter-declared name-derived ID behavior.

- [ ] **Step 4: Implement pure planner**

Generate create/add-target/update/rename/replace/skip/block operations. Default all conflicts to block/skip; never overwrite silently. Freeze source/precondition hashes into the plan.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test planner`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/{naming,planner}.rs src-tauri/tests/planner.rs
git commit -m "feat: plan deterministic migrations"
```

---

### Task 6: Bambu writer, identity generation, and sidecar contracts

**Files:**
- Create: `src-tauri/src/writer.rs`
- Test: `src-tauri/tests/writer.rs`
- Extend: `fixtures/expected/**`

**Interfaces:**
- Produces `Writer::stage(plan, roots) -> Result<StagedRun, AppError>`, `GeneratedArtifact`, `BambuIdentity`, and adapter-specific `SidecarContract` application.
- Writes only under an explicitly supplied staging root.

- [ ] **Step 1: Write failing golden writer tests**

Cover Orca factory/user to Bambu source, Bambu user to custom-only, selected nozzles only, Standard/High Flow/E3D vector policies, deterministic `P.......` ID, exact JSON fields, exact pre-sync `.info` bytes, additional target, rename, update, delete/hold metadata, and unknown-field rejection.

```rust
#[test]
fn writer_never_invents_cloud_setting_id() {
    let staged = stage_synthetic_custom().unwrap();
    let info = staged.info_for("Synthetic PLA @Bambu Lab H2C 0.4 nozzle");
    assert_eq!(info.setting_id, "");
    assert_eq!(info.to_bytes(), fixture("expected/pre-sync.info"));
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test writer`
Expected: FAIL because writer is absent.

- [ ] **Step 3: Implement policy-driven transformation**

Apply only classified fields, preserve target-machine fields, execute field-specific mapped/derived rules, flatten inheritance, normalize variant vectors through policy, and generate exact JSON/sidecar pairs.

- [ ] **Step 4: Implement adapter-owned identity semantics**

Generate current name-derived IDs, detect existing equivalent identities by normalized tuple/fingerprint, preserve IDs for updates, and block unsafe rename/re-key behavior.

- [ ] **Step 5: Verify GREEN and golden diffs**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test writer`
Expected: all byte/semantic golden tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/writer.rs src-tauri/tests/writer.rs fixtures/expected
git commit -m "feat: generate Bambu filament artifacts"
```

---

### Task 7: Per-file transaction journal, backup, rollback, and receipts

**Files:**
- Create: `src-tauri/src/transaction.rs`
- Create: `src-tauri/src/receipt.rs`
- Create: `docs/receipts.md`
- Test: `src-tauri/tests/transaction.rs`

**Interfaces:**
- Produces `Transaction::preflight`, `commit`, `rollback_owned`, `restore_preview`, `RunReceipt`, `JournalEntry`, and `CommitOutcome`.
- Consumes a frozen `MigrationPlan` and `StagedRun`.

- [ ] **Step 1: Write failing fault-injection tests**

Cover precondition mismatch, partial commit, same-directory temp replacement, created-file rollback, updated-file rollback, deleted-file rollback, unrelated post-run file preservation, externally modified owned-path preservation, backup checksum, and receipt round trip.

```rust
#[test]
fn rollback_preserves_unrelated_profile_created_after_commit() {
    let run = commit_fixture_run();
    write_unrelated_profile(&run.destination);
    run.transaction.rollback_owned().unwrap();
    assert!(run.destination.join("Unrelated.json").exists());
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test transaction`
Expected: FAIL because transaction/receipt do not exist.

- [ ] **Step 3: Implement backup and journaled commit**

Create checksummed ZIP evidence, record pre-run bytes/absence and hashes, recheck each precondition, and commit each path through same-directory temp files.

- [ ] **Step 4: Implement hash-guarded rollback and restore preview**

Only journal-owned unchanged committed paths may roll back. Externally changed paths remain untouched and are reported.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test transaction`
Expected: all fault-injection tests pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/{transaction,receipt}.rs src-tauri/tests/transaction.rs docs/receipts.md
git commit -m "feat: commit migrations transactionally"
```

---

### Task 8: Graceful process handling and synchronization evidence

**Files:**
- Create: `src-tauri/src/sync.rs`
- Extend: `src-tauri/src/platform.rs`
- Test: unit tests with fake process/clock/filesystem traits

**Interfaces:**
- Produces `ProcessController`, `SyncMonitor`, `SyncExpectation`, `SyncObservation`, and evidence transitions through `CloudIdAssigned` only.
- External process/filesystem/time dependencies are traits for deterministic tests.

- [ ] **Step 1: Write failing state-machine tests**

Cover already stopped, graceful-close accepted, save prompt/still running, launch failure, Bambu exit, blank ID, duplicate ID, unique ID, timeout, local acknowledgement, and prohibition on automated `ams_verified`.

```rust
#[test]
fn cloud_id_never_implies_ams_verified() {
    let result = monitor_with_ids(["PFUSabc"]).finish();
    assert_eq!(result.evidence, EvidenceLevel::CloudIdAssigned);
    assert_ne!(result.evidence, EvidenceLevel::AmsVerified);
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml sync platform`
Expected: FAIL because sync interfaces are missing.

- [ ] **Step 3: Implement process adapters and monitor**

Use normal OS close requests where available, never force kill, detect unresolved running state, launch the selected executable, and poll characterized local artifacts with cancellation and timeout.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml sync platform`
Expected: all state-machine tests pass without launching real slicers.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/{sync,platform}.rs
git commit -m "feat: monitor Bambu synchronization evidence"
```

---

### Task 9: Typed Tauri command surface

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`
- Test: command/service tests in `commands.rs`

**Interfaces:**
- Commands: `discover`, `catalog_sources`, `catalog_targets`, `preview_names`, `build_plan`, `execute_plan`, `cancel_run`, `record_ams_verification`, `restore_preview`, `restore_owned`.
- Commands accept opaque IDs and typed requests; paths are canonicalized in Rust.

- [ ] **Step 1: Write failing command-boundary tests**

Assert unapproved paths, stale plan IDs, changed source hashes, ineligible accounts, running Bambu, unsupported schemas, and frontend-supplied arbitrary paths are rejected.

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands`
Expected: FAIL because commands are absent.

- [ ] **Step 3: Implement service state and commands**

Store discovery snapshot/catalog/plans behind managed application state. Serialize only frontend-required data. Emit typed run-progress events.

- [ ] **Step 4: Restrict capabilities**

Allow only core window/event/dialog capabilities required by the UI. Do not expose generic filesystem or shell plugins to the webview.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml commands && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src-tauri
git commit -m "feat: expose safe Tauri migration commands"
```

---

### Task 10: Bambu/Orca visual system and application shell

**Required skill:** invoke `frontend-design:frontend-design` before implementation.

**Files:**
- Modify: `src/app.css`, `src/App.svelte`
- Create: `src/lib/types.ts`, `src/lib/api.ts`, `src/lib/state.ts`
- Create: `src/lib/components/DiscoveryBar.svelte`
- Test: `src/lib/__tests__/state.test.ts`

**Interfaces:**
- Produces typed frontend API wrappers and one central app state reducer/store.
- Visual tokens include Bambu green, deep graphite, cool neutral surfaces, Orca orange accent, semantic success/warning/error colors, and complete theme overrides.

- [ ] **Step 1: Write failing frontend state tests**

Cover discovery loading/error/ready, selected account, persisted theme, stale-plan invalidation, and progress evidence levels.

- [ ] **Step 2: Verify RED**

Run: `npm test -- --run src/lib/__tests__/state.test.ts`
Expected: FAIL because frontend state is missing.

- [ ] **Step 3: Implement visual tokens and shell**

Create system/light/dark palettes, typography scale, focus states, reduced-motion handling, responsive three-region workspace, and restrained Orca orange for source/migration accents rather than status semantics.

- [ ] **Step 4: Implement typed API/state and discovery bar**

No raw path operations occur in frontend code. Errors show actionable path/version details.

- [ ] **Step 5: Verify GREEN and accessibility basics**

Run: `npm test -- --run src/lib/__tests__/state.test.ts && npm run check && npm run build`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src package*.json *.json *.ts *.js index.html
git commit -m "feat: add Bambu-inspired application shell"
```

---

### Task 11: Source catalog filters and persistent selection

**Files:**
- Create: `src/lib/components/FilterPanel.svelte`
- Create: `src/lib/components/SourceTable.svelte`
- Test: `src/lib/__tests__/filters.test.ts`

**Interfaces:**
- Consumes `SourceProfileSummary`, `FilterFacets`, and central state.
- Produces filter query and stable selected source IDs independent of current visibility.

- [ ] **Step 1: Write failing component/state tests**

Cover text/source app/source kind/vendor/material/family/compatible/status filters, multi-select chips, counts, select visible, deselect visible, reset filters without clearing selection, selected-only mode, and unsupported-source diagnostics.

- [ ] **Step 2: Verify RED**

Run: `npm test -- --run src/lib/__tests__/filters.test.ts`
Expected: FAIL because components/filter logic are absent.

- [ ] **Step 3: Implement filters and virtualized/contained source list**

Use semantic controls, keyboard navigation, stable IDs, visible selection counts, and no color-only status. Keep rendering responsive for thousands of rows.

- [ ] **Step 4: Verify GREEN**

Run: `npm test -- --run src/lib/__tests__/filters.test.ts && npm run check`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components src/lib/__tests__/filters.test.ts
git commit -m "feat: filter and select source profiles"
```

---

### Task 12: Target, nozzle, naming, and plan workspace

**Files:**
- Create: `src/lib/components/TargetPanel.svelte`
- Create: `src/lib/components/NamingPanel.svelte`
- Create: `src/lib/components/PlanTable.svelte`
- Extend: `src/lib/__tests__/workflow.test.ts`

**Interfaces:**
- Consumes catalogs and naming/planner API.
- Produces selected target/nozzle IDs, custom-printer toggle, naming templates/rules, per-row overrides, and explicit conflict decisions.

- [ ] **Step 1: Write failing workflow tests**

Cover official default visibility, custom-printer toggle/unverified marker, independent nozzle selection, select-all-nozzles, separate preset/AMS templates, regex error blocking, live preview, per-row name override, conflict default skip/block, and plan invalidation after input changes.

- [ ] **Step 2: Verify RED**

Run: `npm test -- --run src/lib/__tests__/workflow.test.ts`
Expected: FAIL because destination/naming/plan components are absent.

- [ ] **Step 3: Implement the three panels**

Keep the single workspace; advanced regex and output toggles are collapsed by default. Every planned filename and target remains visible before migration.

- [ ] **Step 4: Verify GREEN**

Run: `npm test -- --run src/lib/__tests__/workflow.test.ts && npm run check && npm run build`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib/components src/lib/__tests__/workflow.test.ts
git commit -m "feat: preview migration targets and names"
```

---

### Task 13: Migration execution, evidence, results, and restore UI

**Files:**
- Create: `src/lib/components/RunProgress.svelte`
- Create: `src/lib/components/ResultSummary.svelte`
- Modify: `src/App.svelte`, `src/lib/api.ts`, `src/lib/state.ts`
- Extend: `src/lib/__tests__/workflow.test.ts`

**Interfaces:**
- Consumes frozen plan IDs and progress events.
- Produces cancellation, evidence display, operator AMS checklist, retry sync, restore preview, and journal-owned restore actions.

- [ ] **Step 1: Write failing execution UI tests**

Cover close-required state, save-prompt pause, backup/write progress, partial failure counts, local-vs-cloud-vs-AMS evidence, duplicate ID error, operator-only AMS verification, retry, restore preview, and externally changed path warning.

- [ ] **Step 2: Verify RED**

Run: `npm test -- --run src/lib/__tests__/workflow.test.ts`
Expected: FAIL for execution/result states.

- [ ] **Step 3: Implement progress and result surfaces**

Use exact counts and paths, no generic success toast, no automatic AMS claim, and an explicit restore-impact preview.

- [ ] **Step 4: Verify GREEN**

Run: `npm test -- --run && npm run check && npm run build`
Expected: all frontend tests/build pass.

- [ ] **Step 5: Commit**

```bash
git add src
git commit -m "feat: report migration and sync outcomes"
```

---

### Task 14: Copied-root integration and Windows Panchroma acceptance

**Files:**
- Create: `src-tauri/tests/copied_roots.rs`
- Extend: `docs/compatibility/windows-x64-baseline.md`
- Create ignored: `.local-characterization/copied-bambu-root/`, `.local-characterization/copied-orca-root/`

**Interfaces:**
- Exercises the full discovery → catalog → resolve → plan → stage → commit → journal/receipt path against copies only.

- [ ] **Step 1: Copy live roots into ignored test roots using read-only source access**

Verify source/destination paths differ and record source hashes before any operation. All writes target a fresh temporary copy of the copied destination.

- [ ] **Step 2: Write failing copied-root acceptance test**

The test selects the 16 Panchroma source families and H2C 0.2/0.4/0.6/0.8 targets, then asserts 16 slicing presets, 16 identities, 64 target JSON/sidecars, unique IDs, no unknown fields, and a complete journal/receipt.

- [ ] **Step 3: Verify RED before wiring the full service path**

Run: `cargo test --manifest-path src-tauri/Cargo.toml --test copied_roots -- --nocapture`
Expected: FAIL on the first missing integration boundary.

- [ ] **Step 4: Complete only the missing service wiring**

Do not special-case Panchroma names or H2C. Fix generic adapter/policy/plan behavior until the copied-root acceptance passes.

- [ ] **Step 5: Prove no live writes**

Hash every live source/profile file before and after the test and assert equality; assert every receipt path is inside the temporary destination.

- [ ] **Step 6: Verify GREEN**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm test -- --run
npm run check
npm run build
```

Expected: all pass; live hashes unchanged.

- [ ] **Step 7: Commit only code/docs, never copied roots**

```bash
git add src-tauri docs/compatibility .gitignore
git commit -m "test: verify migration against copied profiles"
```

---

### Task 15: CI, packaging, documentation, and public release readiness

**Files:**
- Create: `.github/workflows/ci.yml`
- Complete: `README.md`
- Create: `SECURITY.md`, `CONTRIBUTING.md`, `NOTICE.md`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces draft artifacts only for platform rows allowed by the compatibility matrix.

- [ ] **Step 1: Add failing local CI parity script/config validation**

Validate workflow syntax, required jobs, artifact checksums, no unsupported release claims, AGPL notices, independence disclaimer, and fixture licensing.

- [ ] **Step 2: Implement CI matrix**

Run Rust fmt/clippy/tests, frontend formatting/typecheck/tests/build, Tauri smoke builds, dependency/license audit, and checksums. Package only characterized matrix rows; unaccepted rows remain build experiments, not release assets.

- [ ] **Step 3: Complete user and contributor docs**

Document supported matrix, safety model, local/offline truth, custom-printer warning, naming rules, conflict behavior, receipts/restores, privacy, and independent-project disclaimer.

- [ ] **Step 4: Run full verification**

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm ci
npm run format:check
npm run lint
npm test -- --run
npm run check
npm run build
npm run tauri build -- --no-bundle
```

Expected: every command exits 0.

- [ ] **Step 5: Run independent correctness and simplification reviews**

Review against every `SCOPE.md` acceptance criterion, confirm no live writes, remove speculative dependencies/abstractions, and rerun the full verification pipeline.

- [ ] **Step 6: Commit release-ready implementation**

```bash
git add .
git commit -m "feat: complete Bambu Filament Migrator v1"
```

- [ ] **Step 7: Create the authorized public GitHub repository only after local verification**

```bash
gh repo create Remindz/bambu-filament-migrator --public --source . --remote origin --push --description "Migrate OrcaSlicer and Bambu Studio filament presets into Bambu custom filaments for AMS-compatible printers."
```

Expected: public repository exists, `main` matches the verified local commit, and no local characterization data is present.
