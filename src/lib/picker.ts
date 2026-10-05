import {
  SOURCE_KEYS,
  passesSourceFilters,
  type SourceFilters,
  type SourceKey,
} from "./state";
import type { CatalogSource } from "./types";

/** [] for all materials, [material], or [material, type]. */
export type PickerPath = string[];

export interface PickerView {
  sources: CatalogSource[];
  filters: SourceFilters;
  selected: Set<string>;
  path: PickerPath;
  query: string;
}

export type CardLayout = "tiered" | "rows" | "single";

export interface TypeSummary {
  type: string;
  count: number;
  selected: number;
  manufacturerNames: string[];
}

export interface MaterialCard {
  id: string;
  label: string;
  count: number;
  manufacturers: number;
  manufacturerNames: string[];
  selected: number;
  types: string[];
  typeRows: TypeSummary[];
  layout: CardLayout;
}

export interface PresetGroup {
  key: string;
  title: string;
  items: CatalogSource[];
}

export type SelectionState = "none" | "mixed" | "all";

export type EmptyStateKind =
  | "no-manufacturers"
  | "no-sources"
  | "no-matches"
  | "nothing-selected"
  | "no-presets";

/** Browse order from the design; materials found in real data follow. */
export const MATERIAL_ORDER = [
  "PLA",
  "PETG",
  "PCTG",
  "ABS",
  "ASA",
  "TPU",
  "PA",
  "PC",
] as const;

const MATERIAL_LABELS: Record<string, string> = { PA: "PA · Nylon" };

const TYPE_ORDER = [
  "Standard",
  "+ & Pro",
  "Matte",
  "Silk",
  "High speed",
  "Shore",
  "Aero",
  "For AMS",
  "Carbon fiber",
  "Glass fiber",
];

export const SOURCE_OPTIONS: ReadonlyArray<{
  key: SourceKey;
  group: "OrcaSlicer" | "Bambu Studio";
  label: string;
}> = [
  {
    key: "orca_slicer:factory_system",
    group: "OrcaSlicer",
    label: "System presets",
  },
  {
    key: "orca_slicer:user_custom",
    group: "OrcaSlicer",
    label: "Your presets",
  },
  {
    key: "bambu_studio:factory_system",
    group: "Bambu Studio",
    label: "System presets",
  },
  {
    key: "bambu_studio:user_custom",
    group: "Bambu Studio",
    label: "Your presets",
  },
];

export function materialLabel(id: string): string {
  return MATERIAL_LABELS[id] ?? id;
}

function typeRank(type: string): number {
  if (type.endsWith("+ & Pro")) return TYPE_ORDER.indexOf("+ & Pro");
  if (type.startsWith("Shore ")) return TYPE_ORDER.indexOf("Shore");
  const rank = TYPE_ORDER.indexOf(type);
  return rank < 0 ? TYPE_ORDER.length : rank;
}

function compareTypes(left: string, right: string): number {
  return typeRank(left) - typeRank(right) || left.localeCompare(right);
}

function materialRank(id: string): number {
  const known = MATERIAL_ORDER.indexOf(id as (typeof MATERIAL_ORDER)[number]);
  if (known >= 0) return known;
  return id === "Other" ? MATERIAL_ORDER.length + 1 : MATERIAL_ORDER.length;
}

function compareMaterials(left: string, right: string): number {
  return materialRank(left) - materialRank(right) || left.localeCompare(right);
}

function unique(values: string[]): string[] {
  return [...new Set(values)];
}

function sortedNames(values: string[]): string[] {
  return unique(values).sort((left, right) => left.localeCompare(right));
}

export function typesOf(items: CatalogSource[]): string[] {
  return unique(items.map((item) => item.material_type)).sort(compareTypes);
}

export function vendorsOf(sources: CatalogSource[]): string[] {
  return sortedNames(sources.map((source) => source.vendor));
}

function matchesQuery(source: CatalogSource, query: string): boolean {
  const needle = query.trim().toLocaleLowerCase();
  if (!needle) return true;
  return [
    source.name,
    source.material,
    materialLabel(source.material_archetype),
    source.material_type,
  ]
    .join(" ")
    .toLocaleLowerCase()
    .includes(needle);
}

export function inScope(source: CatalogSource, path: PickerPath): boolean {
  return (
    (!path[0] || source.material_archetype === path[0]) &&
    (!path[1] || source.material_type === path[1])
  );
}

/** Presets that pass the filters, Show selected only, search, and the path. */
export function viewSources(
  view: PickerView,
  path: PickerPath = view.path,
): CatalogSource[] {
  return view.sources.filter(
    (source) =>
      passesSourceFilters(source, view.filters) &&
      (!view.filters.selectedOnly || view.selected.has(source.id)) &&
      matchesQuery(source, view.query) &&
      inScope(source, path),
  );
}

function selectedIn(view: PickerView, items: CatalogSource[]): number {
  return items.filter((item) => view.selected.has(item.id)).length;
}

function typeSummary(
  view: PickerView,
  items: CatalogSource[],
  type: string,
): TypeSummary {
  const typed = items.filter((item) => item.material_type === type);
  return {
    type,
    count: typed.length,
    selected: selectedIn(view, typed),
    manufacturerNames: vendorsOf(typed),
  };
}

export function materialCards(view: PickerView): MaterialCard[] {
  const visible = viewSources(view, []);
  return unique(visible.map((source) => source.material_archetype))
    .sort(compareMaterials)
    .map((id) => {
      const items = visible.filter((item) => item.material_archetype === id);
      const types = typesOf(items);
      const manufacturerNames = vendorsOf(items);
      return {
        id,
        label: materialLabel(id),
        count: items.length,
        manufacturers: manufacturerNames.length,
        manufacturerNames,
        selected: selectedIn(view, items),
        types,
        typeRows: types.map((type) => typeSummary(view, items, type)),
        layout:
          types.length >= 3 ? "tiered" : types.length === 2 ? "rows" : "single",
      };
    });
}

export function typeCards(view: PickerView): TypeSummary[] {
  const items = viewSources(view, view.path.slice(0, 1));
  return typesOf(items).map((type) => typeSummary(view, items, type));
}

export function isTiered(view: PickerView, material: string): boolean {
  return typesOf(viewSources(view, [material])).length >= 3;
}

/**
 * Applies the tiering rule to a requested path. A material with one type opens
 * its presets; a material listed as rows has no type tier, so going back to it
 * returns to all materials. History travel passes `exact`.
 */
export function normalizePath(
  view: PickerView,
  path: PickerPath,
  motion: "forward" | "back",
  exact = false,
): PickerPath {
  const next = path.filter(Boolean).slice(0, 2);
  if (exact || next.length !== 1) return next;
  const types = typesOf(viewSources(view, next));
  if (types.length === 1) return [next[0], types[0]];
  if (types.length === 2 && motion === "back") return [];
  return next;
}

export function scopeSelection(
  view: PickerView,
  path: PickerPath,
): { ids: string[]; selected: number; state: SelectionState } {
  const ids = viewSources(view, path).map((source) => source.id);
  const selected = ids.filter((id) => view.selected.has(id)).length;
  return {
    ids,
    selected,
    state: selected === 0 ? "none" : selected === ids.length ? "all" : "mixed",
  };
}

export function hiddenSelectedCount(view: PickerView): number {
  return view.sources.filter(
    (source) =>
      view.selected.has(source.id) &&
      !passesSourceFilters(source, view.filters),
  ).length;
}

function byName(left: CatalogSource, right: CatalogSource): number {
  return left.name.localeCompare(right.name);
}

export function groupPresets(view: PickerView): PresetGroup[] {
  const items = viewSources(view);
  return vendorsOf(items).map((vendor) => ({
    key: `vendor:${vendor}`,
    title: vendor,
    items: items.filter((item) => item.vendor === vendor).sort(byName),
  }));
}

export function searchGroups(view: PickerView): PresetGroup[] {
  const items = viewSources(view);
  const groups = new Map<string, CatalogSource[]>();
  const ordered = [...items].sort(
    (left, right) =>
      compareMaterials(left.material_archetype, right.material_archetype) ||
      compareTypes(left.material_type, right.material_type) ||
      byName(left, right),
  );
  for (const item of ordered) {
    const title = `${materialLabel(item.material_archetype)} · ${item.material_type}`;
    groups.set(title, [...(groups.get(title) ?? []), item]);
  }
  return [...groups].map(([title, list]) => ({
    key: `search:${title}`,
    title,
    items: list,
  }));
}

export function emptyStateKind(view: PickerView): EmptyStateKind | null {
  if (viewSources(view).length > 0) return null;
  const vendors = vendorsOf(view.sources);
  if (
    vendors.length > 0 &&
    vendors.every((vendor) => view.filters.hiddenVendors.has(vendor))
  ) {
    return "no-manufacturers";
  }
  if (view.filters.sources.size === 0) return "no-sources";
  if (view.query.trim()) return "no-matches";
  if (view.filters.selectedOnly) return "nothing-selected";
  return "no-presets";
}

export function sourceTriggerLabel(sources: Set<SourceKey>): string {
  if (sources.size === SOURCE_KEYS.length) return "All sources";
  if (sources.size === 0) return "None";
  const has = (...keys: SourceKey[]) =>
    sources.size === keys.length && keys.every((key) => sources.has(key));
  if (has("orca_slicer:factory_system", "orca_slicer:user_custom"))
    return "OrcaSlicer";
  if (has("bambu_studio:factory_system", "bambu_studio:user_custom"))
    return "Bambu Studio";
  if (has("orca_slicer:factory_system", "bambu_studio:factory_system"))
    return "System presets";
  if (has("orca_slicer:user_custom", "bambu_studio:user_custom"))
    return "Your presets";
  return `${sources.size} of ${SOURCE_KEYS.length}`;
}

/** Trigger text for a checklist that stores its unchecked values. */
export function checklistTriggerLabel(
  values: string[],
  hidden: Set<string>,
  allLabel: string,
  name: (value: string) => string = (value) => value,
): string {
  const shown = values.filter((value) => !hidden.has(value));
  if (shown.length === values.length) return allLabel;
  if (shown.length === 0) return "None";
  if (shown.length === 1) return name(shown[0]);
  return `${shown.length} of ${values.length}`;
}

export function vendorTriggerLabel(
  vendors: string[],
  hidden: Set<string>,
): string {
  return checklistTriggerLabel(vendors, hidden, "All manufacturers");
}

/** Select all / Clear all for the values the menu search currently shows. */
export function setVisibleVendors(
  vendors: string[],
  hidden: Set<string>,
  query: string,
  checked: boolean,
): Set<string> {
  const needle = query.trim().toLocaleLowerCase();
  const next = new Set(hidden);
  for (const vendor of vendors) {
    if (!vendor.toLocaleLowerCase().includes(needle)) continue;
    if (checked) next.delete(vendor);
    else next.add(vendor);
  }
  return next;
}

/** A copy of `values` with `value` present or absent. */
export function withValue<T>(
  values: Set<T>,
  value: T,
  present: boolean,
): Set<T> {
  return new Set(
    present ? [...values, value] : [...values].filter((item) => item !== value),
  );
}

/** Select all / Clear all for any checklist that stores unchecked values. */
export const setShownValues = setVisibleVendors;

export interface DrillHistory {
  back: PickerPath[];
  forward: PickerPath[];
}

export function emptyHistory(): DrillHistory {
  return { back: [], forward: [] };
}

function samePath(left: PickerPath, right: PickerPath): boolean {
  return left.join("/") === right.join("/");
}

/** A new navigation pushes the old level to Back and clears Forward. */
export function recordNavigation(
  history: DrillHistory,
  from: PickerPath,
  to: PickerPath,
): DrillHistory {
  if (samePath(from, to)) return history;
  return { back: [...history.back, [...from]], forward: [] };
}

export function travel(
  history: DrillHistory,
  current: PickerPath,
  direction: -1 | 1,
): { history: DrillHistory; path: PickerPath } | null {
  const from = direction < 0 ? history.back : history.forward;
  if (!from.length) return null;
  const path = from[from.length - 1];
  const remaining = from.slice(0, -1);
  return direction < 0
    ? {
        path,
        history: {
          back: remaining,
          forward: [...history.forward, [...current]],
        },
      }
    : {
        path,
        history: { back: [...history.back, [...current]], forward: remaining },
      };
}
