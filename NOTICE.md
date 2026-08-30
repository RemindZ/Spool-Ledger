# Notices

Bambu Filament Migrator

Copyright (C) 2026 Matthias (RemindZ)

This program is licensed under the GNU Affero General Public License, version 3. The complete license text is provided in [LICENSE](LICENSE).

## Independent project

Bambu Filament Migrator is an independent compatibility utility. It is not affiliated with or endorsed by Bambu Lab, the OrcaSlicer project, SoftFever, or their contributors. Bambu Lab, Bambu Studio, OrcaSlicer, AMS, and related names and marks belong to their respective owners. Those names are used only to describe compatibility.

The project does not include Bambu credentials, imitate a Bambu service, automate Bambu Studio controls, or call an undocumented Bambu cloud API.

## Fixture provenance

Files under `fixtures/synthetic/` and `fixtures/expected/` are synthetic or reconstructed test data. They preserve profile inheritance, JSON structure, sidecar byte layout, printer/nozzle vectors, and synchronization-state shapes needed for deterministic tests without redistributing a user's profile collection. Public fixtures use synthetic account identifiers and non-secret placeholder cloud IDs where a post-sync structure must be represented.

Private copied-root characterization data lives under `.local-characterization/`. It is excluded from version control and release artifacts. It must never be committed or redistributed.

## Third-party software

Rust and npm dependencies retain their original copyrights and licenses. Dependency versions are recorded in `Cargo.lock` and `package-lock.json`. CI audits Rust advisory, source, and license policy and checks npm advisories before producing the accepted Windows smoke artifact.
