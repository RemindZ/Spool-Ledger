# macOS universal compatibility baseline

- **Release status:** blocked
- **Architecture:** universal (`arm64` and `x86_64`)
- **Minimum target:** macOS 12.0, pending real-installation confirmation
- **Signing:** unsigned initial-release artifact
- **Bambu Studio:** pending characterization
- **OrcaSlicer:** pending characterization

This row remains disabled in `release-matrix.json` until every acceptance item below has recorded evidence. A successful GitHub Actions build is not acceptance.

## Automated evidence

- [ ] Rust and frontend verification pass natively on the registered Apple Silicon runner.
- [ ] The packaged executable reports both `arm64` and `x86_64` through `lipo -archs`.
- [ ] The DMG contains the expected bundle identifier, version, icon, and minimum system version.
- [ ] Read-only discovery finds the installed Bambu Studio and OrcaSlicer roots.
- [ ] Read-only cataloging resolves installed profile resources without copying proprietary profiles into the repository.
- [ ] Copied-root planning, writing, backup, restore, and receipts pass while configured live-root hashes remain unchanged.

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
