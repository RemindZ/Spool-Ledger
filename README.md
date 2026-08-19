# Bambu Filament Migrator

A local-first desktop utility for migrating OrcaSlicer and Bambu Studio filament presets into Bambu custom filaments for AMS-compatible printers.

> This project is independent and is not affiliated with or endorsed by Bambu Lab or the OrcaSlicer project.

Implementation is in progress against [SCOPE.md](SCOPE.md). The application will not write to live slicer roots until a reviewed migration plan, backup, and explicit run action exist.

## Development

```bash
npm install
cargo test --manifest-path src-tauri/Cargo.toml
npm run check
npm run build
npm run tauri dev
```

## License

AGPL-3.0. See [LICENSE](LICENSE).
