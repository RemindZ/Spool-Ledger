# Step surface texture: rectilinear infill skin (in use)

Chosen by Matthias on 2026-10-02 after a texture exploration with Codex (see texture-options.html).

Asset: `texture-companion-infill.png`, 1024 × 1024, seamlessly tileable. A Codex-generated macro of a real FDM rectilinear top skin in matte graphite-green filament: 24 px roads with 3 px grooves, 10° off vertical.

Post-processing: Codex returned it almost flat (contrast ≈ 1.7), so its own luminance relief was amplified ~6.3× onto a single #2C3733 hue (contrast ≈ 11). Per-pixel only, so tiling is unchanged. The faint original is kept at tmp/texture-companion-infill-original.png.

Placement in bambu-filament-migrator.html: collapsed step spines, the open step header bar, tiled at 320 px and shown at 43% over #2E3833 (implemented as a 57% #2E3833 overlay). The filament-roll photo stays exclusive to the top bar.
