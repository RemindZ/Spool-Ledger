# Spool Ledger brand specification

Primary name: **Spool Ledger**  
Search and product descriptor: **Bambu Filament Migrator**

## Palette

| Role | Light | Dark |
| --- | --- | --- |
| Ground | `#E7ECE9` | `#0D1410` |
| Surface | `#F8FAF9` | `#131D18` |
| Raised | `#FFFFFF` | `#1A2720` |
| Ink | `#17211C` | `#EEF4F0` |
| Graphite | `#526159` | `#A7B8AE` |
| Border | `#CBD5CF` | `#2D4136` |
| Registration | `#009A3C` | `#10BD52` |
| Filament source | `#E86F2D` | `#F08A4E` |

```css
:root {
  --bg: oklch(93.84% 0.0067 160.07);
  --surface: oklch(98.33% 0.0025 165.08);
  --fg: oklch(23.62% 0.0172 163.06);
  --muted: oklch(47.73% 0.0224 162.20);
  --border: oklch(86.36% 0.0136 159.87);
  --accent: oklch(59.84% 0.1735 147.93);
}
```

- Display: `Segoe UI Variable Display`, `Segoe UI`, system sans-serif.
- Body: `Segoe UI Variable Text`, `Segoe UI`, system sans-serif.
- Mono: `Cascadia Mono`, `SFMono-Regular`, `Consolas`, monospace.

Observed visual rules:

1. Present the interface as a compact slicer workstation, not a dashboard or setup wizard.
2. Use Orca orange only for source provenance and route entry; Bambu green owns destinations and verified local actions.
3. Build structure with borders and tonal separation before using the single raised shadow tier.
4. Keep slicing-preset names and AMS custom-filament identities visibly separate.
5. Use the two spool arcs and central profile tile as the structural motif; reserve the green square for registration, selection, and verified destination states.
6. Keep the orange filament tail small and directional. It represents source provenance and never becomes a general-purpose action color.
7. Keep workspace navigation on the stable surface token in both themes. Do not derive it through cross-hue OKLCH interpolation.
