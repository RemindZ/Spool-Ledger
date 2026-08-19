import { describe, expect, it } from "vitest";
import {
  activeFilterChips,
  appReducer,
  createInitialState,
  resetFilters,
  sourceFacets,
  visibleSources,
} from "../state";
import type { CatalogSource } from "../types";

const sources: CatalogSource[] = [
  {
    id: "satin",
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
    id: "matte",
    name: "Panchroma PLA Matte",
    vendor: "Polymaker",
    material: "PLA",
    family: "Panchroma",
    variant: "Matte",
    source_app: "orca_slicer",
    source_kind: "user_custom",
    compatible_printers: ["Bambu Lab X1 Carbon 0.4 nozzle"],
    migration_status: "unsupported",
    warnings: ["Missing parent profile"],
  },
  {
    id: "petg",
    name: "Generic PETG",
    vendor: "Generic",
    material: "PETG",
    family: "Generic PETG",
    variant: "",
    source_app: "bambu_studio",
    source_kind: "factory_system",
    compatible_printers: [],
    migration_status: "already_migrated",
    warnings: [],
  },
];

describe("source filters", () => {
  it("builds sorted facets with counts", () => {
    const facets = sourceFacets(sources);
    expect(facets.vendors).toEqual([
      { value: "Generic", count: 1 },
      { value: "Polymaker", count: 2 },
    ]);
    expect(facets.materials).toEqual([
      { value: "PETG", count: 1 },
      { value: "PLA", count: 2 },
    ]);
  });

  it("combines every filter dimension and exposes removable chips", () => {
    let state = appReducer(createInitialState(), {
      type: "sources_loaded",
      catalogId: "sources",
      sources,
    });
    state = appReducer(state, {
      type: "filters_changed",
      filters: {
        search: "matte",
        sourceApps: new Set(["orca_slicer"]),
        sourceKinds: new Set(["user_custom"]),
        vendors: new Set(["Polymaker"]),
        materials: new Set(["PLA"]),
        families: new Set(["Panchroma"]),
        variants: new Set(["Matte"]),
        compatiblePrinters: new Set(["Bambu Lab X1 Carbon 0.4 nozzle"]),
        migrationStatuses: new Set(["unsupported"]),
      },
    });
    expect(visibleSources(state).map((source) => source.id)).toEqual(["matte"]);
    expect(
      activeFilterChips(state.filters).map((chip) => chip.label),
    ).toContain("Unsupported");
  });

  it("resets filters without clearing hidden selections", () => {
    let state = appReducer(createInitialState(), {
      type: "sources_loaded",
      catalogId: "sources",
      sources,
    });
    state = appReducer(state, { type: "source_toggled", sourceId: "satin" });
    state = appReducer(state, {
      type: "filters_changed",
      filters: { search: "PETG", selectedOnly: true },
    });
    state = appReducer(state, {
      type: "filters_changed",
      filters: resetFilters(),
    });
    expect(state.selectedSourceIds).toEqual(new Set(["satin"]));
    expect(visibleSources(state)).toHaveLength(3);
  });
});
