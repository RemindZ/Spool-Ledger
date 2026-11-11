import { describe, expect, it } from "vitest";
import {
  emptyHistory,
  emptyStateKind,
  groupPresets,
  hiddenSelectedCount,
  inScope,
  materialCards,
  materialLabel,
  normalizePath,
  recordNavigation,
  scopeSelection,
  searchGroups,
  setVisibleVendors,
  sourceTriggerLabel,
  travel,
  typeCards,
  typesOf,
  vendorTriggerLabel,
  viewSources,
  type PickerView,
} from "../picker";
import { defaultFilters, type SourceFilters, type SourceKey } from "../state";
import type { CatalogSource } from "../types";

function preset(
  id: string,
  vendor: string,
  archetype: string,
  type: string,
  overrides: Partial<CatalogSource> = {},
): CatalogSource {
  return {
    id,
    name: `${vendor} ${archetype} ${type}`,
    vendor,
    material: archetype,
    material_archetype: archetype,
    material_type: type,
    family: `${vendor} ${archetype}`,
    variant: type,
    source_app: "orca_slicer",
    source_kind: "factory_system",
    compatible_printers: [],
    migration_status: "new",
    warnings: [],
    ...overrides,
  };
}

const catalog: CatalogSource[] = [
  preset("pla-std", "Overture", "PLA", "Standard"),
  preset("pla-matte", "Overture", "PLA", "Matte"),
  preset("pla-silk", "Fiberlogy", "PLA", "Silk", {
    source_app: "bambu_studio",
  }),
  preset("pla-matte-2", "Fiberlogy", "PLA", "Matte", {
    source_kind: "user_custom",
  }),
  preset("abs-std", "Overture", "ABS", "Standard"),
  preset("abs-gf", "Fiberlogy", "ABS", "Glass fiber"),
  preset("pctg", "Fiberlogy", "PCTG", "Standard"),
  preset("pva", "Overture", "PVA", "Standard"),
  preset("other", "Overture", "Other", "Standard"),
];

function view(overrides: Partial<PickerView> = {}): PickerView {
  return {
    sources: catalog,
    filters: defaultFilters(),
    selected: new Set<string>(),
    path: [],
    query: "",
    ...overrides,
  };
}

function filters(overrides: Partial<SourceFilters>): SourceFilters {
  return { ...defaultFilters(), ...overrides };
}

describe("material tiers", () => {
  it("orders known materials first, appends extras, and hides empty ones", () => {
    expect(materialCards(view()).map((card) => card.id)).toEqual([
      "PLA",
      "PCTG",
      "ABS",
      "PVA",
      "Other",
    ]);
    expect(materialLabel("PA")).toBe("PA · Nylon");
    const vendorFiltered = view({
      filters: filters({ hiddenVendors: new Set(["Fiberlogy"]) }),
    });
    expect(materialCards(vendorFiltered).map((card) => card.id)).toEqual([
      "PLA",
      "ABS",
      "PVA",
      "Other",
    ]);
  });

  it("recomputes the tiering rule from the types visible under the filters", () => {
    const [pla, , abs, pva] = materialCards(view());
    expect(pla.layout).toBe("tiered");
    expect(pla.types).toEqual(["Standard", "Matte", "Silk"]);
    expect(abs.layout).toBe("rows");
    expect(abs.types).toEqual(["Standard", "Glass fiber"]);
    expect(pva.layout).toBe("single");

    const orcaOnly = view({
      filters: filters({
        sources: new Set<SourceKey>([
          "orca_slicer:factory_system",
          "orca_slicer:user_custom",
        ]),
      }),
    });
    const plaOrca = materialCards(orcaOnly)[0];
    expect(plaOrca.types).toEqual(["Standard", "Matte"]);
    expect(plaOrca.layout).toBe("rows");
  });

  it("summarises counts, manufacturers and selection on each card", () => {
    const card = materialCards(
      view({ selected: new Set(["pla-matte", "abs-std"]) }),
    )[0];
    expect(card).toMatchObject({
      id: "PLA",
      count: 4,
      manufacturers: 2,
      selected: 1,
    });
    const types = typeCards(view({ path: ["PLA"] }));
    expect(types.map((item) => item.type)).toEqual([
      "Standard",
      "Matte",
      "Silk",
    ]);
    expect(types[1]).toMatchObject({
      count: 2,
      manufacturerNames: ["Fiberlogy", "Overture"],
    });
  });

  it("opens single-type materials straight into presets and skips absent tiers going back", () => {
    expect(normalizePath(view(), ["PCTG"], "forward")).toEqual([
      "PCTG",
      "Standard",
    ]);
    expect(normalizePath(view(), ["ABS"], "back")).toEqual([]);
    expect(normalizePath(view(), ["PLA"], "back")).toEqual(["PLA"]);
    expect(normalizePath(view(), ["ABS"], "back", true)).toEqual(["ABS"]);
  });

  it("orders types with Standard first and fibers last", () => {
    expect(
      typesOf([
        preset("a", "X", "PLA", "Carbon fiber"),
        preset("b", "X", "PLA", "PLA+ & Pro"),
        preset("c", "X", "PLA", "Standard"),
        preset("d", "X", "PLA", "High speed"),
      ]),
    ).toEqual(["Standard", "PLA+ & Pro", "High speed", "Carbon fiber"]);
  });
});

describe("selection scope", () => {
  it("bulk selection affects only the visible presets in scope", () => {
    const hidden = view({
      filters: filters({
        sources: new Set<SourceKey>(["orca_slicer:factory_system"]),
      }),
    });
    const scope = scopeSelection(hidden, ["PLA"]);
    expect(scope.ids).toEqual(["pla-std", "pla-matte"]);
    expect(scope.state).toBe("none");
    const partial = scopeSelection(
      { ...hidden, selected: new Set(["pla-std"]) },
      ["PLA"],
    );
    expect(partial.state).toBe("mixed");
    const all = scopeSelection(
      { ...hidden, selected: new Set(["pla-std", "pla-matte"]) },
      ["PLA"],
    );
    expect(all.state).toBe("all");
  });

  it("counts selected presets hidden by the filters", () => {
    const filtered = view({
      filters: filters({ hiddenVendors: new Set(["Fiberlogy"]) }),
      selected: new Set(["pla-silk", "pla-std", "abs-gf"]),
    });
    expect(hiddenSelectedCount(filtered)).toBe(2);
  });

  it("show selected only keeps just the selected presets", () => {
    const only = view({
      filters: filters({ selectedOnly: true }),
      selected: new Set(["abs-gf"]),
    });
    expect(viewSources(only, []).map((item) => item.id)).toEqual(["abs-gf"]);
  });

  it("groups presets by manufacturer at the preset level", () => {
    const groups = groupPresets(view({ path: ["PLA", "Matte"] }));
    expect(
      groups.map((group) => [group.title, group.items.map((item) => item.id)]),
    ).toEqual([
      ["Fiberlogy", ["pla-matte-2"]],
      ["Overture", ["pla-matte"]],
    ]);
  });
});

describe("search", () => {
  it("is scoped to the current level and grouped by material and type", () => {
    expect(
      searchGroups(view({ query: "matte" })).map((group) => group.title),
    ).toEqual(["PLA · Matte"]);
    expect(
      searchGroups(view({ query: "standard" })).map((group) => group.title),
    ).toEqual([
      "PLA · Standard",
      "PCTG · Standard",
      "ABS · Standard",
      "PVA · Standard",
      "Other · Standard",
    ]);
    expect(
      searchGroups(view({ query: "standard", path: ["ABS"] })).map(
        (group) => group.title,
      ),
    ).toEqual(["ABS · Standard"]);
    expect(inScope(catalog[0], ["PLA", "Standard"])).toBe(true);
    expect(inScope(catalog[0], ["PLA", "Matte"])).toBe(false);
  });
});

describe("filter menus", () => {
  it("labels the source trigger", () => {
    const label = (keys: SourceKey[]) => sourceTriggerLabel(new Set(keys));
    expect(
      label([
        "orca_slicer:factory_system",
        "orca_slicer:user_custom",
        "bambu_studio:factory_system",
        "bambu_studio:user_custom",
      ]),
    ).toBe("All sources");
    expect(label([])).toBe("None");
    expect(
      label(["orca_slicer:factory_system", "orca_slicer:user_custom"]),
    ).toBe("OrcaSlicer");
    expect(
      label(["bambu_studio:factory_system", "bambu_studio:user_custom"]),
    ).toBe("Bambu Studio");
    expect(
      label(["orca_slicer:factory_system", "bambu_studio:factory_system"]),
    ).toBe("System presets");
    expect(label(["orca_slicer:user_custom", "bambu_studio:user_custom"])).toBe(
      "Your presets",
    );
    expect(label(["orca_slicer:user_custom"])).toBe("1 of 4");
  });

  it("labels the manufacturer trigger", () => {
    const all = ["Fiberlogy", "Northstar", "Overture"];
    expect(vendorTriggerLabel(all, new Set())).toBe("All manufacturers");
    expect(vendorTriggerLabel(all, new Set(all))).toBe("None");
    expect(vendorTriggerLabel(all, new Set(["Fiberlogy", "Northstar"]))).toBe(
      "Overture",
    );
    expect(vendorTriggerLabel(all, new Set(["Fiberlogy"]))).toBe("2 of 3");
  });

  it("scopes manufacturer Select all and Clear all to the searched names", () => {
    const all = ["Fiberlogy", "Northstar", "Overture"];
    expect([...setVisibleVendors(all, new Set(), "o", false)].sort()).toEqual([
      "Fiberlogy",
      "Northstar",
      "Overture",
    ]);
    expect([...setVisibleVendors(all, new Set(), "fib", false)]).toEqual([
      "Fiberlogy",
    ]);
    expect(
      [...setVisibleVendors(all, new Set(all), "over", true)].sort(),
    ).toEqual(["Fiberlogy", "Northstar"]);
  });

  it("explains each empty state", () => {
    const base = view();
    const vendors = new Set(catalog.map((source) => source.vendor));
    expect(
      emptyStateKind({ ...base, filters: filters({ hiddenVendors: vendors }) }),
    ).toBe("no-manufacturers");
    expect(
      emptyStateKind({ ...base, filters: filters({ sources: new Set() }) }),
    ).toBe("no-sources");
    expect(emptyStateKind({ ...base, query: "zzz" })).toBe("no-matches");
    expect(
      emptyStateKind({ ...base, filters: filters({ selectedOnly: true }) }),
    ).toBe("nothing-selected");
    expect(
      emptyStateKind({
        ...base,
        filters: filters({ hiddenStatuses: new Set(["new"]) }),
      }),
    ).toBe("no-presets");
    expect(emptyStateKind(base)).toBeNull();
  });
});

describe("drilldown history", () => {
  it("moves back and forward, stops at the ends, and clears forward on navigation", () => {
    let trail = emptyHistory();
    trail = recordNavigation(trail, [], ["PLA"]);
    trail = recordNavigation(trail, ["PLA"], ["PLA", "Matte"]);
    expect(travel(trail, ["PLA", "Matte"], 1)).toBeNull();

    const back = travel(trail, ["PLA", "Matte"], -1);
    expect(back?.path).toEqual(["PLA"]);
    const back2 = travel(back!.history, back!.path, -1);
    expect(back2?.path).toEqual([]);
    expect(travel(back2!.history, back2!.path, -1)).toBeNull();

    const forward = travel(back2!.history, back2!.path, 1);
    expect(forward?.path).toEqual(["PLA"]);
    const branched = recordNavigation(
      forward!.history,
      ["PLA"],
      ["ABS", "Standard"],
    );
    expect(travel(branched, ["ABS", "Standard"], 1)).toBeNull();
    expect(travel(branched, ["ABS", "Standard"], -1)?.path).toEqual(["PLA"]);
  });

  it("ignores navigation that stays on the same level", () => {
    const trail = recordNavigation(emptyHistory(), ["PLA"], ["PLA"]);
    expect(trail.back).toEqual([]);
  });
});
