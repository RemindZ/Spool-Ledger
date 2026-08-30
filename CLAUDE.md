# Spool Ledger project instructions

## Product identity

- Primary product name: **Spool Ledger**.
- Permanent descriptor: **Bambu Filament Migrator**.
- Repository and compatibility identifier: `bambu-filament-migrator`.
- Spool Ledger is an independent local-first desktop utility. It is not affiliated with or endorsed by Bambu Lab or OrcaSlicer.
- Use the supplied files in `public/` for the product mark. Never redraw or approximate the logo in CSS, SVG, or another asset.

## Sources of truth

Use this order when requirements appear to conflict:

1. Existing production behavior, schemas, adapters, tests, and safety contracts.
2. `SCOPE.md`, including its acceptance criteria and official-target-only v1 boundary.
3. `docs/design/DESIGN.md` and `docs/design/brand-spec.md`.
4. The immutable 14-file design handoff in `design-v2/`.
5. `README.md` and generated documentation media.

The prototype demonstrates presentation and interaction intent. It never authorizes weaker validation, fabricated state, custom-printer support, or a different backend contract.

## Non-negotiable data safety

- **Never write to live OrcaSlicer or Bambu Studio profile roots during development, tests, screenshots, or automated acceptance.**
- Use synthetic fixtures, `tempfile` roots, copied profile roots, and byte-verified before/after hashes.
- Installed-profile tests are read-only. Any generated profile output belongs in a temporary account copy.
- Do not perform a real migration, commit, merge, push, publish, or create a release without Matthias's explicit approval for that instance.
- Never force-kill Bambu Studio or Spool Ledger. Request graceful close and stop if the application remains open.
- Never automate Bambu Studio through mouse, keyboard, accessibility, or screen-scraping actions.
- Never call undocumented Bambu cloud APIs or read credentials, cookies, tokens, or authenticated payloads.
- Never fabricate `PFUS...` cloud setting IDs or infer AMS verification from local metadata.
- Never unpack a complete backup ZIP over an account. Restore only journal-owned paths whose current hashes still match the run.
- Frontend code sends opaque IDs and typed data. Filesystem and process boundaries stay in Rust.

## Supported behavior

- Sources: OrcaSlicer factory/system presets, OrcaSlicer user/custom presets, and Bambu Studio user/custom presets.
- Targets: installed official printers discovered from Bambu manifests. User-created or custom printers remain out of scope for v1.
- First-run setup stores source filters, enabled official printers, and default nozzles in workspace preferences schema v2.
- Missing target templates return a typed **Target profile required** result. The app offers only server-validated installed Bambu profiles or validated Bambu user sources, or removes every affected filament.
- Installed printer artwork is read from canonical approved Bambu catalog roots through opaque catalog and printer IDs. Paths never reach the frontend.
- Plans are frozen and fingerprinted. Execution stages, validates, backs up, journals, commits, and receipts exact operations.
- Evidence levels remain separate:
  - `created_local`
  - `loaded_by_bambu`
  - `cloud_id_assigned`
  - `ams_verified`
- Automated monitoring may report at most `cloud_id_assigned`. Only an operator can record `ams_verified`.

## Architecture

- Desktop shell: Tauri 2.
- Backend: Rust edition 2024 under `src-tauri/src/`.
- Frontend: Svelte 5 runes and strict TypeScript under `src/`.
- Build: Vite 7.
- Frontend tests: Vitest and Svelte Testing Library.
- Rust integration tests: `src-tauri/tests/`.
- Synthetic fixtures: `fixtures/`.
- Visual fixture and captures: `tests/visual/`.
- Documentation screenshots: `docs/screenshots/`.
- Product-tour media and license notes: `docs/demo/`.

Keep each boundary narrow. Reuse existing reducers, components, typed commands, and adapter policies before adding another abstraction or dependency.

## Development method

- Follow RED -> GREEN -> REFACTOR for every bug fix or behavior change.
- Reproduce the real failure and identify its root cause before editing production code.
- Add the smallest regression that fails for the observed reason.
- Preserve unknown-field fail-closed behavior. Add field policies only with characterization evidence and focused tests.
- Do not hide failures with fallback UI, broad error swallowing, skipped assertions, or weakened acceptance criteria.
- Customer-facing copy uses plain sentence case and active verbs. **No em dashes in customer-facing copy.**

## Required verification

Run from the repository root:

```bash
npm test -- --run
npm run check
npm run lint
npm run format:check
npm run build
npm run visual:check
npm run release:check
npm audit
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Read-only and copied-root Windows acceptance:

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test commands installed_elegoo_pet_cf_resolves_bambu_pet_cf_target_without_live_writes -- --ignored --exact --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --test commands installed_orca_sources_catalog_account_over_default_mirrors -- --ignored --exact --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --test commands installed_bambu_profiles_discover_and_catalog_h2c -- --ignored --exact --nocapture
cargo test --manifest-path src-tauri/Cargo.toml --test copied_roots copied_windows_roots_reproduce_all_panchroma_h2c_artifacts_without_live_writes -- --ignored --exact --nocapture
```

Every protected live-root hash must be identical before and after ignored acceptance.

## Windows release build

Windows 11 x64 is the only release-supported platform. macOS and Linux remain unclaimed until each has its own real-installation acceptance matrix.

The current Tauri bundler can lock the shared executable when MSI and NSIS are produced in one invocation. Build each artifact separately and build the standalone executable last:

```bash
npm run tauri build -- --bundles nsis
npm run tauri build -- --bundles msi
npm run tauri build -- --no-bundle
```

Do not use one combined `tauri build` as release evidence while that lock warning is reproducible. Record SHA-256 hashes in `src-tauri/target/release/bundle/SHA256SUMS.txt` and verify them with `sha256sum -c`.

Release requirements that must not regress:

- `src-tauri/src/main.rs` uses release-only `windows_subsystem = "windows"`, so the packaged app has no terminal window.
- Tauri `img-src` CSP includes `blob:`, because installed printer artwork is rendered from generated blob URLs.
- MSI, NSIS, and standalone executable are unsigned unless signing credentials were explicitly supplied. Label them truthfully.

## Documentation and demo media

- README screenshots and video use only synthetic data and actual production components.
- Do not expose personal account IDs, local paths, tokens, or live profile contents.
- By user request, documentation demonstrations must not use Sunlu or Polymaker. Use synthetic vendors such as Fiberlogy and Overture.
- Never stage or invent progress as a claim about a live migration. Label fixture-based screenshots and video as synthetic demonstrations.
- The product tour may use the exact supplied logo and an original low-key audio bed. Keep license/provenance notes beside the media.
- Documentation must distinguish local profile creation, Bambu loading, cloud ID assignment, and human AMS verification.

## Current acceptance boundary

Automated Windows implementation, copied-root safety, package construction, and built-app artwork rendering are verified. Owner workflow dogfood and human cloud/AMS acceptance remain separate gates. Do not represent those human gates as complete until Matthias records them.
