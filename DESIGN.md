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

The 2026-10-02 update uses a sage light theme and a lifted charcoal dark theme. Dark surfaces are never near-black.

| Role | Light | Dark | Meaning |
| --- | --- | --- | --- |
| Ground | `oklch(91% 0.022 160.07)` | `oklch(28% 0.01 163.06)` | Slicer workspace |
| Surface | `oklch(97% 0.009 165.08)` | `oklch(32% 0.012 163.06)` | Panels and controls |
| Raised surface | `#FFFFFF` | `oklch(36% 0.014 163.06)` | Active or floating layers |
| Primary text | `oklch(27% 0.018 163.06)` | `oklch(90% 0.008 163.06)` | Main content |
| Secondary text | `oklch(43% 0.024 162.2)` | `oklch(74% 0.016 163.06)` | Supporting content |
| Divider | `oklch(80% 0.025 159.87)` | `oklch(43% 0.016 163.06)` | Structure |
| Bambu green | `oklch(59.84% 0.1735 147.93)` | `oklch(70% 0.112 147.93)` | Destination, primary action, verified states, registration square |
| Orca orange | `#E86F2D` | `oklch(74% 0.112 48)` | Source provenance and source selection only |
| Danger | `#BD3434` | `oklch(75% 0.13 25)` | Blocking conflicts and failures |
| Warning | `#A95E00` | `oklch(77% 0.108 75)` | Pending checks and stopped synchronization |

Bambu green is the action and destination color. Orca orange marks source provenance and source selection (picker checkboxes, selection meters and pills) and never competes with the primary action. Semantic danger and warning colors remain independent of both brands.

Chrome surfaces stay dark in both themes: the title bar, the step spines, the open step header and the Run & verify rail header. They resolve the dark token set locally through `.dark-island`, so text, muted text and borders always use dark-theme values on them.

Brand hues are fills and marks. Text and control boundaries use contrast tokens that keep each hue's meaning and meet WCAG AA on every panel surface and on the dark chrome. `src/lib/__tests__/contrast-tokens.test.ts` enforces these pairings for both themes.

| Token | Light | Dark | Use |
| --- | --- | --- | --- |
| `--text-faint` | `oklch(43% 0.024 162.2)` | `oklch(74% 0.016 163.06)` | Kickers, table headers, helper text (4.5:1) |
| `--orca-ink` | `oklch(48% 0.145 47)` | `oklch(78% 0.11 50)` | Orca source text (4.5:1) |
| `--action` / `--on-action` | Bambu green / `#07120C` | Bambu green / `oklch(20% 0.02 163.06)` | Primary action fill and label (4.5:1) |
| `--on-danger` | white | `oklch(20% 0.02 163.06)` | Label on danger fill (4.5:1) |
| `--control-border` | `oklch(58% 0.025 160)` | `oklch(64% 0.02 163.06)` | Input, select, and checkbox boundaries (3:1) |
| `--focus-ring` | `oklch(48% 0.14 148)` | Bambu green | Solid 2 px focus outline (3:1) |

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

### Signature element and textures

The memorable element is the Spool Ledger route rendered as a step bench: three steps that read as one physical object, joined by a route strand that fills orange (filament profiles), amber (naming) and green (printers and nozzles) as the operator advances.

Two supplied textures carry the surface language. Both are generated decorative assets with provenance notes in `public/design/`:

- `filament-header-texture.png`, a photographic filament roll, appears only on the title bar, as a `cover` background at 32 % opacity positioned `center 54%`.
- `texture-companion-infill.png`, a rectilinear FDM infill skin, appears only on the collapsed step spines, the open step header bar and the Run & verify rail header. It tiles at 320 × 320 px and shows at 43 % over `#2E3833`.

The step icons (`step-icon-filament.png`, `step-icon-naming.png`, `step-icon-printer.png`) appear at 46 px on spines and 42 px with a soft drop shadow in open headers. Never redraw the logo, the icons or the textures.

## Information architecture

### Persistent top bar

The chrome is two rows. The work area starts below them, with no intro card or separate stepper. The workspace heading is visually hidden for screen readers.

1. The title bar, which is also the drag region: the Spool Ledger lockup (184 px), the route **OrcaSlicer → Bambu Studio** with an orange source dot and screen-reader detection state, and one theme button at the right end.
2. The toolbar (62 px): the tabs Workspace, Review plan, Run & verify and Restore, the **Destination** account select, and a green-tinted **Local only** chip.

Source folders and discovery refresh live in the Catalog popover of the filament picker footer. The platform is not labelled, because Windows is the only release platform. No sample-workspace chip appears in production.

Do not expose unrestricted file paths as editable inputs. Discovery and account selection operate through backend-approved opaque IDs.

### Main routing bench

The bench is a three-step accordion: **Filament profiles → Naming → Printers & nozzles**. Exactly one step is open at a time and the workspace starts on Filament profiles. The bench height is `clamp(540px, calc(100dvh - 262px), 940px)`.

```text
┌──────────────────────────────────────────────┬────┬────┐
│ 01 Filament profiles (textured header)       │ 02 │ 03 │
│ Source · Manufacturer · Search               │    │    │
│ All materials / PLA / Matte                  │spine│spine│
│ material cards, type cards or preset rows    │    │    │
│ N presets selected · Show selected only ...  │  2 │  2 │
└──────────────────────────────────────────────┴────┴────┘
 Step 1 of 3 · Filament profiles   3 sources 2 targets 2 outputs  [Continue to naming →]
```

Collapsed steps are 84 px textured spines, completed steps on the left and upcoming steps on the right. A spine shows the mono step number (with a 6 px green square once completed), the step icon, the vertical title, and a live count ("3 selected", "2 outputs", "2 targets"). The whole spine is a button that opens its step and moves focus to the step heading.

The action bar below the bench shows "Step N of 3 · Title" with a one-line hint, the source, target and output counts, a readiness tag, **Back** (hidden on step 1) and one primary button: **Continue to naming**, **Continue to printers**, **Continue to review**. The last one builds the frozen plan with the existing command and opens Review plan.

Readiness is checked per step: step 1 needs a preset ("Select a preset"); step 2 needs an output and a valid AMS template ("Enable an output", "Fix the AMS template"); step 3 needs presets, targets, outputs and an eligible account ("Select hardware" and so on). The tag reads "Ready to continue" on steps 1 and 2 and "Ready to build" on step 3.

Below 900 px the accordion stacks vertically and spines become 68 px horizontal bars. Never introduce horizontal page scrolling. Tables may scroll inside their own containers.

### First-run setup

After real source and target catalogs load, first-run setup asks which detected source applications and profile kinds to include, then which official Bambu printers and default nozzles to enable. The choices are validated against the current catalogs and persisted atomically. The normal workspace does not appear behind onboarding, and completing setup does not silently select source filaments.

The destination settings control reopens the same official-printer picker later. Disabled printers are absent from the destination rail and plan requests. Newly discovered printers remain disabled until the operator enables them.

## Real workflow

The interface must preserve this order:

1. Discover installed slicers and account roots.
2. Select one eligible Bambu destination account.
3. Catalog source profiles.
4. Filter and select source profiles.
5. Select official printers discovered from installed Bambu manifests.
6. Select nozzle diameters independently per printer.
7. Configure separate slicing-preset and AMS-identity names.
8. Review a deterministic plan.
9. Resolve supported conflicts explicitly.
10. Commit locally through staging, backup, journal, and transactional writes.
11. Launch Bambu Studio when requested and monitor local synchronization evidence.
12. Let the operator confirm slicing-list and AMS visibility.
13. Offer journal-owned restore preview and restore.

A plan becomes stale whenever source selection, target selection, naming, rules, row overrides, or conflict decisions change. Rebuild it automatically when a plan is already visible.

## Filament profiles step

The picker is a three-tier drilldown.

- **Filters:** a **Source** checklist grouped OrcaSlicer and Bambu Studio (System presets, Your presets, each with a count), a **Manufacturer** checklist with its own search box, and the retained **Compatible printer** and **Migration status** checklists. Every value starts checked. Select all and Clear all act only on the values a menu search shows. A filtered trigger gets a 1 px inner outline.
- **Search presets** searches name, material and type within the current level and shows flat results grouped by "Material · Type".
- **Materials** appear as cards in the order PLA, PETG, PCTG, ABS, ASA, TPU, PA · Nylon, PC, followed by any other material from real data and then Other. Materials without visible presets are hidden. Each card shows the preset and manufacturer counts, an orange selection meter, a selected pill and a tri-state checkbox that selects every visible preset in that material.
- **Tiering** is recomputed from the visible types: three or more types open a type tier, exactly two types are listed as rows on the card, and one type opens its presets directly.
- **Presets** are grouped by manufacturer with per-group Select all or Clear. Each row shows the product name, "OrcaSlicer · System preset" or "Bambu Studio · Your preset", migration status when it is not new, any warnings, and an Orca or Bambu tag.
- The breadcrumb reads "All materials / PLA / Matte" or "ABS · Glass fiber" for materials without a type tier. The up button skips the absent tier. The right side shows "N shown · M selected" and, at the preset level and in search results, "Select all N" or "Clear these".
- The footer shows "N presets selected" with "· K hidden by filters", a **Show selected only** switch, **Clear selection**, **Saved filters**, and the **Catalog** popover with catalog status, Refresh discovery and Source folders. Footer popovers open upward.
- Empty states cover no manufacturers, no sources, no search matches, nothing selected in Show selected only, and no presets for the filters.

The material archetype and type come from the Rust catalog (`material_archetype`, `material_type`). The archetype comes from the resolved `filament_type`; the type comes from preset-name keywords such as Matte, Silk, High speed, Carbon fiber, Glass fiber, Aero, For AMS and Shore hardness. Presets without a keyword stay under their material as Standard, and presets without a material go under Other. Nothing is guessed. Abstract inheritance bases are excluded by the catalog and never selectable.

Drilling in or out slides the level in from ±16 px over 0.26 s, resets the scroll position, moves focus to the current breadcrumb and announces the level politely. Mouse Back and Forward and Alt+Left and Alt+Right retrace visited levels from an internal history, only while the step is open, the Workspace tab is active and no dialog is open. Both mouse presses are cancelled so the WebView never navigates.

Selection survives filtering. During the initial source catalog, keep the final bench dimensions stable and show the centered determinate loading state with an exact **loaded of total** portable-preset count.

## Printers and nozzles step

- Official printers come from installed manifests and only enabled official printers appear, as cards in a `repeat(auto-fill, minmax(280px, 1fr))` grid up to 1280 px wide. A card's border turns green-tinted when any of its nozzles is selected.
- **Manage enabled printers** opens the reusable enabled-printer manager without exposing custom targets.
- Installed printer cover art is requested through current opaque catalog and official-printer IDs. The backend keeps paths private, and the UI falls back to a local printer glyph.
- Nozzle selection is independent per printer, with a real Select all or Clear all action per printer.
- Unsupported nozzle or printer combinations are disabled with a reason.

Use Bambu green for selected, supported destination targets.

## Naming step

The step uses two columns up to 1280 px: the template composers on the left (`1.5fr`) and the Advanced naming rules, Outputs and Naming preset disclosures on the right (`minmax(280px, 1fr)`) behind a 1 px divider. It becomes one column below 1040 px.

Keep slicing and AMS naming visibly separate:

- Bambu slicing preset template.
- AMS custom-filament identity template.
- Before and after preview for both.
- Ordered wildcard and regular-expression rules.
- Optional source conditions.
- Reusable naming presets.
- Exact per-row overrides from the plan.

Each template shows its backend-rendered output directly below it, and the example line names the first selected preset and first selected nozzle. When naming rules change a name, the pre-rule value appears as **Before rules**. The **Variable** palette opens inline under the composer that triggered it, closes on selection, Close or Escape, and keeps the slicing-only scope rules.

Advanced rules should remain collapsed by default. Invalid regular expressions must block planning with a specific error. Do not hide transformed filenames or generated identities.

## Target profile required

Missing or ambiguous target templates return a typed dependency result rather than a generic schema error. The dialog names every affected selected filament and offers only candidates validated against the current material, official printer, nozzle, field policy, and server-held catalogs. A Bambu user-source candidate is offered only when that source can safely provide the target-effective profile and is explicitly included in the migration request.

The app never fabricates a missing template. The operator can resolve every issue with an approved current candidate, remove every affected filament, or cancel and keep setup selections unchanged. Cancel receives initial focus and Escape, backdrop close, focus restoration, and keyboard operation match the shared native dialog contract.

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

Run & verify uses a 300 to 380 px step rail on the left (sticky at 16 px) and, on the right, the run status card, Migration evidence and Run details. It becomes one column below 960 px.

1. The rail header uses the infill texture with "TRANSACTION", a live summary ("No transaction started", "Step 5 of 10 · Create ZIP backup", "Finished · 4.7 s") and a 4 px progress bar.
2. The rail lists ten steps with one-line descriptions. Pending steps show a numbered ring. The active step lifts, reveals its description, and pulses with an orbiting arc. Completed steps fill green, draw a check and show their measured duration. The last step settles to a check when the run finishes.
3. The status card states what happened in plain words with the actions that resolve it (Retry synchronization, Open journal-owned restore). Its headline re-enters with a 6 px rise when it changes.
4. Migration evidence keeps the four evidence levels as separate tiles. A reached level pops its green registration square, tints the tile and fills a 3 px line along its bottom edge. Per-operation rows follow, with skipped and blocked operations in their own treatment.
5. **Run details** keeps created, updated, deleted, skipped, and backup counts on one line. The plan ID, backup hash, and receipt path open on request.

Timing is real. The Rust execution and synchronization commands stamp every phase event with monotonic `elapsed_ms`; step durations and the total come only from those events, never from a UI timer. Only completed steps show a time.

The support card appears only after a resolved run, never beside a timeout, stopped synchronization, or active monitoring.

The ten steps:

1. Validate frozen plan.
2. Close Bambu Studio gracefully.
3. Stage output.
4. Validate output.
5. Create ZIP backup.
6. Commit files.
7. Write journal and receipt.
8. Launch Bambu Studio.
9. Monitor synchronization.
10. Finished or stopped.

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
- Visible focus rings on every interactive element; spine focus rings are inset.
- Minimum WCAG AA contrast in both themes, including muted text on the dark chrome.
- Text and icons supplement semantic color.
- Every input has a persistent label.
- Tables retain meaningful headers.
- Status updates use an appropriate live region without announcing noisy polling.
- 44 px targets on coarse pointers.
- No auto-scroll, auto-rotate, or motion that fights manual navigation.
- Preserve a 320 px minimum viewport without horizontal body scrolling.

| Motion | Duration | Easing |
| --- | --- | --- |
| Accordion grow | 0.44 s | `cubic-bezier(.2,0,0,1)` |
| Step content reveal | 0.36 s after 0.1 s | `cubic-bezier(.2,0,0,1)` |
| Drilldown level slide | 0.26 s | `cubic-bezier(.2,0,0,1)` |
| Theme icon swap | 0.28 s | `cubic-bezier(.2,0,0,1)` |
| Route strand segment | 0.32 s | `cubic-bezier(.2,0,0,1)` |
| Run rail fill and progress | 0.5 s | `cubic-bezier(.2,0,0,1)` |
| Check draw | 0.35 s | `cubic-bezier(.2,0,0,1)` |
| Active node pulse and orbit | 1.4 s and 0.9 s | ease-out and linear |
| Status headline swap | 0.36 s | `cubic-bezier(.2,0,0,1)` |
| Evidence line and mark pop | 0.6 s and 0.4 s | `cubic-bezier(.2,0,0,1)` |

Under `prefers-reduced-motion: reduce` these transitions and animations are disabled. State changes stay visible through color, icon and text.

## Required controls and backing functionality

Every control below is required unless marked contextual. Each one must connect to the named state change or backend command. Do not render placeholder variants.

### Global and discovery controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Theme | One 40 × 40 icon button at the right end of the title bar | Cycles Automatic, then the opposite of what Automatic resolves to, then the remaining theme. Follows OS changes while on Automatic, names the current and next mode, and persists locally. |
| Retry discovery | Button, shown after discovery failure | Re-runs installation, manifest, account, and executable discovery. |
| Destination account | Select/menu of discovered accounts | Allows only backend-classified eligible accounts to become write targets. Ineligible accounts remain visible with a reason but cannot be selected. |
| Source roots | Backend-discovered source labels, not editable path fields | Catalogs selected approved Orca/Bambu system and user roots through opaque IDs. |
| Source folders | Section of the Catalog popover with application/kind selectors and native picker | Requests an operating-system folder picker from Rust, validates the chosen root, and catalogs the returned opaque ID. No path crosses from the webview. |
| Registered source row | Read-only path, validation state, and Remove button | Shows persisted manual roots. Remove unregisters only the root and never deletes source profiles. |

### Source filtering and selection controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Source | Checklist menu grouped by application | Filters the four application and kind pairs. Select all and Clear all apply to all four. |
| Manufacturer | Checklist menu with search | Filters canonical vendor values from the catalog. Select all and Clear all apply to the searched names only. |
| Compatible printer | Checklist menu with search | Filters source compatibility metadata. |
| Migration status | Checklist menu | Filters new, already migrated, incomplete, conflicting, and unsupported states. |
| Search presets | Text input | Searches name, material and type within the current drilldown level without losing hidden selections. |
| Material and type cards | Card grid with tri-state checkbox | Opens the next tier and selects or clears every visible preset in scope. |
| Breadcrumb and up button | Navigation | Moves up one level, skipping an absent type tier. Mouse Back and Forward and Alt+arrows retrace visited levels. |
| Source row selection | Checkbox per selectable profile | Adds or removes the source ID while preserving selection through filtering. Abstract profiles never receive this control. |
| Select all N / Clear these | Button at the preset level and in search results | Selects or clears the visible presets in scope. |
| Show selected only | Switch | Shows only currently selected source profiles. |
| Clear selection | Button | Clears all selected source IDs. |
| Saved filters | Footer popover | Names, saves, loads and deletes the real filter state locally. |
| Catalog | Footer popover | Shows catalog status, refreshes discovery, and manages source folders. |

### Printer and nozzle controls

| Control | UI form | Required behavior |
| --- | --- | --- |
| Nozzle diameter | Checkbox per supported official printer/nozzle | Selects each diameter independently. Unsupported combinations are disabled with a reason. |
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
14. First-run setup with real source counts and enabled official printers.
15. Enabled-printer manager with a bounded printer list.
16. Target profile dependency with all affected filaments and safe resolution choices.
17. Installed printer-artwork fallback.

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
- Only official printers discovered from installed Bambu manifests are presented as targets.
- Offline and synchronization limits appear before the irreversible commit.
- The design remains coherent at desktop, medium, and 320 px widths.
