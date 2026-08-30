# Compatibility evidence

Bambu Filament Migrator enables writes only for compatibility-matrix rows backed by characterized profile artifacts and acceptance evidence.

| Matrix row | Bambu Studio | OrcaSlicer | Platform | Schema evidence | Write status | AMS evidence |
| --- | --- | --- | --- | --- | --- | --- |
| `windows-x64-bambu-02.08.02.60-orca-2.4.2` | 02.08.02.60 | 2.4.2 | Windows 11 x64 | Synthetic fixtures derived from a successful 16-family Panchroma migration | Characterized baseline | Operator confirmed Satin and final collection; formal app-run acceptance remains Phase 0 work |
| `macos-universal` | Pending | Pending | macOS 12+ universal | Baseline document created; real installation not yet characterized | Release blocked | Pending |

`release-matrix.json` is the machine-readable release gate. A row remains disabled until its evidence document satisfies the local-write, Bambu-load, restart/resynchronization, and operator AMS requirements in `SCOPE.md`. Automated compilation alone never enables a row.

Application and profile schema versions are tracked separately. Adding a row requires sanitized fixtures, hashes, and the acceptance levels defined in `SCOPE.md`.

Real characterization files belong under `.local-characterization/` and are gitignored. Only fictional, redistribution-safe structural fixtures are committed.
