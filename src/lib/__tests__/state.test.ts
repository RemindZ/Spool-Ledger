import { describe, expect, it } from "vitest";
import {
  appReducer,
  createInitialState,
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
  target_catalog_ids: ["target:bambu"],
  accounts: [
    {
      id: "2182110758",
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
      accountId: "2182110758",
    });
    expect(selected.selectedAccountId).toBe("2182110758");
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
    ]);
    state = appReducer(state, {
      type: "nozzle_toggled",
      printerId: "official:H2C",
      diameter: "0.4",
    });
    expect(selectedNozzleRequests(state)).toEqual([
      { printer_id: "official:H2C", diameters: ["0.4"] },
    ]);
  });

  it("invalidates stale plans after selection, target, account, or naming changes", () => {
    const plan: MigrationPlan = { id: "plan-1", operations: [] };
    const planned = reduce([
      { type: "sources_loaded", catalogId: "sources:1", sources },
      { type: "targets_loaded", catalogId: "targets:1", printers },
      { type: "plan_built", plan },
    ]);
    expect(planned.plan?.id).toBe("plan-1");
    for (const action of [
      { type: "source_toggled", sourceId: "panchroma-satin" },
      { type: "nozzle_toggled", printerId: "official:H2C", diameter: "0.4" },
      { type: "account_selected", accountId: "2182110758" },
      {
        type: "templates_changed",
        preset: "{clean_name}",
        ams: "{vendor} {clean_name}",
      },
    ] satisfies AppAction[]) {
      expect(appReducer(planned, action).plan).toBeNull();
    }
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
