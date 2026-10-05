/** The three workspace steps of the 2026-10-02 accordion. */
export const WORKSPACE_STEPS = [
  {
    title: "Filament profiles",
    description:
      "Start with a material, narrow it to a type, then pick the exact presets.",
    hint: "Pick the presets to migrate. Nothing is written yet.",
    next: "Continue to naming",
    icon: "/design/step-icon-filament.png",
  },
  {
    title: "Naming",
    description: "Keep slicing presets and AMS identities distinct.",
    hint: "Set how each output will be named in Bambu Studio.",
    next: "Continue to printers",
    icon: "/design/step-icon-naming.png",
  },
  {
    title: "Printers & nozzles",
    description: "Select nozzle sizes for each printer.",
    hint: "Building a plan does not write any files.",
    next: "Continue to review",
    icon: "/design/step-icon-printer.png",
  },
] as const;
export const STEP_IDS = [
  "step-sources",
  "step-naming",
  "step-targets",
] as const;
