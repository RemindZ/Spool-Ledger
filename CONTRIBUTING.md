# Contributing

Bambu Filament Migrator accepts focused fixes and characterized profile support. Every contribution must preserve the local-data safety contract in [SCOPE.md](SCOPE.md).

## Ground rules

- Work on a branch. Do not push unreviewed changes directly to `main`.
- Use test-driven development for behavior changes. Run the failing reproduction before implementing the fix.
- Use only synthetic fixtures, temporary account roots, or private ignored copies. Never run tests against a live slicer profile root.
- Do not commit credentials, account identifiers, cloud setting IDs from a real account, receipts, backup archives, or copied user profiles.
- Do not add Bambu UI automation, credential handling, an undocumented cloud API, or fabricated `PFUS` IDs.
- Do not claim platform support without an acceptance record from that platform.
- Keep unknown cross-application fields fail-closed. A field-policy change needs source provenance, an explicit class, and a regression fixture.
- Keep the application independent and avoid third-party trademarks or artwork except where names are needed for compatibility descriptions.

## Setup

Install Node.js 24 and Rust 1.88, then run:

```bash
npm ci
CARGO_BUILD_JOBS=1 cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm test -- --run
```

## Required verification

Before submitting a change, run:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
CARGO_BUILD_JOBS=1 cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
CARGO_BUILD_JOBS=1 cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm run release:check
npm run format:check
npm run lint
npm test -- --run
npm run check
npm run build
npm run tauri build -- --no-bundle
```

A change is not ready when any command fails. Include the exact failing output if an environment prevents a gate from running.

## Profile fixtures

Public fixtures must be synthetic or reconstructed. Preserve the structural behavior needed by the test, but replace personal names, account IDs, cloud IDs, machine names, and nonessential values. Document fixture provenance in [NOTICE.md](NOTICE.md).

Private copied-root characterization belongs under `.local-characterization/`, which is ignored by Git. Before using it, record live-root hashes. After the run, verify those hashes again and stage only code, synthetic fixtures, and documentation.

## Release automation

Pull requests run only on GitHub-hosted infrastructure. The self-hosted MacBook is reserved for protected release tags and must never be targeted by contribution workflows.

Changes to `.github/`, release scripts, `docs/compatibility/`, `docs/release/`, or `src-tauri/tauri.conf.json` require owner review. A build artifact does not establish platform support; only an enabled `docs/compatibility/release-matrix.json` row backed by completed evidence may enter a draft release.

See [docs/release/RELEASING.md](docs/release/RELEASING.md) for the owner-only release ritual. Contributors must not create or move release tags.

## Pull requests

Explain the user-visible outcome, the safety boundary affected, the RED reproduction, the GREEN verification, and any platform evidence. Keep unrelated refactors out of the change. Contributions are licensed under AGPL-3.0.
