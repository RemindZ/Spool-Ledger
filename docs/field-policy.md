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

## Characterized OrcaSlicer 2.4 filament fields

OrcaSlicer 2.4 saves user custom filament presets fully flattened, so every Orca filament option appears in the source file. The following 35 keys are classified as `target_machine`:

`activate_chamber_temp_control`, `adaptive_pressure_advance`, `adaptive_pressure_advance_bridges`, `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs`, `chamber_temperature`, `dont_slow_down_outer_wall`, `filament_cooling_final_speed`, `filament_cooling_initial_speed`, `filament_cooling_moves`, `filament_ironing_flow`, `filament_ironing_inset`, `filament_ironing_spacing`, `filament_ironing_speed`, `filament_loading_speed`, `filament_loading_speed_start`, `filament_multitool_ramming`, `filament_multitool_ramming_flow`, `filament_multitool_ramming_volume`, `filament_ramming_parameters`, `filament_retract_lift_above`, `filament_retract_lift_below`, `filament_retract_lift_enforce`, `filament_shrinkage_compensation_z`, `filament_stamping_distance`, `filament_stamping_loading_speed`, `filament_toolchange_delay`, `filament_unloading_speed`, `filament_unloading_speed_start`, `idle_temperature`, `internal_bridge_fan_speed`, `pellet_flow_coefficient`, `support_material_interface_fan_speed`, `textured_cool_plate_temp`, `textured_cool_plate_temp_initial_layer`.

Evidence:

- Every key is in the OrcaSlicer filament-preset option list in [`Preset.cpp`](https://github.com/OrcaSlicer/OrcaSlicer/blob/08f086daf317c657e86c8292cc34ea14c6322527/src/libslic3r/Preset.cpp).
- No key appears anywhere in Bambu Studio [`Preset.cpp`](https://github.com/bambulab/BambuStudio/blob/da8b44ee34dd349f2ae0df3f1cbae366df482354/src/libslic3r/Preset.cpp).
- No key appears in the installed Bambu Studio or Bambu Studio Beta system profile roots.

Bambu Studio has no destination for these values, so the writer keeps the resolved Bambu target, which never contains them. Resolver and writer regressions verify the classification and that none of the keys reach staged Bambu output.

`chamber_temperature` is not mapped to Bambu `chamber_temperatures`. In OrcaSlicer the value only takes effect when `activate_chamber_temp_control` is on, while Bambu applies `chamber_temperatures` directly. Copying an inactive Orca value could enable chamber heating the user never enabled, so the Bambu target value is kept. A gated mapping needs its own characterization and machine-safety review.

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
