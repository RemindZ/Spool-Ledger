# Windows x64 baseline

## Frozen environment

- Date characterized: 2026-08-19
- Operating system: Windows 11 Pro x64
- Bambu Studio application: `02.08.02.60`
- Bambu system-manifest version: `02.08.00.04`
- OrcaSlicer application: `2.4.2`
- Orca Bambu-manifest version: `02.01.00.19`
- Destination family exercised: Bambu Lab H2C
- Nozzles exercised: 0.2, 0.4, 0.6, and 0.8 mm

Versions were read from installed executable/configuration metadata and manifests. Profile schema versions remain separate from application versions.

## The manual workflow that proved direct generation

This record preserves the successful working method without publishing the user's profile files or account identifier.

1. Inspect Orca's user filament root and system profile manifest.
2. Identify each thin user preset and resolve its missing Orca-only parent chain.
3. Confirm the equivalent parent does not exist in Bambu Studio. Merely setting `compatible_printers` to an empty list does not flatten inheritance.
4. Resolve each source material's explicit base and machine overrides.
5. Create a Bambu source preset inheriting a real installed target template, with explicit H2C printer/nozzle compatibility and supported extruder variants.
6. Place one Satin source preset in a copied/live-controlled destination as a canary while Bambu Studio is stopped.
7. Open Bambu Studio and confirm the source appears in the normal filament list.
8. Use Bambu's UI once for Satin to capture the exact flattened custom-profile shape for all four H2C nozzles.
9. Observe that the four flattened profiles share one `filament_id`, have empty inheritance, and use one paired `.info` sidecar per JSON profile.
10. Confirm the current identity algorithm against upstream behavior:

```text
filament_id = "P" + first_7_hex(md5(custom_display_name + "@" + non_secret_user_id))
```

11. Generate a second identity directly with exact target-template structure, per-nozzle names, deterministic shared ID, and characterized pre-sync sidecars.
12. Launch Bambu Studio and observe four unique `PFUS...` setting IDs assigned through Bambu's normal authenticated synchronization.
13. Generate the remaining identities directly and repeat synchronization observation.
14. Verify final filesystem counts and ask the operator to confirm the intended AMS/custom-filament surface.

## Proven outcome

- 16 ordinary Bambu slicing presets.
- 16 custom filament identities.
- 64 flattened H2C printer/nozzle profiles.
- 64 paired `.info` sidecars.
- 64 unique `PFUS...` setting IDs after normal Bambu synchronization.
- Operator confirmed the custom entries were visible in the intended Bambu interface.
- Automated tests compared source and destination bytes during copies.
- Timestamped ZIP backups preceded every live write in the manual proof.

## Important limits

- A `PFUS...` ID proves cloud-ID assignment, not AMS visibility.
- Formal `ams_verified` evidence requires the operator checklist, restart, and one normal resynchronization.
- The migrator must reproduce the behavior through versioned contracts rather than hardcoding Panchroma or H2C.
- Development and automated acceptance never write to the live roots used for this characterization.
- Rename, update, delete, hold, and additional-target contracts are represented by sanitized fixtures and upstream behavior; they require explicit operator acceptance before a release adapter can claim them.

## Public fixture mapping

The committed `Northstar PLA Aurora` fixtures are fictional replacements for the real material names. They preserve the structural properties needed to test:

- Thin Orca inheritance.
- Source material overrides.
- H2C three-variant vector normalization.
- Flattened custom profiles.
- Pre-sync and post-sync sidecars.
- Local identity and cloud setting-ID separation.
