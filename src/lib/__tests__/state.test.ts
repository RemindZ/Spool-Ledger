import { describe, expect, it } from "vitest";
import {
  appReducer,
  createInitialState,
  loadFilterPresets,
  loadNamingPresets,
  loadWorkspacePreferences,
  persistFilterPresets,
  persistNamingPresets,
  persistWorkspacePreferences,
  selectedNozzleRequests,
  toggleSetValue,
  removeSetValue,
  visibleSources,
  type AppAction,
} from "../state";
import type {
  CatalogSource,
  DiscoveryResponse,
  MigrationPlan,
  PrinterTarget,
} from "../types";

const discovery: DiscoveryResponse = {
  source_root_ids: ["source:orca:system"],
  manual_source_roots: [],
  target_catalog_ids: ["target:bambu"],
  orca_slicer_detected: true,
  bambu_studio_detected: true,
  platform: "windows",
  accounts: [
    {
      id: "0000000000",
      eligibility: "eligible",
      filament_profile_count: 16,
    },
  ],
};

const sources: CatalogSource[] = [
  {
    id: "panchroma-satin",
    name: "Panchroma PLA Satin",
    vendor: "Polymaker",
    material: "PLA",
    family: "Panchroma",
    variant: "Satin",
    source_app: "orca_slicer",
    source_kind: "factory_system",
    compatible_printers: ["Bambu Lab X1 Carbon 0.4 nozzle"],
    migration_status: "new",
    warnings: [],
  },
  {
    id: "bambu-basic",
    name: "Bambu PLA Basic",
    vendor: "Bambu Lab",
    material: "PLA",
    family: "PLA Basic",
    variant: "Blue",
    source_app: "bambu_studio",
    source_kind: "user_custom",
    compatible_printers: [],
    migration_status: "already_migrated",
    warnings: [],
  },
];

const printers: PrinterTarget[] = [
  {
    id: "official:H2C",
    name: "Bambu Lab H2C",
    code: "H2C",
    kind: "official",
    verified: true,
    artwork_available: false,
    extruder_variants: [
      "Direct Drive Standard",
      "Direct Drive High Flow",
      "Direct Drive E3D High Flow",
    ],
    nozzles: ["0.2", "0.4", "0.6", "0.8"].map((diameter) => ({
      id: `H2C:${diameter}`,
      diameter,
      printer_preset_name: `Bambu Lab H2C ${diameter} nozzle`,
      selected: false,
      supported: true,
    })),
  },
];

function reduce(actions: AppAction[]) {
  return actions.reduce(appReducer, createInitialState());
}

describe("application state", () => {
  it("updates plain sets without mutating the source collection", () => {
    const source = new Set(["PLA", "PETG"]);
    const toggled = toggleSetValue(source, "PLA");
    const removed = removeSetValue(source, "PETG");
    expect(toggled).toEqual(new Set(["PETG"]));
    expect(removed).toEqual(new Set(["PLA"]));
    expect(toggled.constructor).toBe(Set);
    expect(source).toEqual(new Set(["PLA", "PETG"]));
  });

  it("round-trips reusable naming presets and rejects corrupt storage", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    };
    const presets = [
      {
        id: "panchroma-clean",
        name: "Panchroma clean",
        preset_template: "{clean_name} - {printer_code}",
        ams_template: "{vendor} {material} {clean_name}",
        preset_rules: [],
        ams_rules: [
          {
            id: "rule-1",
            kind: "wildcard" as const,
            pattern: "Polymaker PLA *",
            replacement: "$1",
            case_sensitive: false,
            condition: null,
          },
        ],
      },
    ];
    persistNamingPresets(storage, presets);
    expect(loadNamingPresets(storage)).toEqual(presets);
    values.set("bfm.naming-presets", "not json");
    expect(loadNamingPresets(storage)).toEqual([]);
  });

  it("round-trips workspace selections and reusable filter presets", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    };
    let configured = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "targets_loaded", catalogId: "targets:1", printers },
      {
        type: "setup_completed",
        sourceApps: new Set(["orca_slicer", "bambu_studio"]),
        sourceKinds: new Set(["factory_system", "user_custom"]),
        enabledPrinterIds: new Set(["official:H2C"]),
        selectedNozzles: { "official:H2C": new Set(["0.4"]) },
      },
      { type: "source_toggled", sourceId: "panchroma-satin" },
      {
        type: "filters_changed",
        filters: { vendors: new Set(["Polymaker"]), selectedOnly: true },
      },
    ]);
    configured = appReducer(configured, {
      type: "templates_changed",
      preset: "{clean_name}",
      ams: "{vendor} {clean_name}",
    });
    persistWorkspacePreferences(storage, configured);
    expect(JSON.parse(values.get("bfm.workspace")!)).not.toHaveProperty(
      "show_custom_printers",
    );
    const restored = createInitialState(
      "system",
      loadWorkspacePreferences(storage),
    );
    expect(restored.selectedSourceIds).toEqual(new Set(["panchroma-satin"]));
    expect(restored.selectedNozzles["official:H2C"]).toEqual(new Set(["0.4"]));
    expect(restored.filters.vendors).toEqual(new Set(["Polymaker"]));
    expect(restored.filters.selectedOnly).toBe(true);
    expect(restored.presetTemplate).toBe("{clean_name}");

    const filterPresets = [
      { id: "polymaker", name: "Polymaker", filters: configured.filters },
    ];
    persistFilterPresets(storage, filterPresets);
    expect(loadFilterPresets(storage)).toEqual(filterPresets);
    values.set("bfm.workspace", "{");
    values.set("bfm.filter-presets", "null");
    expect(loadWorkspacePreferences(storage)).toBeNull();
    expect(loadFilterPresets(storage)).toEqual([]);
  });

  it("starts fresh installs without setup or implicitly enabled printers", () => {
    const state = createInitialState();

    expect(state.setupComplete).toBe(false);
    expect(state.enabledPrinterIds).toEqual(new Set());
    expect(selectedNozzleRequests(state)).toEqual([]);
  });

  it("migrates version 1 preferences without losing selections and shows setup once", () => {
    const values = new Map<string, string>();
    const storage = {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
    };
    let configured = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "targets_loaded", catalogId: "targets:1", printers },
      { type: "source_toggled", sourceId: "panchroma-satin" },
      {
        type: "nozzle_toggled",
        printerId: "official:H2C",
        diameter: "0.4",
      },
    ]);
    configured = {
      ...configured,
      setupComplete: true,
      enabledPrinterIds: new Set(["official:H2C"]),
      selectedNozzles: { "official:H2C": new Set(["0.4"]) },
    };
    persistWorkspacePreferences(storage, configured);
    const legacy = JSON.parse(values.get("bfm.workspace")!);
    legacy.version = 1;
    delete legacy.setup_complete;
    delete legacy.enabled_printer_ids;
    values.set("bfm.workspace", JSON.stringify(legacy));

    const preferences = loadWorkspacePreferences(storage)!;

    expect(preferences.setupComplete).toBe(false);
    expect(preferences.enabledPrinterIds).toEqual(["official:H2C"]);
    expect(preferences.selectedSourceIds).toEqual(["panchroma-satin"]);
    expect(preferences.selectedNozzles).toEqual({ "official:H2C": ["0.4"] });
  });

  it("applies first-run filters, enabled printers, and supported nozzle defaults atomically", () => {
    const planned: MigrationPlan = { id: "stale-plan", operations: [] };
    let state = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "targets_loaded", catalogId: "targets:1", printers },
      { type: "plan_built", plan: planned },
    ]);

    state = appReducer(state, {
      type: "setup_completed",
      sourceApps: new Set(["orca_slicer"]),
      sourceKinds: new Set(["factory_system"]),
      enabledPrinterIds: new Set(["official:H2C", "official:unknown"]),
      selectedNozzles: {
        "official:H2C": new Set(["0.4", "1.0"]),
        "official:unknown": new Set(["0.4"]),
      },
    });

    expect(state.setupComplete).toBe(true);
    expect(state.filters.sourceApps).toEqual(new Set(["orca_slicer"]));
    expect(state.filters.sourceKinds).toEqual(new Set(["factory_system"]));
    expect(state.enabledPrinterIds).toEqual(new Set(["official:H2C"]));
    expect(state.selectedNozzles).toEqual({
      "official:H2C": new Set(["0.4"]),
    });
    expect(state.plan).toBeNull();
  });

  it("removes disabled printers from plan requests and leaves new discoveries disabled", () => {
    const morePrinters: PrinterTarget[] = [
      ...printers,
      {
        ...printers[0],
        id: "official:X1C",
        name: "Bambu Lab X1 Carbon",
        code: "X1C",
        nozzles: printers[0].nozzles.map((nozzle) => ({
          ...nozzle,
          id: `X1C:${nozzle.diameter}`,
          printer_preset_name: `Bambu Lab X1 Carbon ${nozzle.diameter} nozzle`,
        })),
      },
    ];
    let state = reduce([
      {
        type: "targets_loaded",
        catalogId: "targets:1",
        printers: morePrinters,
      },
    ]);
    state = appReducer(state, {
      type: "setup_completed",
      sourceApps: new Set(["orca_slicer"]),
      sourceKinds: new Set(["factory_system"]),
      enabledPrinterIds: new Set(["official:H2C"]),
      selectedNozzles: { "official:H2C": new Set(["0.4"]) },
    });
    state = {
      ...state,
      selectedNozzles: {
        ...state.selectedNozzles,
        "official:X1C": new Set(["0.4"]),
      },
    };

    expect(selectedNozzleRequests(state)).toEqual([
      { printer_id: "official:H2C", diameters: ["0.4"] },
    ]);

    state = appReducer(state, {
      type: "enabled_printers_changed",
      enabledPrinterIds: new Set(["official:H2C"]),
      selectedNozzles: {},
    });
    expect(state.enabledPrinterIds).toEqual(new Set(["official:H2C"]));
    expect(selectedNozzleRequests(state)).toEqual([]);

    state = appReducer(state, {
      type: "enabled_printers_changed",
      enabledPrinterIds: new Set(),
      selectedNozzles: {},
    });
    expect(state.enabledPrinterIds).toEqual(new Set());
    expect(state.selectedNozzles).toEqual({});
    expect(selectedNozzleRequests(state)).toEqual([]);
  });

  it("prunes persisted selections that no longer exist in refreshed catalogs", () => {
    let state = createInitialState();
    state = {
      ...state,
      selectedSourceIds: new Set(["panchroma-satin", "gone-source"]),
      enabledPrinterIds: new Set(["official:H2C", "gone-printer"]),
      selectedNozzles: {
        "official:H2C": new Set(["0.4", "1.0"]),
        "gone-printer": new Set(["0.4"]),
      },
    };
    state = appReducer(state, {
      type: "sources_loaded",
      catalogId: "sources:1",
      sources,
    });
    state = appReducer(state, {
      type: "targets_loaded",
      catalogId: "targets:1",
      printers,
    });
    expect(state.selectedSourceIds).toEqual(new Set(["panchroma-satin"]));
    expect(state.selectedNozzles).toEqual({
      "official:H2C": new Set(["0.4"]),
    });
  });

  it("represents discovery loading, error, and ready states", () => {
    const loading = reduce([{ type: "discovery_started" }]);
    expect(loading.phase).toBe("discovering");
    const failed = appReducer(loading, {
      type: "failed",
      message: "Bambu Studio not found",
    });
    expect(failed.phase).toBe("error");
    expect(failed.error).toContain("not found");
    const ready = appReducer(failed, { type: "discovery_loaded", discovery });
    expect(ready.phase).toBe("ready");
    expect(ready.error).toBeNull();
  });

  it("selects only eligible accounts and persists explicit theme choice", () => {
    const ready = reduce([{ type: "discovery_loaded", discovery }]);
    const selected = appReducer(ready, {
      type: "account_selected",
      accountId: "0000000000",
    });
    expect(selected.selectedAccountId).toBe("0000000000");
    const themed = appReducer(selected, {
      type: "theme_changed",
      theme: "dark",
    });
    expect(themed.theme).toBe("dark");
  });

  it("keeps hidden selections while filters change", () => {
    let state = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "source_toggled", sourceId: "panchroma-satin" },
      { type: "source_toggled", sourceId: "bambu-basic" },
    ]);
    state = appReducer(state, {
      type: "filters_changed",
      filters: { search: "panchroma", sourceApps: new Set(["orca_slicer"]) },
    });
    expect(visibleSources(state)).toHaveLength(1);
    expect(state.selectedSourceIds).toEqual(
      new Set(["panchroma-satin", "bambu-basic"]),
    );
    state = appReducer(state, { type: "select_visible", selected: false });
    expect(state.selectedSourceIds).toEqual(new Set(["bambu-basic"]));
  });

  it("selects nozzles independently and never selects all by choosing a printer", () => {
    let state = reduce([
      { type: "targets_loaded", catalogId: "targets:1", printers },
      {
        type: "setup_completed",
        sourceApps: new Set(["orca_slicer"]),
        sourceKinds: new Set(["factory_system"]),
        enabledPrinterIds: new Set(["official:H2C"]),
        selectedNozzles: { "official:H2C": new Set(["0.4"]) },
      },
    ]);
    state = appReducer(state, {
      type: "nozzle_toggled",
      printerId: "official:H2C",
      diameter: "0.6",
    });
    expect(selectedNozzleRequests(state)).toEqual([
      { printer_id: "official:H2C", diameters: ["0.4", "0.6"] },
    ]);
  });

  it("rejects unsupported nozzle toggles and excludes them from select all", () => {
    const withUnsupported: PrinterTarget[] = [
      {
        ...printers[0],
        nozzles: [
          ...printers[0].nozzles,
          {
            id: "H2C:1.0",
            diameter: "1.0",
            printer_preset_name: "Bambu Lab H2C 1.0 nozzle",
            selected: false,
            supported: false,
          },
        ],
      },
    ];
    let state = reduce([
      {
        type: "targets_loaded",
        catalogId: "targets:unsupported",
        printers: withUnsupported,
      },
      {
        type: "setup_completed",
        sourceApps: new Set(["orca_slicer"]),
        sourceKinds: new Set(["factory_system"]),
        enabledPrinterIds: new Set(["official:H2C"]),
        selectedNozzles: { "official:H2C": new Set(["0.4"]) },
      },
    ]);

    state = appReducer(state, {
      type: "nozzle_toggled",
      printerId: "official:H2C",
      diameter: "1.0",
    });
    expect(selectedNozzleRequests(state)).toEqual([
      { printer_id: "official:H2C", diameters: ["0.4"] },
    ]);

    state = appReducer(state, {
      type: "select_printer_nozzles",
      printerId: "official:H2C",
      selected: true,
    });
    expect(selectedNozzleRequests(state)[0].diameters).toEqual([
      "0.2",
      "0.4",
      "0.6",
      "0.8",
    ]);
  });

  it("invalidates stale plans after selection, target, account, or naming changes", () => {
    const plan: MigrationPlan = { id: "plan-1", operations: [] };
    const planned = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "targets_loaded", catalogId: "targets:1", printers },
      {
        type: "setup_completed",
        sourceApps: new Set(["orca_slicer"]),
        sourceKinds: new Set(["factory_system"]),
        enabledPrinterIds: new Set(["official:H2C"]),
        selectedNozzles: { "official:H2C": new Set(["0.4"]) },
      },
      { type: "plan_built", plan },
    ]);
    expect(planned.plan?.id).toBe("plan-1");
    for (const action of [
      { type: "source_toggled", sourceId: "panchroma-satin" },
      { type: "nozzle_toggled", printerId: "official:H2C", diameter: "0.4" },
      { type: "account_selected", accountId: "0000000000" },
      {
        type: "rules_changed",
        presetRules: [],
        amsRules: [],
      },
      {
        type: "templates_changed",
        preset: "{clean_name}",
        ams: "{vendor} {clean_name}",
      },
    ] satisfies AppAction[]) {
      expect(appReducer(planned, action).plan).toBeNull();
    }
  });

  it("clears conflict decisions when the plan context changes", () => {
    const decision = {
      source_id: "panchroma-satin",
      printer_id: "official:H2C",
      nozzle: "0.4",
      choice: "update" as const,
    };
    const decided = reduce([
      { type: "discovery_loaded", discovery },
      { type: "conflict_decisions_changed", decisions: [decision] },
    ]);
    expect(decided.conflictDecisions).toEqual([decision]);

    for (const action of [
      { type: "account_selected", accountId: "0000000000" },
      { type: "source_toggled", sourceId: "panchroma-satin" },
      {
        type: "templates_changed",
        preset: "{clean_name}",
        ams: "{vendor} {clean_name}",
      },
      {
        type: "outputs_changed",
        outputs: { slicing_presets: true, custom_filaments: false },
      },
    ] satisfies AppAction[]) {
      expect(appReducer(decided, action).conflictDecisions).toEqual([]);
    }
  });

  it("returns typed dependency planning to setup without a generic error", () => {
    let state = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "source_toggled", sourceId: "panchroma-satin" },
      { type: "source_toggled", sourceId: "bambu-basic" },
      { type: "planning_started" },
    ]);

    state = appReducer(state, { type: "planning_stopped" });
    state = appReducer(state, {
      type: "sources_removed",
      sourceIds: new Set(["panchroma-satin"]),
    });

    expect(state.phase).toBe("ready");
    expect(state.error).toBeNull();
    expect(state.selectedSourceIds).toEqual(new Set(["bambu-basic"]));
  });

  it("records synchronization evidence without discarding a committed local run", () => {
    const local = {
      run_id: "run-1",
      plan_id: "plan-1",
      committed_files: 3,
      created_files: 3,
      updated_files: 0,
      deleted_files: 0,
      skipped_operations: 0,
      backup_sha256: "backup-sha256",
      backup_file_count: 3,
      receipt_path: "receipt.json",
    };
    let state = reduce([{ type: "execution_completed", result: local }]);
    state = appReducer(state, { type: "synchronization_started" });
    expect(state.phase).toBe("synchronizing");
    state = appReducer(state, {
      type: "synchronization_phase",
      phase: "monitoring",
    });
    state = appReducer(state, {
      type: "synchronization_completed",
      result: {
        timed_out: true,
        highest_evidence: "created_local",
        observations: [
          {
            operation_id: "op-1",
            info_path: "profile.info",
            evidence: "created_local",
            state: "created_local_unsynchronized",
            setting_id: null,
            diagnostic: null,
          },
        ],
      },
    });
    expect(state.phase).toBe("complete");
    expect(state.result).toEqual(local);
    expect(state.synchronization?.timed_out).toBe(true);
    expect(state.progress["op-1"].state).toBe("created_local_unsynchronized");

    const failed = appReducer(state, {
      type: "synchronization_failed",
      message: "launch failed",
    });
    expect(failed.result).toEqual(local);
    expect(failed.syncError).toBe("launch failed");
  });

  it("keeps cloud assignment distinct from operator AMS verification", () => {
    const progressed = reduce([
      {
        type: "progress_updated",
        operationId: "op-1",
        evidence: "cloud_id_assigned",
        state: "cloud_id_assigned",
      },
    ]);
    expect(progressed.progress["op-1"].evidence).toBe("cloud_id_assigned");
    expect(progressed.progress["op-1"].evidence).not.toBe("ams_verified");
    const verified = appReducer(progressed, {
      type: "ams_verified",
      operationIds: ["op-1"],
    });
    expect(verified.progress["op-1"].evidence).toBe("ams_verified");
  });
});
