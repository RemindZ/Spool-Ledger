# Migration receipts and recovery

Every committed migration keeps two independent forms of recovery evidence in its run directory:

- `filament-before-run.zip` is a checksummed snapshot of the complete destination `filament` directory before the run. It is disaster-recovery evidence only. The application never applies it as an automatic whole-folder restore.
- `journal.json` records each path owned by the run, its pre-run state, intended bytes and hash, committed state, and rollback result.

`receipt.json` summarizes the run, references both files, and records exact create, update, delete, commit, rollback, and external-conflict counts.

## Automatic rollback

Automatic rollback walks only entries owned by the current run. A path is restored or removed only when its current state still equals the committed state recorded in the journal. If Bambu Studio, cloud synchronization, or another process changed that path after commit, the migrator preserves the external version and marks an `external_change` conflict.

New unrelated files are never removed. A newly created path is removed only when its hash still matches the migrator's committed bytes. Updated and deleted paths are restored from the per-entry pre-run bytes, not by unpacking the ZIP.

## Manual restore

Manual restore first produces a per-path preview. Bambu Studio must be stopped before applying safe entries. Paths marked as externally changed remain untouched and require an explicit operator decision. The ZIP may be extracted manually for disaster recovery, but doing so can overwrite profiles or cloud changes created after the migration and is therefore never an automatic application action.
