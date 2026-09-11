# macOS universal compatibility baseline

- **Release status:** v0.9.0 alpha preview only, not fully accepted
- **Architecture:** universal (`arm64` and `x86_64`)
- **Minimum target:** macOS 12.0, pending real-installation confirmation
- **Signing:** unsigned initial-release artifact
- **Bambu Studio:** pending characterization
- **OrcaSlicer:** pending characterization

This row remains disabled for fully accepted releases in `release-matrix.json` until every acceptance item below has recorded evidence. A successful GitHub Actions build is not acceptance.

The owner explicitly authorized distribution as a macOS alpha preview for the v0.9.0 prerelease despite unavailable detailed human acceptance records. `alpha_preview_tag` permits only that exact prerelease, not a stable release or a later version. macOS alpha preview: additional testing is needed, and we welcome your feedback. The universal executable contains both architectures; this does not establish installed acceptance on Intel Macs or on the minimum macOS version.

## Automated evidence

- [x] Rust and frontend verification pass natively on the registered Apple Silicon runner.
- [x] The packaged executable reports both `arm64` and `x86_64` through `lipo -archs`.
- [x] The DMG contains the expected bundle identifier, version, icon, and minimum system version.
- [x] Read-only discovery finds the installed Bambu Studio and OrcaSlicer roots.
- [x] Read-only cataloging resolves installed profile resources without copying proprietary profiles into the repository.
- [x] Copied-root planning, writing, backup, restore, and receipts pass while configured live-root hashes remain unchanged.

Installed-profile and copied-root acceptance recorded 2026-09-05:

- Protected-main commit: `b8c1f70d8f1a90e5c9d0aaedc89a44d071f6b606`
- GitHub Actions run: <https://github.com/RemindZ/bambu-filament-migrator/actions/runs/33960999211>
- Both explicitly selected installed/copy characterization tests passed, followed by dependency audit, universal packaging, and checkout cleanup.
- Downloaded artifact: `Spool-Ledger-v0.1.0-macos-universal.dmg`, 11,975,573 bytes.
- Independently verified SHA-256: `caec47e6e85a2b8fb3cd644342f9b3d626474f3b1fe3aff49107eef363dc27e3`.
- These tests use an inert process backend for copied-root execution. They do not establish real application launch, Gatekeeper acceptance, cloud synchronization, or operator AMS visibility.


Acceptance build evidence recorded 2026-08-30:

- Protected-main commit: `f822785bb5a93c4b1ed510c68c191445fd2c1ea2`
- GitHub Actions run: <https://github.com/RemindZ/bambu-filament-migrator/actions/runs/33321306614>
- Artifact: `Spool-Ledger-v0.1.0-macos-universal.dmg`
- Size: 11,975,615 bytes
- SHA-256: `d91fb285d8bb51c456d3cc61cc18e6887d119614c553d66440050927a2464abc`
- Downloaded `macos-SHA256SUMS.txt` verification: `OK`

The protected workflow also required the bundle identifier `io.github.remindz.bambu-filament-migrator`, app version `0.1.0`, minimum system version `12.0`, a present referenced ICNS file, and both universal architectures before uploading the evidence.

## Human acceptance

During v0.9.0 release preparation, the owner confirmed: "Mac checks already completed". This records the owner's confirmation, not a new automated or independently observed test. The exact human-tested macOS, hardware, filesystem, slicer versions, and artifact identity have not yet been linked to that confirmation. The detailed checklist and release-enable gate remain pending that evidence; the CI artifact above is not assumed to be the human-tested artifact.

- [ ] Bambu Studio receives a graceful close request and exits normally.
- [ ] Spool Ledger launches the installed Bambu Studio application.
- [ ] An explicitly approved migration commits the reviewed local files.
- [ ] Bambu Studio acknowledges each local profile.
- [ ] Unique cloud setting IDs are observed through Bambu Studio's normal authenticated flow.
- [ ] Profiles persist after Bambu Studio restart and one normal resynchronization.
- [ ] The operator confirms intended printer or AMS visibility.

## Release-enable gate

After all items pass, record the exact macOS, hardware, filesystem, Bambu Studio, OrcaSlicer, app artifact, and SHA-256 values here. Only then may `macos-universal.release_enabled` change to `true`.
