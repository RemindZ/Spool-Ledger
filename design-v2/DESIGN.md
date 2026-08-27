# Spool Ledger Design Handoff

## Product

Spool Ledger is a local-first desktop utility for moving OrcaSlicer and Bambu Studio filament presets into Bambu Studio slicing presets and AMS custom-filament identities. **Bambu Filament Migrator** is the permanent product descriptor and search-facing subtext. The utility replaces a repetitive sequence of Bambu dialogs with one inspectable migration plan.

The audience is a technically comfortable 3D-printing user who may manage dozens of filament families across several printers and nozzle sizes. The interface must feel like a precise slicer workstation, not a generic web dashboard or setup wizard.

This is an independent community project. Do not imply affiliation with, sponsorship by, or endorsement from Bambu Lab or OrcaSlicer.

## Design goal

Make source-to-destination routing visible at a glance while keeping a dense workspace calm enough for daily use. The defining visual is the Spool Ledger system: two spool arcs contain a neutral profile-data tile, an orange filament tail identifies provenance, and a green registration notch identifies verified destinations.

The UI may be visually redesigned, but it must preserve the real workflow and backend truth described below. Never add a control, count, status, cloud claim, or editable setting unless the implementation genuinely supports it.

## Visual direction

### Palette

Use these current tokens as the starting contract. Refine their relationships if needed, but preserve the semantic ownership of each hue.

| Role | Light | Dark | Meaning |
| --- | --- | --- | --- |
| Ground | `#E7ECE9` | `#111714` | Slicer workspace |
| Surface | `#F8FAF9` | `#18201C` | Panels and controls |
| Raised surface | `#FFFFFF` | `#202923` | Active or floating layers |
| Primary text | `#17211C` | `#E8EEEA` | Main content |
| Secondary text | `#526159` | `#A8B7AF` | Supporting content |
| Divider | `#CBD5CF` | `#324139` | Structure |
| Bambu green | `#009A3C` | `#00B548` | Destination, commit, verified local action |
| Orca orange | `#E86F2D` | `#F08A4E` | Source provenance only |
| Danger | `#BD3434` | `#EF7676` | Blocking conflicts and failures |
| Warning | `#A95E00` | `#E3A24C` | Unverified custom targets and pending checks |

Bambu green is the action and destination color. Orca orange is deliberately restrained and should never compete with the primary action. Semantic danger and warning colors remain independent of both brands.

### Typography

Use native desktop typography so the Tauri application feels installed rather than web-hosted:

- Display and headings: `Segoe UI Variable Display`, `Segoe UI`, system sans-serif.
- Body and controls: `Segoe UI Variable Text`, `Segoe UI`, system sans-serif.
- IDs, versions, hashes, nozzle values, and counts: `Cascadia Mono`, `SFMono-Regular`, `Consolas`, monospace.

Prefer compact, highly legible sizes. Use tabular numerals for counts and nozzle sizes. Avoid oversized marketing typography.

### Shape and elevation

- Small radius: 5 px.
- Standard radius: 8 px.
- Large radius: 12 px.
- Use borders and tonal separation before shadows.
- Use one restrained shadow tier for genuinely raised content.
- Do not turn every section into a rounded card.

### Signature element

The memorable element is the Spool Ledger construction used as a structural language rather than repeated decoration. It should visually connect:

1. Source profiles through a short Orca-orange filament tail.
2. Naming and policy transformation through graphite profile tiles and ledger rows.
3. Printer, nozzle, and AMS outputs through compact Bambu-green registration notches.

The motif is structural, not decorative. Rounded-square profile tiles, open spool arcs, and registration notches should help users understand where selected material is going and where a conflict interrupts the path. It must collapse cleanly on narrow layouts and respect reduced-motion preferences.

## Information architecture

### Persistent top bar

Show:

- Product name.
- Detected OrcaSlicer and Bambu Studio state.
- Selected eligible Bambu account.
- Light, dark, and system theme control.
- A compact safety indicator stating that plans are local and reviewed before writes.

Do not expose unrestricted file paths as editable inputs. Discovery and account selection operate through backend-approved opaque IDs.

### Main routing bench

Desktop uses three rails:

```text
┌────────────────────┬──────────────────────────┬────────────────────┐
│ Source profiles    │ Naming and transformation│ Bambu destinations │
│ filters + table    │ templates + rules         │ printers + nozzles │
│ Orca provenance    │ live before/after preview │ AMS target status  │
└────────────────────┴──────────────────────────┴────────────────────┘
                      Spool Ledger registration route
```

Below the rails, a full-width migration plan shows every output operation. Execution evidence and results replace or extend this lower area without causing the upper workspace to jump.

At medium widths, source remains full-width and transformation/destination share a second row. At narrow widths, use one ordered column: source, transformation, targets, plan, execution. Never introduce horizontal page scrolling. Tables may scroll inside their own containers.

## Real workflow

The interface must preserve this order:

1. Discover installed slicers and account roots.
2. Select one eligible Bambu destination account.
3. Catalog source profiles.
4. Filter and select source profiles.
5. Select official or explicitly revealed custom printers.
6. Select nozzle diameters independently per printer.
7. Configure separate slicing-preset and AMS-identity names.
8. Review a deterministic plan.
9. Resolve supported conflicts explicitly.
10. Commit locally through staging, backup, journal, and transactional writes.
11. Launch Bambu Studio when requested and monitor local synchronization evidence.
12. Let the operator confirm slicing-list and AMS visibility.
13. Offer journal-owned restore preview and restore.

A plan becomes stale whenever source selection, target selection, naming, rules, row overrides, or conflict decisions change. Rebuild it automatically when a plan is already visible.

## Source rail

Support these real facets:

- Source application.
- Factory/system versus user/custom origin.
- Manufacturer.
- Material.
- Family.
- Variant.
- Compatible source printer.
- Migration status.
- Selected-only state.
- Text search.

Selection must survive filtering. Show visible-selection counts separately from total-selection counts. Abstract inheritance bases and include fragments are never selectable.

During the initial source catalog, keep the final routing bench dimensions stable and show a centered determinate loading state. It combines a restrained circular loading indicator with a progress bar and an exact **loaded of total** portable-preset count supplied by the catalog operation. The source, naming, and destination rails remain hidden and inert until cataloging completes. Announce completion once without narrating every progress increment.

Use Orca orange only for source provenance markers, route entry points, and small source accents.

## Destination rail

- Official printers come from installed manifests.
- Custom printers remain hidden until **Show custom printers** is enabled.
- Every custom printer is marked **Unverified custom printer** in both color and text.
- Nozzle selection is independent per printer.
- Include a real select-all-nozzles action per printer.
- Unsupported nozzle or printer combinations are disabled with a reason.

Use Bambu green for selected, supported destination targets.

## Naming and transformation rail

Keep slicing and AMS naming visibly separate:

- Bambu slicing preset template.
- AMS custom-filament identity template.
- Before and after preview for both.
- Ordered wildcard and regular-expression rules.
- Optional source conditions.
- Reusable naming presets.
- Exact per-row overrides from the plan.

Advanced rules should remain collapsed by default. Invalid regular expressions must block planning with a specific error. Do not hide transformed filenames or generated identities.

## Migration plan

Each row shows:

- Source profile.
- Destination slicing-preset name.
- AMS identity name and local `P.......` filament ID.
- Printer and nozzle target.
- Proposed action.
- Conflict reason.
- Inline name overrides.
- Explicit conflict decision where applicable.

Supported visible actions are:

- Create.
- Add target.
- Update.
- Rename.
- Replace.
- Skip.
- Blocked.

Safety rules:

- Block is the default for conflicts.
- Skip is explicit and non-destructive.
- Update is allowed only for the same identity and target with changed material settings.
- Rename and replace stay visibly blocked when the active adapter lacks a characterized safe transition.
- Replace is never the default.
- No existing different profile is overwritten without a reviewed explicit decision.

Do not use color alone. Every action needs text and an icon or shape.

## Execution and evidence

Local execution phases should be visually distinct and stable:

- Validating frozen plan.
- Closing Bambu Studio gracefully.
- Staging.
- Validating output.
- Creating ZIP backup.
- Committing files.
- Writing journal and receipt.
- Launching Bambu Studio.
- Monitoring synchronization.
- Finished or stopped.

Never show a generic success toast. Report exact counts and per-item outcomes.

Evidence levels are separate and must never be collapsed:

1. `created_local`: committed and parse-validated locally.
2. `loaded_by_bambu`: Bambu Studio acknowledged or rewrote the characterized local artifact.
3. `cloud_id_assigned`: a unique `PFUS...` setting ID appeared in the expected sidecar.
4. `ams_verified`: the operator explicitly confirmed the intended AMS/custom-filament surface after normal synchronization.

A cloud ID does not mean AMS verification. Automated monitoring may report at most `cloud_id_assigned`.

During active synchronization, show a real cancel-monitoring control tied to the committed run ID. A timeout preserves the highest evidence reached and offers retry or restore. No force-kill action exists.

## Recovery

The result area must expose:

- Run ID.
- Exact local committed-file count.
- Backup checksum and file count.
- Journal-owned restore preview.
- Per-path safe or externally changed state.
- Restore action only for paths still matching the run's committed hashes.

Do not suggest unpacking the full ZIP over an account. The ZIP is disaster-recovery evidence, while normal restore is journal-owned and path-specific.

## Content style

- Plain verbs and sentence case.
- Controls state the action: **Build migration plan**, **Commit migration**, **Retry synchronization**, **Preview restore**.
- Errors explain what changed and what the user can do next.
- Avoid hype, filler, emojis, fake reassurance, and em dashes.
- Use “Bambu slicing preset” and “AMS custom filament” consistently.
- State offline limits before commit: local generation does not guarantee cloud persistence or AMS visibility.

## Accessibility and motion

- Full keyboard operation.
- Visible focus rings on every interactive element.
- Minimum WCAG AA contrast.
- Text and icons supplement semantic color.
- Every input has a persistent label.
- Tables retain meaningful headers.
- Status updates use an appropriate live region without announcing noisy polling.
- Respect `prefers-reduced-motion`.
- No auto-scroll, auto-rotate, or motion that fights manual navigation.
- Preserve a 320 px minimum viewport without horizontal body scrolling.

## Required controls and backing functionality

Every control below is required unless marked contextual. Each one must connect to the named state change or backend command. Do not render placeholder variants.

### Global and discovery controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Theme | Three-option segmented control: System, Light, Dark | Changes the document theme immediately and persists locally. |
| Retry discovery | Button, shown after discovery failure | Re-runs installation, manifest, account, and executable discovery. |
| Destination account | Select/menu of discovered accounts | Allows only backend-classified eligible accounts to become write targets. Ineligible accounts remain visible with a reason but cannot be selected. |
| Source roots | Backend-discovered source labels, not editable path fields | Catalogs selected approved Orca/Bambu system and user roots through opaque IDs. |

### Source filtering and selection controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Search | Text input | Filters names, vendor, material, family, and variant without losing hidden selections. |
| Source application | Multi-select facet | Filters OrcaSlicer and Bambu Studio sources. |
| Source kind | Multi-select facet | Filters factory/system and user/custom sources. |
| Manufacturer | Multi-select facet | Filters canonical vendor values from the catalog. |
| Material | Multi-select facet | Filters PLA, PETG, ABS, and other discovered values. |
| Family | Multi-select facet | Filters discovered filament families. |
| Variant | Multi-select facet | Filters discovered variants. |
| Compatible printer | Multi-select facet | Filters source compatibility metadata. |
| Migration status | Multi-select facet | Filters new, already migrated, incomplete, conflicting, and unsupported states. |
| Selected only | Checkbox/toggle | Shows only currently selected source profiles. |
| Source row selection | Checkbox per selectable profile | Adds or removes the source ID while preserving selection through filtering. Abstract profiles never receive this control. |
| Select visible | Button/checkbox action | Selects only rows currently passing all filters. |
| Clear selection | Button | Clears all selected source IDs. |
| Active-filter chips | Removable chips | Remove one active filter without resetting unrelated filters. |
| Filter preset name | Text input | Names a reusable filter preset. |
| Save/load/delete filter preset | Buttons/menu | Persists and restores real filter state locally. |

### Printer and nozzle controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Show custom printers | Toggle, off by default | Re-catalogs targets with custom printers visible. Every custom target remains marked unverified. |
| Nozzle diameter | Checkbox per supported printer/nozzle | Selects each diameter independently. Unsupported combinations are disabled with a reason. |
| Select all nozzles | Button per printer | Selects or clears all supported nozzle diameters for that printer only. |

### Naming controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Bambu slicing preset template | Text input | Updates the slicing name template and live preview. Rebuilds an existing plan. |
| AMS identity template | Text input | Updates the custom-filament identity template and live preview. Rebuilds an existing plan. |
| Advanced naming rules | Disclosure/accordion | Reveals real ordered wildcard/regex rule editing without changing behavior merely by opening. |
| Rule output | Select: slicing preset or AMS identity | Chooses which naming pipeline receives the rule. |
| Rule kind | Select: wildcard or regular expression | Changes the parser used by the backend preview/planner. |
| Rule pattern | Text input | Stores the matching pattern. Invalid regex blocks planning and shows the parser error. |
| Rule replacement | Text input | Supports wildcard replacement and regex capture references. |
| Case sensitive | Checkbox | Controls matching case for that rule. |
| Optional source condition | Field select plus value input | Restricts a rule by source application, source kind, vendor, material, or family. |
| Add/remove/reorder rule | Buttons | Updates actual ordered rule arrays and regenerates preview/plan. |
| Naming preset name | Text input | Names a reusable combination of both templates and both rule lists. |
| Save/load/delete naming preset | Buttons/menu | Persists or restores the real templates and rule lists. |
| Before/after preview | Read-only paired values | Displays backend-rendered slicing and AMS names, never locally fabricated output. |

### Per-run output controls

These belong in a collapsed **Advanced outputs** section and default on.

| Control | UI form | Required behavior |
| --- | --- | --- |
| Create Bambu slicing presets | Toggle/checkbox | Includes or excludes normal slicing-preset JSON artifacts from the frozen plan and transaction. |
| Create AMS custom filaments | Toggle/checkbox | Includes or excludes flattened custom JSON plus paired `.info` artifacts from the frozen plan and transaction. |

At least one output must remain enabled. Changing either output invalidates and rebuilds an existing plan. The plan and result counts must reflect only enabled artifact kinds.

### Plan-row controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Slicing preset name | Text input per operation | Creates an exact source/printer/nozzle override and rebuilds the plan. |
| AMS identity name | Text input per operation | Creates an exact AMS override, recomputes the local identity when appropriate, and rebuilds the plan. |
| Conflict decision | Select shown for blocked or explicitly resolved rows | Sends Rename, Update, Replace, Skip, or no decision for the exact source/printer/nozzle tuple. Backend safety rules remain authoritative. |
| Build migration plan | Primary button | Sends selected opaque IDs, nozzles, templates, rules, overrides, output choices, and conflict decisions to the backend. Disabled until required inputs exist. |
| Commit migration | Primary destructive-boundary button | Executes only the exact frozen non-blocked plan ID after the offline/synchronization limitation is visible. |

Rename and Replace may remain selected but blocked when the active adapter cannot execute them safely. The UI must show the backend reason and must not visually imply the choice succeeded.

### Execution, synchronization, and recovery controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Local execution phase | Read-only live status | Displays validating, graceful close, staging, backup, commit, and finished events from the Tauri channel. |
| Cancel monitoring | Button during synchronization only | Cancels the active synchronization token for the committed run ID. It does not cancel or undo the completed local transaction. |
| Retry synchronization | Button after timeout/failure | Reuses the same committed run ID and journal-owned sidecars. |
| Record AMS verification | Button after cloud assignment | Opens or performs an explicit operator checklist action for selected cloud-assigned operations only. |
| Preview restore | Button after a committed run | Loads journal-owned paths and indicates safe versus externally changed paths. |
| Restore owned paths | Confirmed button | Restores only paths still matching the run's committed hashes. Never unpacks the full ZIP over the account. |

### Controls that must not be designed

- Credential, login, token, or cloud API fields.
- A force-close or force-kill control.
- An automatic “Verify AMS” control.
- An editable cloud ID.
- An unrestricted filesystem path input.
- A whole-account restore button.
- Numeric sliders with no continuous numeric setting in the product model.
- Decorative toggles that do not change the frozen backend plan.

## State coverage required in designs

Provide desktop light and dark designs for:

1. Initial source catalog with determinate loaded-of-total progress.
2. First discovery with no eligible account.
3. Ready workspace with selected sources and nozzles.
4. Advanced naming rules open with live preview.
5. Plan with create, add-target, skip, and blocked rows.
6. Conflict decision selected but still unsafe.
7. Local execution in staging/backup/commit phases.
8. Synchronization monitoring.
9. Timeout with retry and restore choices.
10. Cloud IDs assigned but AMS checks pending.
11. Operator-confirmed AMS verification.
12. Restore preview containing both safe paths and externally changed paths.
13. Narrow responsive layout.

## Non-negotiable product truth

Do not design any of the following as though they exist:

- Bambu UI automation.
- Direct undocumented Bambu cloud API access.
- Credential entry or storage.
- Fabricated `PFUS...` IDs.
- Automatic AMS verification.
- Force-killing Bambu Studio.
- Writes to an unapproved or ineligible account.
- Whole-account automatic ZIP restore.
- Supported rename or replace transitions that the active adapter blocks.
- macOS or Linux release claims without platform acceptance evidence.

## Design acceptance checklist

- The interface reads as one material-routing workstation, not a collection of dashboard cards.
- Source, transformation, destination, plan, execution, and recovery remain understandable without documentation.
- Bambu green and Orca orange retain distinct semantic roles in both themes.
- Dense tables remain legible and keyboard-operable.
- No state causes large layout shifts.
- Every visible control maps to a real implemented callback or command.
- Every status is supported by backend evidence.
- Cloud assignment and AMS verification remain visibly separate.
- Custom printers are hidden by default and marked unverified when shown.
- Offline and synchronization limits appear before the irreversible commit.
- The design remains coherent at desktop, medium, and 320 px widths.
