import { describe, expect, it } from "vitest";
import {
  appReducer,
  createInitialState,
  defaultFilters,
  loadFilterPresets,
  loadWorkspacePreferences,
  passesSourceFilters,
  persistFilterPresets,
  persistWorkspacePreferences,
  type PreferencesStorage,
} from "../state";
import type { CatalogSource } from "../types";

const sources: CatalogSource[] = [
  {
    id: "satin",
    name: "Northstar PLA Satin",
    vendor: "Northstar",
    material: "PLA",
    material_archetype: "PLA",
    material_type: "Standard",
    family: "Northstar PLA",
    variant: "Satin",
    source_app: "orca_slicer",
    source_kind: "factory_system",
    compatible_printers: ["Bambu Lab X1 Carbon 0.4 nozzle"],
    migration_status: "new",
    warnings: [],
  },
  {
    id: "matte",
    name: "Northstar PLA Matte",
    vendor: "Northstar",
    material: "PLA",
    material_archetype: "PLA",
    material_type: "Matte",
    family: "Northstar PLA",
    variant: "Matte",
    source_app: "orca_slicer",
    source_kind: "user_custom",
    compatible_printers: ["Bambu Lab H2C 0.4 nozzle"],
    migration_status: "unsupported",
    warnings: ["Missing parent profile"],
  },
  {
    id: "petg",
    name: "Overture PETG",
    vendor: "Overture",
    material: "PETG",
    material_archetype: "PETG",
    material_type: "Standard",
    family: "Overture PETG",
    variant: "",
    source_app: "bambu_studio",
    source_kind: "factory_system",
    compatible_printers: [],
    migration_status: "already_migrated",
    warnings: [],
  },
];

function memoryStorage(): PreferencesStorage & { values: Map<string, string> } {
  const values = new Map<string, string>();
  return {
    values,
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => void values.set(key, value),
  };
}

const visible = (filters = defaultFilters()) =>
  sources
    .filter((source) => passesSourceFilters(source, filters))
    .map((source) => source.id);

describe("source filters", () => {
  it("start with every source, manufacturer, printer and status checked", () => {
    expect(visible()).toEqual(["satin", "matte", "petg"]);
  });

  it("combine source pairs and unchecked manufacturers, printers and statuses", () => {
    expect(
      visible({
        ...defaultFilters(),
        sources: new Set(["orca_slicer:factory_system"]),
      }),
    ).toEqual(["satin"]);
    expect(
      visible({ ...defaultFilters(), hiddenVendors: new Set(["Northstar"]) }),
    ).toEqual(["petg"]);
    expect(
      visible({
        ...defaultFilters(),
        hiddenStatuses: new Set(["unsupported", "already_migrated"]),
      }),
    ).toEqual(["satin"]);
    expect(
      visible({
        ...defaultFilters(),
        hiddenPrinters: new Set(["Bambu Lab H2C 0.4 nozzle"]),
      }),
    ).toEqual(["satin"]);
  });

  it("maps first-run application and kind choices onto source pairs", () => {
    const state = appReducer(createInitialState(), {
      type: "setup_completed",
      sourceApps: new Set(["orca_slicer"]),
      sourceKinds: new Set(),
      enabledPrinterIds: new Set(),
      selectedNozzles: {},
    });
    expect([...state.filters.sources]).toEqual([
      "orca_slicer:factory_system",
      "orca_slicer:user_custom",
    ]);
  });

  it("selects and clears explicit catalog ids only", () => {
    let state = appReducer(createInitialState(), {
      type: "sources_loaded",
      catalogId: "sources",
      sources,
    });
    state = appReducer(state, {
      type: "sources_selected",
      sourceIds: ["satin", "petg", "missing"],
      selected: true,
    });
    expect([...state.selectedSourceIds].sort()).toEqual(["petg", "satin"]);
    state = appReducer(state, {
      type: "sources_selected",
      sourceIds: ["petg"],
      selected: false,
    });
    expect([...state.selectedSourceIds]).toEqual(["satin"]);
  });
});

describe("filter persistence", () => {
  it("round-trips the picker filters in workspace preferences and saved filters", () => {
    const storage = memoryStorage();
    const filters = {
      ...defaultFilters(),
      search: "matte",
      sources: new Set([
        "orca_slicer:user_custom",
        "bambu_studio:user_custom",
      ] as const),
      hiddenVendors: new Set(["Overture"]),
      hiddenPrinters: new Set(["Bambu Lab H2C 0.4 nozzle"]),
      hiddenStatuses: new Set(["unsupported"] as const),
      selectedOnly: true,
    };
    persistWorkspacePreferences(storage, {
      ...createInitialState(),
      setupComplete: true,
      filters,
    });
    expect(JSON.parse(storage.values.get("bfm.workspace")!).version).toBe(2);
    expect(loadWorkspacePreferences(storage)?.filters).toEqual(filters);

    persistFilterPresets(storage, [{ id: "mine", name: "Mine", filters }]);
    expect(loadFilterPresets(storage)).toEqual([
      { id: "mine", name: "Mine", filters },
    ]);
  });

  it("migrates earlier include-list filters without guessing manufacturers", () => {
    const storage = memoryStorage();
    storage.setItem(
      "bfm.workspace",
      JSON.stringify({
        version: 2,
        setup_complete: true,
        enabled_printer_ids: [],
        selected_account_id: null,
        selected_source_ids: [],
        selected_nozzles: {},
        filters: {
          search: "",
          source_apps: ["bambu_studio"],
          source_kinds: [],
          vendors: ["Overture"],
          materials: ["PLA"],
          families: [],
          variants: [],
          compatible_printers: [],
          migration_statuses: ["new", "incomplete"],
          selected_only: false,
        },
      }),
    );
    const filters = loadWorkspacePreferences(storage)!.filters;
    expect([...filters.sources]).toEqual([
      "bambu_studio:factory_system",
      "bambu_studio:user_custom",
    ]);
    expect(filters.hiddenVendors.size).toBe(0);
    expect([...filters.hiddenStatuses].sort()).toEqual([
      "already_migrated",
      "conflicting",
      "unsupported",
    ]);
  });
});
