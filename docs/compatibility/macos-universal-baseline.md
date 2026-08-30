# macOS universal compatibility baseline

- **Release status:** blocked
- **Architecture:** universal (`arm64` and `x86_64`)
- **Minimum target:** macOS 12.0, pending real-installation confirmation
- **Signing:** unsigned initial-release artifact
- **Bambu Studio:** pending characterization
- **OrcaSlicer:** pending characterization

This row remains disabled in `release-matrix.json` until every acceptance item below has recorded evidence. A successful GitHub Actions build is not acceptance.

## Automated evidence

- [x] Rust and frontend verification pass natively on the registered Apple Silicon runner.
- [x] The packaged executable reports both `arm64` and `x86_64` through `lipo -archs`.
- [x] The DMG contains the expected bundle identifier, version, icon, and minimum system version.
- [ ] Read-only discovery finds the installed Bambu Studio and OrcaSlicer roots.
- [ ] Read-only cataloging resolves installed profile resources without copying proprietary profiles into the repository.
- [ ] Copied-root planning, writing, backup, restore, and receipts pass while configured live-root hashes remain unchanged.

Acceptance build evidence recorded 2026-08-30:

- Protected-main commit: `f822785bb5a93c4b1ed510c68c191445fd2c1ea2`
- GitHub Actions run: <https://github.com/RemindZ/bambu-filament-migrator/actions/runs/33321306614>
- Artifact: `Spool-Ledger-v0.1.0-macos-universal.dmg`
- Size: 11,975,615 bytes
- SHA-256: `d91fb285d8bb51c456d3cc61cc18e6887d119614c553d66440050927a2464abc`
- Downloaded `macos-SHA256SUMS.txt` verification: `OK`

The protected workflow also required the bundle identifier `io.github.remindz.bambu-filament-migrator`, app version `0.1.0`, minimum system version `12.0`, a present referenced ICNS file, and both universal architectures before uploading the evidence.

## Human acceptance

- [ ] Bambu Studio receives a graceful close request and exits normally.
- [ ] Spool Ledger launches the installed Bambu Studio application.
- [ ] An explicitly approved migration commits the reviewed local files.
- [ ] Bambu Studio acknowledges each local profile.
- [ ] Unique cloud setting IDs are observed through Bambu Studio's normal authenticated flow.
- [ ] Profiles persist after Bambu Studio restart and one normal resynchronization.
- [ ] The operator confirms intended printer or AMS visibility.

## Release-enable gate

After all items pass, record the exact macOS, hardware, filesystem, Bambu Studio, OrcaSlicer, app artifact, and SHA-256 values here. Only then may `macos-universal.release_enabled` change to `true`.
