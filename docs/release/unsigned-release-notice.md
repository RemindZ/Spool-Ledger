## Unsigned initial release

Spool Ledger v0.9.0 is the first public beta.

Windows x64 is the beta-supported platform. **macOS is an alpha preview: additional testing is needed, and we welcome your feedback.** The Mac build contains Apple Silicon and Intel executable slices, but exact human-tested environment and artifact records are incomplete. Do not interpret a universal build as verified compatibility on every Mac or macOS 12.

Report installation and workflow issues through GitHub Issues. Include OS, hardware, slicer versions, and app version, but do not attach credentials, account identifiers, or live profile contents.

These Windows and macOS files are built automatically from the tagged source and are not code signed.

### Included artifacts

Replace `X.Y.Z` with the release version shown above.

- `Spool-Ledger-vX.Y.Z-windows-x64-setup.exe` - Windows x64 NSIS installer
- `Spool-Ledger-vX.Y.Z-windows-x64.msi` - Windows x64 MSI installer
- `Spool-Ledger-vX.Y.Z-windows-x64-portable.exe` - Windows x64 portable executable
- `Spool-Ledger-vX.Y.Z-macos-universal.dmg` - universal macOS disk image for Apple Silicon and Intel

Compatibility evidence:

- [Windows x64 baseline](https://github.com/Remindz/bambu-filament-migrator/blob/main/docs/compatibility/windows-x64-baseline.md)
- [Universal macOS baseline](https://github.com/Remindz/bambu-filament-migrator/blob/main/docs/compatibility/macos-universal-baseline.md)

- Windows SmartScreen may warn before opening an installer or portable executable. Verify `SHA256SUMS.txt` and the GitHub provenance attestation before continuing.
- macOS Gatekeeper may require right-clicking the application and choosing **Open**, or approving it in **System Settings > Privacy & Security**. Do not disable Gatekeeper globally.
- Linux is not supported in this release. If you need Linux support, open an issue with your distribution, desktop environment, and Bambu Studio or OrcaSlicer installation method.

Verify a downloaded checksum:

```bash
sha256sum -c SHA256SUMS.txt
```

Verify GitHub build provenance:

```bash
gh attestation verify <downloaded-file> -R Remindz/bambu-filament-migrator
```

A successful automated build proves artifact provenance and build integrity. Platform support remains tied to the compatibility records linked from the repository.
