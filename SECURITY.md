# Security Policy

## Supported versions

Security fixes are applied to the current development branch until the first tagged release. After release, the latest minor version will receive fixes.

## Reporting a vulnerability

Use private vulnerability reporting on the project repository when it is available. If the repository does not yet expose that feature, contact the repository owner through their GitHub profile and request a private reporting channel.

Do not open a public issue containing:

- A live slicer profile or Bambu account directory.
- Account identifiers, cloud setting IDs, receipts, backup archives, or local paths that identify a person.
- A proof of concept that can overwrite data outside a temporary fixture.

Include the affected version or commit, operating system, reproduction steps using synthetic data, expected behavior, observed behavior, and the smallest relevant logs. Acknowledgement and remediation timing depend on severity and reproducibility.

## Security boundaries

Bambu Filament Migrator is designed to:

- Read profile data locally and write only beneath a selected eligible Bambu account.
- Reject caller-supplied filesystem paths at the command boundary.
- Freeze source and destination preconditions before execution.
- Stop Bambu Studio before local commit.
- Stage, parse, back up, journal, and receipt every owned write.
- Preserve externally changed files during Restore.
- Avoid Bambu credentials, UI automation, undocumented cloud APIs, telemetry, and fabricated cloud IDs.

A defect that can modify a live slicer profile root, escape the approved account path, overwrite a destination without an explicit safe transition, expose private profile data, or misreport synchronization evidence is security-sensitive.

## Safe testing

Reproduce filesystem issues only with `fixtures/synthetic`, temporary directories, or a private copied root. Never point automated tests at a live OrcaSlicer or Bambu Studio profile directory. Before and after copied-root acceptance, hash the live roots and verify that they are unchanged.
