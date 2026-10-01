# Repository Guidelines

Spool Ledger (Bambu Filament Migrator) is a local-first Tauri 2 desktop utility that migrates OrcaSlicer and Bambu Studio filament profiles into Bambu user presets and AMS custom filaments. `CLAUDE.md` is the full project contract; `SCOPE.md` and `DESIGN.md` define product scope and design. This file summarizes what reviewers and agents must check.

## Project Structure

- `src-tauri/src/`: Rust 2024 backend. Filesystem, process, and network boundaries live here.
- `src-tauri/resources/field-policy-v1.json`: fail-closed field classification for cross-application transfer.
- `src-tauri/tests/`: Rust integration tests. Ignored tests read installed profiles and must stay read-only.
- `src/`: Svelte 5 runes frontend with strict TypeScript. `src/lib/components/` holds UI, `src/lib/state.ts` holds reducers.
- `src/lib/__tests__/`: Vitest and Svelte Testing Library tests, including CSS contract tests.
- `tests/visual/`: synthetic visual fixture and reference captures.
- `fixtures/`: synthetic profile fixtures.

## Verification Commands

Run from the repository root:

- `npm test -- --run`, `npm run check`, `npm run lint`, `npm run format:check`, `npm run build`
- `npm run visual:check`, `npm run release:check`
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`

## Code Review Rules

Prioritize data safety and truthful state over style. Flag a change as a blocking issue when it breaks any rule below.

### Live profile safety

- Nothing may write to live OrcaSlicer or Bambu Studio profile roots from tests, fixtures, screenshots, or development paths. Tests use `tempfile` roots, synthetic fixtures, or copied roots with before and after hashes.
- Restore must only touch journal-owned paths whose current hashes still match the committed run. Unpacking a full backup ZIP over an account is never acceptable.
- Execution must stage, validate, back up, journal, and receipt exact operations from a frozen, fingerprinted plan. Flag any path that writes outside that sequence.
- Bambu Studio and Spool Ledger are closed gracefully. Flag force-kill calls and any mouse, keyboard, accessibility, or screen-scraping automation of Bambu Studio.

### Fail-closed profile handling

- Unknown profile fields must block cross-application output. Flag silent pass-through, silent drops, or broad catch-all field policies.
- A new entry in `field-policy-v1.json` needs characterization evidence (upstream option lists or installed profiles) and a resolver or writer test. Mapping a value between applications needs a machine-safety review, for example chamber heating.
- Missing target templates must return the typed **Target profile required** result, never a fabricated template.

### Evidence and claims

- Evidence levels stay separate: `created_local`, `loaded_by_bambu`, `cloud_id_assigned`, `ams_verified`. Automated monitoring reports at most `cloud_id_assigned`; only an operator records `ams_verified`.
- Never fabricate `PFUS...` setting IDs, infer AMS verification from local metadata, call undocumented Bambu cloud APIs, or read credentials, cookies, or tokens.
- UI must not show a control, count, or status that no backend state supports.

### Boundaries

- The frontend sends opaque IDs and typed data. Filesystem paths, process control, and network calls stay in Rust. Flag new webview access to paths or the network.
- The only permitted network request is the public GitHub release list read in `src-tauri/src/updates.rs`. It must stay read-only and unauthenticated, and send no profile data. The update notice may only open a validated `vX.Y.Z` release page; flag any download or install path.
- Supported targets are installed official Bambu printers from manifests. Flag custom-printer support as out of scope for v1.
- Tauri CSP changes need a stated reason. `img-src` must keep `blob:` for printer artwork.

### Tests and UI

- Behavior changes and bug fixes need a regression test that fails without the change. Flag weakened assertions, skipped tests, or broad error swallowing.
- Customer-facing copy uses plain sentence case and active verbs, with no em dashes.
- UI colors come from the theme tokens in `src/app.css`. Text and control pairs must keep WCAG AA contrast in both themes; `src/lib/__tests__/contrast-tokens.test.ts` enforces the token pairs.
- Documentation media uses synthetic data only. Flag personal account IDs, local paths, or real vendor data such as Sunlu or Polymaker in docs or demos.
