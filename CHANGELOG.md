# Changelog

All notable changes to Spool Ledger (Bambu Filament Migrator) are recorded here. Versions follow [Semantic Versioning](https://semver.org/).

## [1.0.0] - 2026-09-29

First stable release for Windows x64.

### Added

- Update notice. At launch, Spool Ledger reads the public GitHub release list once. When a newer release exists, you can open its release page in your browser, dismiss the notice until the next launch, or skip that version. Nothing is downloaded or installed automatically.
- Run status card on **Run & evidence**. It states the outcome first and places **Retry synchronization** and **Open journal-owned restore** beside it.
- Per-field **Variable** menus in the naming templates, and an output line under each template that shows the name it will produce.
- Filter chips for source application, manufacturer, material, and status, with a **More filters** menu for origin, family, variant, and compatible printer. Each chip shows how many values are selected.
- `AGENTS.md` review rules for Codex pull request reviews.

### Changed

- The workspace chrome is one window bar and one toolbar. Panels carry step numbers, and the setup intro card is gone, which gives the workspace more room.
- The theme control moved into the appearance menu in the window bar.
- The 11 execution steps collapse into one progress bar with **Show steps**.
- The backup hash, receipt path, and plan ID moved into a collapsible **Run details** section.
- The support card appears only after a successful run.
- Skipped and blocked operations on **Run & evidence** use their own icons and colors instead of the success style.

### Fixed

- Migrations from OrcaSlicer 2.4 no longer stop with "unclassified fields". The 35 OrcaSlicer-only filament settings that Bambu Studio does not have are now classified and left out of generated profiles. An inactive OrcaSlicer chamber temperature is never turned into an active Bambu chamber target.
- All text and controls meet WCAG AA contrast in light and dark themes, and focus rings are clearly visible.
- The printer artwork no longer covers the printer name, and the drive mode count uses correct grammar.
- Execution step names are no longer cut off.
- The destination account menu names the empty state instead of appearing blank.
- The migration plan no longer scrolls the whole window sideways on narrow windows.

## [0.9.0] - 2026-09-12

First public beta.

### Added

- Select OrcaSlicer or Bambu Studio factory and custom presets, plus installed official Bambu printers and nozzles.
- Review a frozen migration plan before creating local profiles.
- Back up affected files and keep transaction journals and receipts.
- Track local creation, Bambu loading, cloud ID assignment, and operator AMS verification separately.
- Restore only transaction-owned files whose current hashes still match the recorded run.
- Unsigned Windows x64 packages and an unsigned, unnotarized macOS alpha preview.

[1.0.0]: https://github.com/RemindZ/Spool-Ledger/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/RemindZ/Spool-Ledger/releases/tag/v0.9.0
