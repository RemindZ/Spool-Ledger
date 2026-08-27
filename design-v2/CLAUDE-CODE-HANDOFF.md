# Claude Code implementation prompt

Implement the approved Spool Ledger design update in this existing production repository.

## Product identity

- Primary name: **Spool Ledger**
- Permanent descriptor: **Bambu Filament Migrator**
- Primary promise: convert custom or built-in OrcaSlicer filament presets into Bambu custom filaments that users can select on the printer screen and assign to AMS slots.

The files beside this prompt are the approved Open Design handoff. The repository itself contains the previous production implementation that must be compared with this handoff.

## Source-of-truth order

1. Existing production code, tests, schemas, adapters, and proven application behavior are authoritative for functional truth.
2. `bambu-filament-migrator.html` is the current approved visual and interaction reference.
3. `DESIGN.md` defines product intent, workflow, accessibility, implementation constraints, and safety language.
4. `brand-spec.md`, `brand-identity-directions.md`, and the supplied media define the approved identity.
5. `README.md` is the approved repository-facing content.

Where visual token examples conflict, follow the current approved `bambu-filament-migrator.html`. Do not weaken functional contracts to imitate prototype behavior.

## Objective

Compare the existing production application with `bambu-filament-migrator.html`, then implement the complete relevant design delta using the repository's existing frontend architecture and component system.

Do not replace the production application with the standalone HTML prototype. Translate the approved design into the actual components, routes, state management, commands, and backend integration already used by the repository.

## Workflow

1. Inspect the repository before editing. Identify the frontend framework, entry points, components, theme implementation, assets, tests, and official validation commands.
2. Locate the real Setup, Review plan, Run and evidence, and Restore surfaces plus their state and backend integrations.
3. Read every handoff document in full.
4. Produce a concise comparison checklist covering layout, branding, typography, colors, component states, interactions, copy, accessibility, responsive behavior, and media. Map each item to the production component or file.
5. Continue directly from comparison into implementation. Do not stop after the audit unless a material conflict genuinely requires user direction.

## Implementation requirements

- Use the supplied logo files exactly. Never redraw, approximate, trace, or rebuild the logo with HTML, CSS, SVG primitives, or an image model.
- Place media in the repository's normal asset location and update paths appropriately.
- Implement both light and dark themes using the approved Spool Ledger identity.
- Preserve the compact slicer-workstation character; do not turn the interface into a generic dashboard.
- Orange represents Orca/source provenance only. Green represents Bambu destinations, selection, verified results, and primary actions.
- Keep the dark workspace green-black. The workspace navigation must use the stable surface color and must not acquire a purple OKLCH interpolation tint.
- Implement the approved custom source-selection checkboxes with clear unchecked, checked, hover, focus, and disabled states in both themes.
- Help buttons must contain one clear question mark. Do not draw a second circle inside the circular button. Keep the larger, legible help glyph consistently across the help-button family.
- Use the Setup to Review plan transition treatment consistently for every workspace transition.
- During initial filament inventory, show a centered loading circle, a progress bar underneath, and real loaded-of-total values. Never invent counts.
- Preserve keyboard navigation, visible focus indicators, accessible names, reduced-motion handling, meaningful status announcements, and appropriate target sizes.
- Update the production README using the supplied `README.md` and media, correcting relative asset paths for the actual repository.

## Functional safety boundaries

The HTML prototype contains illustrative data and simulated interactions. Do not copy fake counts, timers, accounts, run IDs, success states, or backend behavior into production.

Do not claim or infer cloud or AMS success from local file creation. Preserve the separate evidence states `created_local`, `loaded_by_bambu`, `cloud_id_assigned`, and `ams_verified`.

Keep Bambu slicing presets separate from AMS custom-filament identities. Do not invent unsupported controls or editable fields.

Restore must remain journal-owned and path-specific. Do not replace it with whole-account or whole-directory restoration.

Do not implement direct cloud API calls or UI automation unless those already exist as an explicitly validated production contract. Characterize installed application versions, schema versions, account roots, and sidecar formats before changing migration behavior.

## Change discipline

- Preserve current business logic, backend commands, adapters, data models, and error handling unless design integration genuinely requires a small change.
- Avoid unrelated refactors.
- Do not modify the handoff reference files.
- Do not create a second parallel frontend.
- Reuse existing components and styling infrastructure where practical.

## Verification

Discover and run the repository's official formatting, linting, type-checking, unit-test, integration-test, and production-build commands.

Verify the real application in both light and dark themes through:

1. Initial inventory and progress.
2. Setup and source selection.
3. Naming and hardware targets.
4. Build and review migration plan.
5. Run and evidence states.
6. Restore preview and journal-owned restore.
7. Keyboard navigation and focus.
8. Theme switching.
9. Responsive or minimum-window behavior.

Confirm there are no overlaps, clipping, horizontal overflow, illegible icons, native unstyled source checkboxes, purple navigation backgrounds, or fabricated success states.

## Final report

Return the comparison checklist and final status, files changed, design changes implemented, functional behavior deliberately preserved, exact test/build results, light/dark verification performed, and any remaining blockers. Include screenshots where the repository workflow supports them.

Do not report completion unless the implementation, tests, build, and visual verification actually passed.
