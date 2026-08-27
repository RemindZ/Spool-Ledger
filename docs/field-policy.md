# Field policy v1

Every Orca-to-Bambu field must be classified before migration. The bundled table is `src-tauri/resources/field-policy-v1.json`.

| Class | Behavior |
| --- | --- |
| `source_material` | Copy the source material value without importing source-machine identity. |
| `target_machine` | Keep the resolved Bambu target value. |
| `mapped` | Apply a key-specific conversion or target-vector rule. |
| `derived` | Generate from the migration plan, identity, target, or destination contract. |
| `metadata` | Use for resolution/eligibility only; do not copy as material data. |
| `reject` | Block migration with an explicit diagnostic. |

Unknown Orca fields block cross-application output. They are never silently dropped or passed through.

## Characterized fan fields

`first_x_layer_part_fan_speed` and `ironing_fan_speed` are classified as `source_material`. Bambu Studio upstream includes both keys in its filament-preset option list in [`Preset.cpp`](https://github.com/bambulab/BambuStudio/blob/926a7192574bcb9b3a732e1ec59a46d79cb45466/src/libslic3r/Preset.cpp). Resolver and writer regressions verify that both values survive Orca-to-Bambu transfer and participate in the generated material settings.

This evidence applies only to these two reviewed fields. Neighboring or newly discovered keys remain fail-closed until they are characterized independently.

## Current mapped vectors

- `filament_flow_ratio`
- `filament_max_volumetric_speed`
- `nozzle_temperature`
- `nozzle_temperature_initial_layer`

The writer maps source variants to matching target extruder variants and retains target defaults for target-only variants. It does not resize vectors generically.

## Policy changes

A policy change requires:

1. A characterized source and destination fixture.
2. A resolver/writer test showing the expected typed result.
3. A compatibility-matrix update when behavior is version-specific.
4. A review of whether the field can affect machine safety or generated G-code.
