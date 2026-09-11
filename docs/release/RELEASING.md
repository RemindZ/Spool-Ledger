# Releasing Spool Ledger

A release is a two-stage operation: automation builds and verifies a draft, then the owner reviews and publishes it. The workflow never publishes automatically.

## Bootstrap the first macOS acceptance

The release matrix intentionally blocks release jobs until real platform evidence exists. For the first release only:

1. Merge the verified implementation with both matrix rows still disabled under explicit approval.
2. From protected `main`, manually run `.github/workflows/macos-acceptance.yml`.
3. Download its universal DMG and checksum evidence, then complete installed-app and slicer acceptance on the MacBook.
4. Record the evidence and enable the platform rows in a follow-up reviewed commit.

The acceptance workflow never runs on a pull request or push, cannot mutate a release, and does not read or write live slicer profiles. It only verifies source, builds the universal application, checks its metadata and architectures, and uploads short-lived evidence.

## 1. Clear compatibility gates

- Complete Windows built-app dogfood, restart/resynchronization, and operator AMS acceptance.
- Complete every item in `docs/compatibility/macos-universal-baseline.md`.
- Remove each platform row's `pending` field and set `release_enabled` to `true` only after its evidence document is complete.
- Keep Linux absent from the release matrix.

For v0.9.0 only, the owner authorized macOS alpha preview distribution with incomplete detailed human acceptance records. Keep `release_enabled: false` and the pending evidence visible; `alpha_preview_tag: "v0.9.0"` permits distribution only when the contract receives `--prerelease`. This exception does not establish full Mac support, authorize a stable release, or carry forward to later versions. Release notes must identify the Mac alpha and request testing feedback.

## 2. Prepare the version

Set the same semantic version in:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

Run the full local verification pipeline and the prospective tag contract:

```bash
npm ci
npm test -- --run
npm run check
npm run lint
npm run format:check
npm run build
npm run visual:check
npm run release:check
npm run release:contract -- --tag v0.9.0 --prerelease
npm audit
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Replace `v0.9.0` with the prepared version. A disabled platform without an applicable explicit preview exception, missing evidence file, unexpected platform, or version mismatch must fail before release work begins.

## 3. Protect and tag the accepted commit

1. Merge the verified release commit to protected `main` under a separate explicit approval.
2. Confirm the MacBook runner is online with labels `self-hosted`, `macOS`, `ARM64`, and `spool-ledger-release`.
3. Create and push a protected `vX.Y.Z` tag under a separate explicit approval.

The tag starts `.github/workflows/release.yml`. Do not move or recreate a tag after a release has been published.

## 4. Observe automated builds

The workflow runs:

1. Linux-hosted release preflight against the tagged source and compatibility matrix.
2. Full Windows verification and unsigned NSIS, MSI, and portable builds on `windows-latest`.
3. Full macOS verification and an unsigned universal DMG on the registered MacBook.
4. GitHub-hosted checksum verification, provenance attestation, and draft release assembly.

One failed or missing platform blocks the assembler. If the MacBook was offline, bring it online and rerun failed jobs. Re-running the same tag may replace assets only while the release remains a draft.

To start a complete manual rebuild, bind both the workflow ref and input to the same protected tag:

```bash
gh workflow run release.yml --ref vX.Y.Z -f tag=vX.Y.Z
```

## 5. Verify the draft independently

Download every draft asset and `SHA256SUMS.txt`, then verify:

```bash
sha256sum -c SHA256SUMS.txt
gh attestation verify <downloaded-file> -R Remindz/bambu-filament-migrator
```

On Windows, dogfood the NSIS installer, MSI installer, and portable executable. On macOS, dogfood the DMG and verify the application opens through the normal unsigned Gatekeeper approval flow. Confirm the displayed app version and complete a read-only discovery pass.

## 6. Complete public media

Upload `docs/marketing/videos/spool-ledger-promo-v2.mp4` through a GitHub issue or pull-request draft. Copy the generated `https://github.com/user-attachments/assets/...` URL into the root README on its own line so GitHub renders the video player. The draft used for uploading does not need to be submitted.

Run README formatting and local-link checks again after replacing the poster fallback.

## 7. Publish explicitly

Review:

- unsigned warnings;
- supported-platform claims;
- artifact names and sizes;
- checksums and attestations;
- compatibility evidence links;
- generated release notes;
- playable README video.

Publishing the draft is a separate irreversible action and requires explicit approval for that release instance.

## Deferred signing

When credentials exist, add Apple Developer ID signing/notarization and Windows Authenticode as a separately reviewed release change. Do not add empty secrets or describe unsigned files as signed in the meantime.
