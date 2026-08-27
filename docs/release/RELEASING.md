# Releasing Spool Ledger

A release is a two-stage operation: automation builds and verifies a draft, then the owner reviews and publishes it. The workflow never publishes automatically.

## 1. Clear compatibility gates

- Complete Windows built-app dogfood, restart/resynchronization, and operator AMS acceptance.
- Complete every item in `docs/compatibility/macos-universal-baseline.md`.
- Remove each platform row's `pending` field and set `release_enabled` to `true` only after its evidence document is complete.
- Keep Linux absent from the release matrix.

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
npm run release:contract -- --tag v0.1.0
npm audit
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
```

Replace `v0.1.0` with the prepared version. A disabled platform, missing evidence file, unexpected platform, or version mismatch must fail before release work begins.

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
