# Spool Ledger - Bambu Filament Migrator Scope

- **Status:** Windows implementation verified; built-app dogfood and operator cloud/AMS acceptance pending
- **Date:** 2026-08-23
- **Owner:** Remindz
- **Repository:** `Remindz/bambu-filament-migrator`
- **License:** AGPL-3.0

## 1. Product statement

**Spool Ledger**, permanently described as **Bambu Filament Migrator**, is a lightweight cross-platform desktop application that converts OrcaSlicer and Bambu Studio filament presets into Bambu Studio slicing presets and AMS custom-filament identities without requiring the user to click through Bambu Studio's custom-filament UI for every material, printer, and nozzle.

The application uses a Tauri 2 shell, a Rust backend, and a Svelte 5 + TypeScript frontend. It ships as one user-facing download per supported operating system and performs all profile inspection and generation locally.

## 2. Problem

Bambu Studio separates a slicing preset from the custom-filament identity shown in its AMS interface. Creating an AMS identity through the official UI requires repetitive manual selection and confirmation for every filament and every printer/nozzle target.

Preset migration is also not a direct file copy:

- Orca user presets may inherit factory parents that are absent from Bambu Studio.
- Factory presets are often thin machine-specific overrides rather than complete profiles.
- Bambu custom filaments require flattened per-printer/nozzle profiles.
- Extruder-setting vectors vary between Standard, High Flow, E3D, dual-extruder, and future printer configurations.
- Local custom `P.......` filament IDs and server-assigned `PFUS...` setting IDs have different responsibilities.
- Bambu Studio and OrcaSlicer store system profiles, user profiles, metadata, and account folders separately.

The current manual workflow is slow, error-prone, and impractical for migrating a large vendor catalog.

## 3. Goals

### G-001 - Remove repetitive Bambu UI work

A user can select many source profiles, printers, and nozzle sizes, preview the complete result, and migrate them in one operation. Bambu Studio's per-filament creation dialog is not part of the required workflow.

### G-002 - Support the requested source classes

The application reads:

- OrcaSlicer factory/system filament presets.
- OrcaSlicer user/custom filament presets.
- Bambu Studio user/custom filament presets.

Bambu factory presets may be read to resolve inheritance and provide target templates, but they are not selectable migration sources in v1 because they already exist in Bambu Studio.

### G-003 - Create both destination layers

For Orca sources, the default operation creates:

1. A valid Bambu Studio slicing preset.
2. The flattened Bambu custom-filament profiles used by the AMS filament list.

For a Bambu user source, the default operation reuses the slicing preset and creates only missing custom-filament targets.

### G-004 - Support AMS-capable Bambu printers

Official target printers and nozzles are discovered from the installed Bambu Studio manifests and profiles rather than maintained as a fixed model list.

Only official target printers are presented in v1. User-created/custom printer presets are outside the supported target matrix until a future adapter is separately characterized and accepted.

### G-005 - Work across desktop platforms

The profile engine, planner, naming engine, writer, backup system, and receipt format work on Windows, macOS, and Linux. Platform-specific code is limited to installation discovery, default paths, process detection, graceful application handling, and packaging.

### G-006 - Be safe by default

The application stages and validates all output, creates a timestamped backup, refuses silent overwrites, writes transactionally, records a receipt, and fails closed on unsupported profile schemas.

### G-007 - Remain local and transparent

No telemetry, hosted service, Bambu credentials, profile uploads, or direct undocumented Bambu cloud API calls are used. The UI shows exactly what will be read, created, updated, skipped, or rejected.

## 4. Non-goals for v1

- Editing or calibrating filament parameters.
- Downloading or maintaining third-party profile catalogs.
- Controlling AMS hardware or printer hardware.
- Sending filament data to a project-operated server.
- Storing Bambu credentials or session tokens.
- Calling undocumented Bambu cloud APIs directly.
- Automating Bambu Studio through mouse, keyboard, or accessibility events.
- Modifying files inside the Bambu Studio or OrcaSlicer installation directories.
- Migrating Bambu factory presets that are already available in Bambu Studio.
- Mobile or browser-only builds.
- Automatic application updates.
- Claiming custom-printer hardware compatibility that cannot be verified locally.

## 5. Terminology

| Term | Meaning |
| --- | --- |
| Source preset | A factory or user filament preset selected for migration. |
| Effective profile | The complete settings obtained after resolving inheritance and includes. |
| Bambu slicing preset | A normal user filament preset visible in Bambu Studio's slicing profile list. |
| Custom filament | A Bambu filament identity represented by flattened user profiles and shown in the AMS filament list. |
| Target | One Bambu printer preset and nozzle diameter combination. |
| Identity | The vendor, material, serial/display name, and Bambu `filament_id` shared by a custom filament's target profiles. |
| Setting ID | A `PFUS...` identifier assigned later by Bambu's authenticated synchronization. |
| Migration plan | The immutable, previewed list of file and metadata operations for one run. |
| Receipt | A machine-readable record of the plan, results, checksums, backup, and verification state. |

## 6. Supported environment

### 6.1 Compatibility matrix

Before implementation planning proceeds beyond characterization, the repository records a compatibility matrix containing:

- Exact Bambu Studio application version.
- Exact OrcaSlicer application version.
- Operating system, architecture, and distribution form.
- Profile schema signatures independent of application version.
- Characterized JSON and `.info` artifact contracts.
- Redistribution-safe fixture hashes.
- Discovery, local-write, Bambu-load, cloud-ID, and human AMS acceptance status.

Application versions and profile schema versions are separate compatibility dimensions. Releases support only explicit matrix rows backed by fixtures and acceptance evidence.

Each adapter declares the schema fingerprints it understands:

- Known supported matrix row: planning and migration are enabled.
- Unknown but structurally readable schema: inspection and dry-run diagnostics are enabled; writes are disabled.
- Incompatible or malformed schema: the source/destination is rejected with an actionable report.

Version strings alone are never trusted as proof of compatibility. Initial implementation targets the frozen Windows x64 baseline characterized in Phase 0. macOS, Linux, additional architectures, portable packaging, and additional slicer versions become release claims only after their own matrix rows pass acceptance.

### 6.2 Installation and account discovery

The application checks platform-standard locations and lets the user select a folder manually when discovery fails.

It discovers separately:

- Application installation and version.
- Factory/system profile root.
- User profile root.
- Available account/profile folders and their eligibility state.
- The explicitly selected destination account.

A writable destination is an explicitly selected Bambu user-account root whose structure matches a supported adapter. Empty, stale, default/local, backup, and unrecognized folders are displayed separately and are not writable by default. The application may read the non-secret account identifier already represented by the selected folder or characterized metadata, but it never reads authentication tokens, cookies, or network payloads.

When multiple eligible accounts exist, migration is limited to one explicitly selected Bambu destination account per run.

### 6.3 Source integrity

Source profile directories are read-only from the application's perspective. Migration never rewrites or normalizes source files in place.

## 7. Primary user flow

The application uses a first-run setup for durable defaults, followed by one desktop workspace for discovery, selection, planning, execution, and restore. The migration itself is not a multi-page wizard.

### 7.0 First-run setup

After the real source and target catalogs load, a fresh installation presents first-run setup before the normal workspace. The setup records:

- Source applications: OrcaSlicer, Bambu Studio, or both, limited to applications with eligible discovered sources.
- Source kinds: factory/system, user/custom, or both, limited to valid combinations in the real catalog.
- One or more enabled official printers discovered from the installed Bambu target catalog.
- At least one supported default nozzle for every enabled printer.

The setup cannot complete unless the selected source filters match at least one eligible source and every enabled printer has a supported nozzle. It does not select filament profiles or start planning automatically. Preferences are stored atomically in workspace preferences schema version 2. Version 1 preferences migrate without losing filter, naming, account, or nozzle choices; previously selected official printers become enabled defaults and onboarding appears once after the schema upgrade.

Newly discovered printers remain disabled until the user enables them. Custom printers are never exposed by setup or derived from old preferences.

### 7.1 Discovery bar

The top bar shows:

- Detected OrcaSlicer installation and schema status.
- Detected Bambu Studio installation and schema status.
- Selected source roots.
- Selected destination account/profile folder.
- Running/stopped state for Bambu Studio.
- Blocking warnings and manual path controls.

### 7.2 Source panel

The left panel lists source filament profiles with multi-select support. Selections persist while filters change.

Available filters:

- Free-text search.
- Source application: OrcaSlicer or Bambu Studio.
- Source kind: factory/system or user/custom.
- Manufacturer/vendor.
- Plastic/material type.
- Profile family.
- Variant or serial.
- Compatible source printer.
- Migration status: new, already migrated, incomplete, conflicting, unsupported.
- Selected only.

Filter behavior includes:

- Multi-select values within a filter.
- Removable active-filter chips.
- Result counts.
- Select visible.
- Deselect visible.
- Clear all selections.
- Reset filters without clearing selections.

Each source row exposes its origin, inheritance health, vendor, material, and compatibility summary without requiring the raw JSON to be opened.

### 7.3 Destination panel

The right panel contains:

- Enabled official Bambu target printers discovered from installed manifests.
- Per-printer expansion.
- Per-nozzle checkboxes.
- Extruder-variant information.
- Select-all-nozzles and clear controls per printer.
- A **Manage enabled printers** cogwheel that opens a Cancel-first native dialog with draft state.
- Installed printer artwork when a validated image is available under the approved Bambu catalog root, with a local glyph fallback.
- Warnings for target templates that cannot be resolved.

Only enabled official printers can appear in plan requests. Saving printer management atomically applies the enabled-printer and nozzle defaults; canceling discards the draft. Every enabled printer retains at least one supported nozzle, newly discovered printers remain disabled, and disabling a printer removes its nozzle selection and invalidates a stale plan.

The printer list is vertically scrollable within a viewport-aware bounded destination region. Printer count cannot make the application body grow unpredictably or scroll horizontally at desktop, medium, narrow, or 320 px widths.

Selecting a printer does not silently select every nozzle. The final nozzle selection is always visible in the plan.

#### 7.3.1 Target-template dependencies

Plan building returns a typed `needs_resolution` result when an expected target template is absent or ambiguous. It aggregates every affected source and target rather than parsing backend error text or stopping at the first problem. A native dialog headed **Target profile required** names the expected template, material, official printer, nozzle, diagnostic, and every affected selected filament.

Resolution options are limited to server-validated candidates from current catalogs:

- Use an installed Bambu profile whose resolved material and official printer/nozzle compatibility match.
- Include and migrate a validated Bambu user source that can safely provide the required target-effective profile.
- Remove every affected filament from the selection.

The app never fabricates a missing target, silently substitutes a vendor profile, or accepts arbitrary frontend-supplied profile names or source IDs. Cancel leaves setup unchanged and produces no plan. A submitted decision is revalidated against the current opaque catalogs, then the real backend planner is retried. The frozen plan records and fingerprints the accepted dependency so later source or destination changes fail before staging.

### 7.4 Naming panel

Normal slicing presets and AMS identities use separate naming templates with live previews.

Default templates:

```text
Bambu slicing preset: {clean_name} - {printer_code}
AMS identity:          {vendor} {material} {clean_name}
```

Supported placeholders:

```text
{source_name}
{clean_name}
{vendor}
{material}
{variant}
{source_app}
{source_kind}
{printer}
{printer_code}
{nozzle}
```

`clean_name` removes duplicate vendor and material fragments without changing the source profile. Every generated result remains editable in the plan.

Advanced naming is collapsed by default and supports ordered rules with:

- Simple wildcard matching.
- Regular expressions and capture groups.
- Case-sensitive or case-insensitive matching.
- Replacement text.
- Conditions by source app, source kind, vendor, material, or profile family.
- Apply-to-all and apply-to-matching-only behavior.
- Live before/after previews.
- Saved reusable naming presets.

Invalid placeholders, regexes, filenames, or duplicate results block migration.

### 7.5 Plan panel

The bottom panel shows one row per source/output operation with:

- Source profile.
- Destination slicing preset.
- Destination AMS identity.
- Printer and nozzle targets.
- Proposed action: create, add targets, update, rename, replace, skip, or block.
- Conflict reason.
- Inline destination-name override.
- Validation state.

The plan is the exact operation set executed after the user selects **Migrate**. It is regenerated whenever source, target, naming, or conflict decisions change.

### 7.6 Run and result

When migration starts, the application:

1. Freezes the approved plan.
2. Checks that the source and destination have not changed since planning.
3. Requests a graceful Bambu Studio close when it is running.
4. Stops if Bambu presents an unsaved-work prompt or remains running.
5. Stages all files outside the live profile directory.
6. Validates staged output.
7. Creates a timestamped ZIP backup.
8. Commits the local files transactionally.
9. Launches Bambu Studio when requested.
10. Monitors characterized local evidence through `cloud_id_assigned` at most.
11. Offers an operator checklist to record slicing-list and AMS visibility without automating the UI.
12. Shows per-item outcomes and evidence levels, then writes a receipt.

There is no generic success toast. Completion reports exact counts for local writes, skips, conflicts, rollback results, Bambu-load acknowledgements, cloud-ID assignments, and operator-verified AMS identities.

## 8. Migration semantics

| Source | Default slicing-preset action | Default custom-filament action |
| --- | --- | --- |
| Orca factory/system | Create resolved Bambu user preset per selected printer family | Create selected printer/nozzle targets |
| Orca user/custom | Create resolved Bambu user preset per selected printer family | Create selected printer/nozzle targets |
| Bambu user/custom | Reuse existing source preset | Create only missing selected targets |
| Existing equivalent destination | Skip | Skip or add missing targets |
| Same name, different effective settings | Conflict | Conflict |

Users may disable slicing-preset or custom-filament output in an advanced per-run section, but the defaults above preserve a valid Bambu source for every generated AMS identity.

## 9. Backend architecture

```text
Orca adapter  ──┐
                ├─> canonical profile ─> planner ─> staged operations ─> Bambu writer
Bambu adapter ──┘
```

### 9.1 Rust modules

| Module | Responsibility |
| --- | --- |
| `discovery` | Locate installations, versions, manifests, account roots, and running processes. |
| `profiles` | Parse profile JSON, metadata, manifests, and source origin. |
| `resolver` | Resolve inheritance/includes and retain value provenance. |
| `targets` | Discover printers, nozzles, and extruder variants. |
| `naming` | Render templates and apply wildcard/regex transformations. |
| `planner` | Produce deterministic operations and conflicts without writing. |
| `writer` | Generate slicing presets and flattened custom-filament profiles. |
| `transaction` | Stage, validate, back up, commit, restore, and checksum. |
| `sync` | Launch Bambu Studio and observe setting-ID synchronization. |
| `platform` | Isolate Windows, macOS, and Linux paths and process behavior. |
| `receipt` | Serialize run inputs, outputs, checksums, and verification results. |

The modules are implementation boundaries, not separate services or processes.

### 9.2 Tauri boundary

The Svelte frontend invokes narrowly scoped typed Rust commands. The webview does not receive unrestricted filesystem or process permissions.

Frontend commands operate on opaque IDs and typed data transfer objects for:

- Discovery status.
- Source summaries and filter facets.
- Target summaries and opaque installed-artwork availability.
- Raw printer artwork bytes requested only by current opaque catalog and official printer IDs.
- Naming previews.
- Ready plans or typed target-template dependency issues and decisions.
- Conflict decisions.
- Run progress and receipts.

All paths are validated again in Rust. Frontend-supplied paths are never trusted directly.

### 9.3 Local application state

No database is used in v1. Small versioned JSON files store:

- User-selected slicer paths.
- Saved naming/filter presets.
- Application preferences.
- Workspace preferences schema version 2, including completion of first-run setup, enabled official printer IDs, and default supported nozzles.
- Migration receipts.

Receipts and backups have explicit retention controls. The application never deletes them silently.

## 10. Profile resolution

### 10.1 Inheritance graph

The resolver constructs a directed graph from preset names and includes. It must:

- Resolve factory and user parents across their valid roots.
- Detect missing parents.
- Detect cycles.
- Reject duplicate ambiguous parent names.
- Apply child overrides in slicer-compatible order.
- Preserve scalar and vector types.
- Record which file supplied each effective value.

A filename suffix such as `- H2C` is never treated as compatibility evidence. Compatibility comes from resolved settings and target templates.

### 10.2 Canonical profile

The canonical model contains at least:

- Stable source identifier.
- Source application and kind.
- Source path and metadata path.
- Display name and raw name.
- Vendor.
- Material type.
- Variant/serial.
- Effective settings with typed values.
- Inheritance and include provenance.
- Compatible source printers.
- Source filament and setting IDs when present.
- Parse warnings and blocking errors.

Unknown settings are preserved as typed pass-through values only when the adapter explicitly declares that behavior safe. Unknown structural fields block writing rather than being discarded silently.

### 10.3 Field policy and source eligibility

Each adapter ships a versioned field-policy table. Every recognized field is classified as exactly one of:

```text
source_material
target_machine
mapped
derived
metadata
reject
```

No field crosses from OrcaSlicer to Bambu Studio through an unclassified pass-through path. A policy entry defines source type, destination type, vector behavior, null/`nil` handling, allowed target variants, and any conversion rule.

Resolver behavior must match characterized slicer behavior for:

- Root precedence and user/system shadowing.
- Include order.
- Inheritance override order.
- `nil`, null, missing, and empty-vector semantics.
- `compatible_printers_condition` and `compatible_prints_condition` evaluation.
- Template and instantiation eligibility.

Only instantiable filament presets are selectable migration sources. Abstract bases and include fragments may participate in resolution but cannot be selected directly.

Unknown fields are preserved only for same-application round trips when a reviewed adapter policy marks them safe. Unknown Orca fields block Bambu output until they receive a reviewed mapping policy. The UI lists every blocking field and its source path.

## 11. Target and custom-filament generation

For every selected printer/nozzle target, the writer:

1. Resolves the installed Bambu target template.
2. Applies only fields classified by the target adapter as `source_material`, `mapped`, or `derived`; source-machine and metadata fields never cross implicitly.
3. Retains fields classified as `target_machine` from the effective target template.
4. Normalizes each mapped vector through the field-specific adapter policy for the target's actual extruder variants.
5. Generates a normal Bambu user slicing preset when required.
6. Generates a flattened custom-filament profile with empty inheritance.
7. Assigns the selected printer/nozzle compatibility.
8. Assigns the shared custom `filament_id` through the versioned Bambu adapter.
9. Generates the profile JSON and its version-specific paired `.info` sidecar. The adapter defines the exact destination subdirectory, filename pairing, JSON fields, sidecar encoding, and initial values for `sync_info`, `user_id`, `setting_id`, `base_id`, and `updated_time`. Unsupported or uncharacterized combinations block writing.

Vector normalization never assumes two extruders or copies the first value into every variant. Target defaults are retained where the source does not define an equivalent variant, as required for Standard, High Flow, E3D, and future profiles. Every normalization rule must have a captured slicer result and golden fixture; generic resize logic is not an accepted substitute.

Target-template lookup is based on normalized material and a selected official printer/nozzle target. An exact valid `Generic {material} @BBL {printer_code}` remains automatic. When that name is absent, compatible installed Bambu system profiles and explicitly selected validated Bambu user sources may be presented as typed choices, but no fallback is applied silently. Candidate validation resolves `filament_type`, checks official target compatibility, applies the complete field policy, and retains catalog provenance. Decisions are cached only within one plan build and are revalidated before execution.

Installed printer artwork follows the same opaque-catalog boundary. The backend locates fixed-name image candidates beneath canonical approved Bambu manifest/vendor roots, rejects symlinks and escapes, retains paths only in server state, and exposes only an availability flag plus validated raw image bytes. Installed artwork is not copied into public or release assets.

## 12. Identity and synchronization

### 12.1 Filament identity

A custom filament has one shared `filament_id` across its printer/nozzle profiles. The ID algorithm is adapter-owned and versioned because Bambu may change it.

A generated ID may be reused only when the existing destination has the same normalized identity tuple, compatible target role, and identity-defining fingerprint. Display-name equality alone never resolves an ID collision.

Each adapter declares whether `filament_id` is name-derived. For name-derived adapters, rename previews whether the operation preserves the existing identity, creates a new identity, or is unsupported. Updates preserve existing IDs. Replacements never silently re-key an identity.

The application checks generated IDs against all existing destination identities. Any collision not proven equivalent by the adapter is a blocking conflict.

### 12.2 Sidecar and synchronization evidence

The application never fabricates `PFUS...` setting IDs. Pre-sync metadata must exactly match a captured official-UI fixture for the target adapter; a blank setting ID alone is not a sufficient metadata contract.

The versioned adapter defines the paired `.info` sidecar byte contract, including encoding, line endings, field order when significant, and valid initial values for:

```text
sync_info
user_id
setting_id
base_id
updated_time
```

Valid `sync_info` transitions such as empty, create, update, delete, and hold are characterized separately. Unsupported transitions block writing.

Verification uses distinct evidence levels:

- `created_local`: committed and parse-validated by the migrator.
- `loaded_by_bambu`: Bambu Studio rewrote or acknowledged the profile through a characterized local artifact.
- `cloud_id_assigned`: a unique expected-prefix setting ID appeared in the correct sidecar.
- `ams_verified`: an operator confirmed that the identity is selectable for the intended printer/nozzle and remains present after restart or normal resynchronization.

Automated filesystem monitoring may report at most `cloud_id_assigned`. It never infers `ams_verified` from metadata. A synchronization timeout reports `created_local_unsynchronized` or the highest observed evidence level and does not automatically delete locally valid profiles. The result offers retry or explicit journal-based restore.

### 12.3 Offline behavior

Planning and local generation work offline. Offline output guarantees only locally generated, parse-validated profile data. Cloud assignment, cross-device persistence, AMS availability, and survival of later cloud reconciliation remain pending until Bambu Studio completes normal authenticated synchronization and the appropriate evidence levels are recorded. The plan and commit confirmation state this limitation explicitly.

## 13. Conflict rules

Conflicts are determined using names, canonical identities, targets, and effective-setting fingerprints.

| Condition | Default action |
| --- | --- |
| Same name, same effective settings, same targets | Skip as already migrated. |
| Same custom identity, missing selected targets | Add only missing targets. |
| Same filename, different content | Block as conflict. |
| Same display name, different filament ID | Block as identity conflict. |
| Same generated ID without matching normalized identity tuple and fingerprint | Block as identity collision, even when display names match. |
| Rename would change a name-derived identity | Preview new identity or block; never silently re-key. |
| Case-only filename collision | Block on every platform. |
| Invalid or reserved filename | Block. |
| Existing unsynchronized metadata | Preserve and report; do not overwrite silently. |

Available explicit conflict decisions are rename, update compatible targets, replace, or skip. Replace is never the default and remains recoverable from the run backup.

## 14. Transaction and recovery

### 14.1 Preflight

Before writing, the backend verifies:

- Source and target files still match their planned hashes.
- The selected destination account remains eligible and matches the planned compatibility-matrix row.
- Bambu Studio is stopped.
- Destination paths remain inside the selected user root.
- Every parent/template resolves.
- Every transformed field has a reviewed adapter policy.
- Every JSON/`.info` pair matches a characterized output contract.
- Every output parses against the target adapter.
- Every filename and identity is unique or proven equivalent by normalized tuple and fingerprint.
- Every journal precondition matches the current destination.
- Sufficient disk space exists for staging, backup, and per-path rollback bytes.

### 14.2 Backup and owned-file journal

A timestamped ZIP contains the complete destination filament directory before the run. The archive is checksummed and referenced by the receipt as disaster-recovery evidence, not as a whole-directory transaction mechanism.

The receipt also contains a per-path journal for every planned create, update, or delete:

- Canonical destination path.
- Operation and migrator ownership marker.
- Precondition hash and pre-run bytes, or an explicit record that the path was absent.
- Intended bytes and hash.
- Committed hash.
- Rollback status and any external-change conflict.

### 14.3 Commit

Files are written to a staging directory first. Each destination write uses a same-directory temporary file and atomic replacement where the platform supports it. The application makes no claim of whole-directory atomicity across Windows, macOS, and Linux.

The commit checks each path's precondition immediately before changing it. A mismatch stops the transaction before that path is touched and invokes owned-file rollback for prior committed operations.

### 14.4 Rollback and restore

Automatic rollback touches only paths written or deleted by the current run. It proceeds only when each path's current hash still matches the journal's committed hash. Any path changed externally after commit is preserved and reported for manual recovery.

Rollback restores prior bytes or removes a newly created path according to the journal. It never blindly restores the complete account directory.

Manual restore requires Bambu Studio to be stopped and previews every affected path. Unrelated profiles and cloud changes created after the backup remain untouched. The full ZIP is available for deliberate disaster recovery but is not applied automatically.

## 15. Process handling

- OrcaSlicer may remain open because its files are read-only during migration.
- Bambu Studio must be closed before destination writes.
- The application requests a normal graceful close.
- It never force-kills Bambu Studio.
- If a save prompt appears or the process remains open, migration pauses without writing.
- After commit, the application can launch the detected Bambu Studio executable and monitor synchronization.
- A launch failure leaves the locally committed files and reports the exact executable/path error.

## 16. Platform and distribution architecture

### 16.1 Technology

- Tauri 2 desktop shell.
- Rust backend.
- Svelte 5 + TypeScript frontend.
- Vite static frontend build.
- Operating-system webview rather than an embedded browser runtime.

### 16.2 Release artifacts

Distribution targets are conditional on matching slicer availability and real acceptance in the compatibility matrix. Initial release engineering targets Windows x64 plus only the macOS architecture and Linux distribution forms for which both slicers and the migrator have passed installation, discovery, write, launch, and operator acceptance.

Candidate artifact forms are:

- Windows x64 installer; portable executable only where the required system WebView and acceptance evidence exist.
- macOS DMG for each separately accepted architecture.
- Linux AppImage for each separately accepted architecture/distribution baseline.

Intel macOS, additional Linux distributions, Flatpak, portable Windows builds, and other architectures are not promised until separately characterized and accepted. Installers handle required runtime setup where the platform permits. The project never describes one binary as universally portable across operating systems.

### 16.3 Signing

The release pipeline supports Windows signing and macOS signing/notarization when credentials are available. Unsigned builds are labeled accurately and are not promoted as signed production artifacts.

## 17. Frontend requirements

The interface must feel like a maintained desktop product rather than a raw configuration editor.

- **Spool Ledger** is the dominant product name; **Bambu Filament Migrator** remains a permanent descriptor in the lockup, desktop title, repository documentation, and accessibility text.
- Approved supplied logo assets are used exactly rather than redrawn or approximated in CSS or SVG.
- Bambu green identifies destinations, selections, primary actions, and verified states; Orca orange is limited to source provenance.
- The dark workspace uses the approved green-black palette, and workspace navigation stays on stable theme surfaces without purple interpolation.
- System, light, and dark themes.
- Keyboard-accessible controls and visible focus states.
- Native dialogs open with Cancel focused, support Escape and backdrop dismissal, and restore focus to the invoking control.
- Source-selection checkboxes have legible rest, hover, checked, focus-visible, and disabled states.
- Help controls show one legible question mark rather than a second circled icon.
- Workspace changes use one restrained transition treatment and honor reduced-motion preferences.
- No status communicated through color alone.
- Responsive desktop layout with independently scrollable source, destination, and plan regions.
- Large profile catalogs remain usable through virtualized rows or equivalent measured rendering.
- Destructive actions use explicit labels and confirmation.
- Errors name the affected profile/path and explain the next action.
- Raw JSON is available for diagnostics but never required for normal use.
- Progress reflects discovery, planning, backup, write, launch, and synchronization separately.
- Local inventory progress uses real loaded-of-total values and the approved theme-specific identity assets; it never fabricates counts.

`DESIGN.md` defines the detailed visual contract, and production asset hashes preserve the approved identity. Optional local design references are not required for builds or tests. Existing production behavior, schemas, adapters, and safety requirements remain authoritative when a prototype interaction conflicts with this scope.

## 18. Security and privacy

- All profile processing is local.
- No telemetry in v1.
- No Bambu credentials, cookies, tokens, or cloud payloads are read or stored.
- No arbitrary shell command interface is exposed to the frontend.
- Tauri capabilities grant only the minimum window/plugin permissions.
- Filesystem and process operations stay in Rust commands.
- Manual paths are canonicalized and constrained to approved roots before use.
- Symlinks and path traversal cannot escape staging, destination, backup, or receipt roots.
- Imported strings are treated as data, not commands or paths.
- Receipts omit credentials and unrelated personal data.

## 19. Error and outcome model

Every source/output records a local operation state and, when applicable, a separate Bambu acceptance evidence level:

```text
planned
created_local
updated_local
skipped_existing
skipped_by_user
blocked_invalid_source
blocked_missing_parent
blocked_unsupported_schema
blocked_conflict
rolled_back
created_local_unsynchronized
loaded_by_bambu
cloud_id_assigned
ams_verified
```

`cloud_id_assigned` and `ams_verified` are never synonyms. Only an operator acceptance record can set `ams_verified`.

A batch may complete partially only when remaining failures occurred before their write operations and no shared transaction invariant was broken. Transaction-level failures roll back only journal-owned paths whose hashes still match the run's committed state.

The final summary includes exact counts by local operation and evidence level, and links each failure to its source profile and diagnostic.

## 20. Testing and verification

### 20.1 Rust tests

- Manifest and metadata parsing.
- Factory and user inheritance, root precedence, and shadowing.
- Includes and override order.
- Missing-parent and cycle detection.
- `nil`, null, empty-vector, and compatibility-condition semantics.
- Field-policy completeness and rejection of unclassified cross-application fields.
- Typed scalar/vector preservation.
- Printer/nozzle discovery.
- Extruder-vector normalization against captured slicer behavior.
- Naming placeholders.
- Wildcard and regex replacement.
- Cross-platform filename validation.
- Effective-setting and identity-defining fingerprints.
- Identity generation, rename behavior, and collision detection.
- Plan determinism.
- Per-path transaction commit, externally changed path detection, and owned-file rollback.
- Receipt and journal round trips.
- Byte-level pre-sync/post-sync `.info` sidecars for create, update, rename, delete, and hold states.
- Synchronization and acceptance evidence transitions.

### 20.2 Fixture policy

Tests use synthetic or redistribution-safe fixtures representing supported Bambu and Orca schemas. The project does not publish users' profiles or copy proprietary profile packs into fixtures.

Every supported adapter/matrix row has golden inputs and expected canonical/output representations. Structural fixtures captured from official UI behavior cover a normal user preset, new custom identity, additional printer/nozzle target, rename, update, delete, pre-sync metadata, and post-sync metadata. Published fixtures are sanitized or reconstructed so they remain redistribution-safe while retaining exact structure and byte-level sidecar behavior.

### 20.3 Frontend tests

- Version 1-to-2 workspace migration, fresh and returning first-run behavior, and official-only enabled-printer sanitization.
- First-run source/default validation and atomic completion.
- Filter combinations and persistent selection.
- Printer/nozzle selection, destination manager cancel/save behavior, and bounded scrolling.
- Installed printer artwork load, cleanup, rejection, and glyph fallback.
- Naming preview and invalid-rule blocking.
- Typed target-template dependency choices, removal of all affected sources, cancellation, retry payloads, and no execution before a ready plan.
- Conflict-decision updates.
- Plan rendering.
- Keyboard navigation, dialog focus restoration, and critical accessibility checks.
- Progress and partial-failure states.
- Light/dark visual fixtures for initial inventory, first-run setup, ready workspace, printer manager, target dependency, and artwork fallback at desktop and 320 px widths.

### 20.4 Integration tests

Temporary fake slicer roots verify discovery, planning, writing, backup, rollback, and receipts without touching real user data. Fault-injection cases cover interruption before commit, partial commit, externally changed owned paths, and an unrelated profile created after migration; rollback must preserve every unrelated or externally changed file.

### 20.5 CI

GitHub Actions runs:

- Rust formatting, linting, unit, and integration tests.
- Frontend formatting, typecheck, linting, and tests.
- Tauri build smoke tests on Windows, macOS, and Linux.
- Artifact checksum generation.
- License and dependency checks.

### 20.6 Real acceptance fixtures

The frozen Windows x64 characterization baseline begins with the proven migration of all 16 Polymaker Panchroma families to H2C source/custom profiles across 0.2, 0.4, 0.6, and 0.8 mm nozzles. Phase 0 records the exact application versions, schema signatures, artifact hashes, pre/post-sync sidecars, and operator evidence that profiles are visible in the slicing list and AMS custom-filament surface after restart and one normal resynchronization.

Every additional platform/architecture matrix row must complete a non-destructive source discovery, dry run, backup, local write, Bambu load observation, cloud-ID observation where authentication is available, restart/resynchronization, and operator AMS acceptance before its release artifact leaves draft status.

## 21. Acceptance criteria

v1 is acceptable only when all of the following are demonstrated:

### AC-001 - Source coverage

The application discovers and classifies Orca factory presets, Orca user presets, and Bambu user presets from supported installations.

### AC-002 - Effective settings

For every supported adapter fixture, the resolver and writer produce a field-by-field typed diff matching Bambu Studio's own characterized effective result, including inheritance order, includes, `nil`/null behavior, compatibility conditions, field-policy decisions, and target-vector normalization. Equality with only the migrator's canonical model is insufficient.

### AC-003 - Filtering

A catalog can be narrowed simultaneously by source app, source kind, manufacturer, material type, and text search without losing hidden selections.

### AC-004 - Naming

Default templates, custom placeholders, wildcard rules, regex captures, and inline overrides produce the exact previewed filenames and identity names.

### AC-005 - Target selection

The user can independently select printers and nozzle diameters. Only selected combinations are generated.

### AC-006 - Official-printer discovery

A newly installed official Bambu printer profile can appear without an application code change when its manifest and schema match a supported adapter.

### AC-007 - Official-target boundary

Only official printers discovered from installed Bambu manifests are presented as selectable targets. User-created/custom printer presets are not exposed by the v1 interface.

### AC-008 - No Bambu creation clicks

A batch can create normal presets and AMS custom identities without opening Bambu Studio's custom-filament creation dialog.

### AC-009 - Safety

A failed local write or load validation rolls back only journal-owned paths whose hashes still match the committed run. An unrelated profile created after migration and any externally changed path remain untouched. The result reports both rollback successes and paths withheld for manual recovery.

### AC-010 - Conflicts

No existing different profile is overwritten without a previewed explicit replace decision.

### AC-011 - Synchronization truth

The application reports `created_local`, `loaded_by_bambu`, `cloud_id_assigned`, and `ams_verified` as separate evidence levels. Automated monitoring never promotes a profile to `ams_verified`. Duplicate IDs, blank IDs, timeouts, and later cloud reconciliation are reported explicitly.

### AC-012 - Cross-platform builds

Every claimed release platform/architecture has a compatibility-matrix row proving slicer installation discovery, dry run, safe write, Bambu launch/load observation, restart/resynchronization, and operator AMS acceptance. A CI build artifact alone does not establish platform support.

### AC-013 - Privacy

A complete migration requires no project-operated network service and stores no Bambu credentials.

### AC-014 - Panchroma outcome

On the frozen Windows x64 baseline, one run reproduces the 16 Panchroma slicing presets, 16 custom identities, and 64 selected H2C nozzle profiles without the Bambu creation dialog. The acceptance record confirms all source presets in the slicing list, all identities selectable in the intended AMS/custom-filament surface, all nozzle targets present, 64 unique cloud IDs, and persistence after Bambu Studio restart plus one normal resynchronization.

### AC-015 - Characterized artifact contract

For every supported Bambu adapter, redistribution-safe official-UI fixtures define exact JSON and `.info` artifacts for create, additional target, update, rename, delete, pre-sync, and post-sync states. Directly generated equivalents pass byte/semantic comparisons and a real Bambu load plus operator acceptance run before the adapter permits writes.

### AC-016 - Destination eligibility and offline truth

Only an explicitly selected eligible Bambu account root can receive writes. Empty, stale, default/local, backup, and unrecognized roots are non-writable by default. Before an offline commit, the UI states that cloud persistence and AMS availability remain unverified until normal Bambu synchronization and operator acceptance occur.

### AC-017 - First-run defaults

After real catalogs load, a fresh or version 1 workspace presents first-run setup exactly once. Completion requires a valid source application/kind combination, at least one enabled official printer, and at least one supported nozzle per enabled printer. It persists schema version 2 atomically, selects no source filaments, and never exposes a custom printer.

### AC-018 - Enabled destination management

The user can reopen **Manage enabled printers**, cancel without state changes, or atomically save enabled official printers and supported nozzle defaults. Only enabled official printers reach plan requests. Newly discovered printers remain disabled, and the destination list stays vertically bounded without body-level horizontal overflow at 320 px and wider supported layouts.

### AC-019 - Installed artwork boundary

Available installed printer artwork is read only from canonical approved Bambu catalog roots through current opaque catalog and official printer IDs. Paths never reach the frontend, symlinks and escapes are rejected, artwork failures use an accessible local fallback, and proprietary installed artwork is not redistributed in application assets.

### AC-020 - Target-template resolution

A missing or ambiguous target template yields a typed **Target profile required** result naming every affected filament. Only current server-validated compatible installed Bambu profiles or validated Bambu user sources are selectable. Cancel produces no plan; removal deselects every affected source; no target is fabricated or substituted silently; accepted decisions are fingerprinted in the frozen plan and revalidated before staging.

### AC-021 - Spool Ledger identity

Spool Ledger is dominant across the desktop, repository, onboarding, inventory, and documentation while Bambu Filament Migrator remains a permanent descriptor. The supplied identity assets are byte-identical to the approved handoff, themes preserve their semantic green/orange roles, and the 17-state light/dark visual matrix passes without console errors or horizontal body overflow at 320 px.

## 22. Delivery phases

### Phase 0 - Format characterization and feasibility gate

For each initially supported Bambu Studio matrix row, capture redistribution-safe structural fixtures produced through the official UI for:

- A normal user filament preset.
- A new custom-filament identity.
- An additional printer/nozzle target.
- Rename.
- Update.
- Delete.
- Pre-sync state.
- Post-sync state.

Record every created, modified, and deleted file and field, including exact paired `.info` sidecars and observed sync transitions.

Using only a temporary copy of an account root, reproduce those artifacts directly. Then require an operator acceptance run proving that Bambu Studio loads them, synchronizes them through its normal authenticated client, exposes them in the slicing and intended AMS/custom-filament surfaces, and retains them after restart plus one normal resynchronization.

Exit gate: the direct-file path is demonstrated without UI automation or project-issued cloud requests on the frozen Windows x64 baseline. Until this gate passes, implementation planning is limited to adapters, fixture capture, compatibility matrices, and characterization tooling.

### Phase 1 - Core profile engine

- Repository/tooling baseline.
- Synthetic fixtures.
- Orca and Bambu discovery/parsing.
- Canonical model.
- Inheritance/include resolver.
- Target discovery.
- Naming and planning engine.

Exit gate: deterministic dry-run plans pass on synthetic fixtures and the Panchroma acceptance source set.

### Phase 2 - Safe writer

- Bambu slicing-preset writer.
- Flattened custom-filament writer.
- Identity generation.
- Staging, backup, transaction, rollback, and receipt.

Exit gate: temporary-root integration tests cover success and injected failures; no real account is required.

### Phase 3 - Tauri workspace

- First-run source and official-printer defaults.
- Discovery bar.
- Source filters/list.
- Enabled destination printer/nozzle selector and manager.
- Safe installed printer artwork.
- Naming preview.
- Typed target-template dependency resolution.
- Plan/conflict table.
- Progress and result views.
- Spool Ledger identity and complete light/dark visual fixtures.

Exit gate: the complete workflow runs against temporary fixture roots through the UI, the visual matrix covers all 17 required states in both themes, and a 320 px viewport has no body-level horizontal overflow.

### Phase 4 - Real Bambu verification

- Graceful process handling.
- Launch behavior.
- Synchronization monitor.
- Windows Panchroma acceptance migration.
- macOS real-installation acceptance.
- Linux remains deferred until issue demand establishes a release baseline.

Exit gate: AC-001 through AC-021 have captured evidence for every claimed compatibility-matrix row.

### Phase 5 - Public release

- AGPL-3.0 license and notices.
- README and contributor documentation.
- Security and issue templates.
- Protected-tag Windows and universal macOS release workflow.
- SHA-256 manifests and GitHub provenance attestations.
- Unsigned artifact disclosure; Apple Developer ID and Windows Authenticode signing remain deferred.
- Machine-readable compatibility gate and draft release acceptance.
- Completed-run Buy Me a Coffee link opened through one scoped system-browser permission.
- Final pre-release step: upload `docs/marketing/videos/spool-ledger-promo-v2.mp4` as a GitHub user attachment and replace the README's linked poster fallback with the generated bare `https://github.com/user-attachments/assets/...` URL so GitHub renders the playable video with audio.

Exit gate: public v1 artifacts and documentation match verified platform support without unsupported claims.

## 23. Public-project boundaries

- Public product name: **Spool Ledger**.
- Permanent descriptor: **Bambu Filament Migrator**.
- Repository slug and compatibility identifier: `bambu-filament-migrator`.
- GitHub owner: `Remindz`.
- License: AGPL-3.0.
- The README must state that the project is independent and is not affiliated with or endorsed by Bambu Lab or the OrcaSlicer project.
- Bambu Lab, Bambu Studio, AMS, and OrcaSlicer names remain the property of their respective owners.
- Contributions that add schema support require fixtures and compatibility evidence.
- Compatibility claims are tied to tested adapters and release evidence, not assumptions.

## 24. Scope-change rule

Any addition that introduces remote services, credentials, profile editing, hardware control, automatic downloads, or direct cloud API access requires a new reviewed scope revision before implementation.
