import type {
  CatalogSource,
  DiscoveryResponse,
  EvidenceLevel,
  LocalRunResult,
  MigrationPlan,
  MigrationStatus,
  NozzleSelection,
  OperationState,
  PrinterTarget,
  SourceApp,
  SourceKind,
} from "./types";

export type Theme = "system" | "light" | "dark";
export type AppPhase =
  | "idle"
  | "discovering"
  | "ready"
  | "planning"
  | "executing"
  | "complete"
  | "error";

export interface SourceFilters {
  search: string;
  sourceApps: Set<SourceApp>;
  sourceKinds: Set<SourceKind>;
  vendors: Set<string>;
  materials: Set<string>;
  families: Set<string>;
  variants: Set<string>;
  compatiblePrinters: Set<string>;
  migrationStatuses: Set<MigrationStatus>;
  selectedOnly: boolean;
}

export interface OperationProgress {
  evidence: EvidenceLevel;
  state: OperationState;
}

export interface AppState {
  phase: AppPhase;
  theme: Theme;
  discovery: DiscoveryResponse | null;
  selectedAccountId: string | null;
  sourceCatalogId: string | null;
  targetCatalogId: string | null;
  sources: CatalogSource[];
  printers: PrinterTarget[];
  selectedSourceIds: Set<string>;
  selectedNozzles: Record<string, Set<string>>;
  showCustomPrinters: boolean;
  filters: SourceFilters;
  presetTemplate: string;
  amsTemplate: string;
  plan: MigrationPlan | null;
  progress: Record<string, OperationProgress>;
  result: LocalRunResult | null;
  error: string | null;
}

export type AppAction =
  | { type: "discovery_started" }
  | { type: "discovery_loaded"; discovery: DiscoveryResponse }
  | { type: "failed"; message: string }
  | { type: "theme_changed"; theme: Theme }
  | { type: "account_selected"; accountId: string }
  | { type: "sources_loaded"; catalogId: string; sources: CatalogSource[] }
  | { type: "targets_loaded"; catalogId: string; printers: PrinterTarget[] }
  | { type: "source_toggled"; sourceId: string }
  | { type: "filters_changed"; filters: Partial<SourceFilters> }
  | { type: "select_visible"; selected: boolean }
  | { type: "clear_selection" }
  | { type: "show_custom_changed"; value: boolean }
  | { type: "nozzle_toggled"; printerId: string; diameter: string }
  | { type: "select_printer_nozzles"; printerId: string; selected: boolean }
  | { type: "templates_changed"; preset: string; ams: string }
  | { type: "planning_started" }
  | { type: "plan_built"; plan: MigrationPlan }
  | { type: "execution_started" }
  | {
      type: "progress_updated";
      operationId: string;
      evidence: EvidenceLevel;
      state: OperationState;
    }
  | { type: "execution_completed"; result: LocalRunResult }
  | { type: "ams_verified"; operationIds: string[] };

const DEFAULT_PRESET_TEMPLATE = "{clean_name} - {printer_code}";
const DEFAULT_AMS_TEMPLATE = "{vendor} {material} {clean_name}";

function emptyFilters(): SourceFilters {
  return {
    search: "",
    sourceApps: new Set(),
    sourceKinds: new Set(),
    vendors: new Set(),
    materials: new Set(),
    families: new Set(),
    variants: new Set(),
    compatiblePrinters: new Set(),
    migrationStatuses: new Set(),
    selectedOnly: false,
  };
}

export function createInitialState(theme: Theme = "system"): AppState {
  return {
    phase: "idle",
    theme,
    discovery: null,
    selectedAccountId: null,
    sourceCatalogId: null,
    targetCatalogId: null,
    sources: [],
    printers: [],
    selectedSourceIds: new Set(),
    selectedNozzles: {},
    showCustomPrinters: false,
    filters: emptyFilters(),
    presetTemplate: DEFAULT_PRESET_TEMPLATE,
    amsTemplate: DEFAULT_AMS_TEMPLATE,
    plan: null,
    progress: {},
    result: null,
    error: null,
  };
}

function invalidatePlan(state: AppState): AppState {
  return { ...state, plan: null, result: null, progress: {} };
}

export function toggleSetValue<T>(values: Set<T>, value: T): Set<T> {
  const next = new Set(values);
  if (next.has(value)) next.delete(value);
  else next.add(value);
  return next;
}

export function removeSetValue<T>(values: Set<T>, value: T): Set<T> {
  const next = new Set(values);
  next.delete(value);
  return next;
}

export function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "discovery_started":
      return { ...state, phase: "discovering", error: null };
    case "discovery_loaded": {
      const selectedAccount = action.discovery.accounts.find(
        (account) =>
          account.id === state.selectedAccountId &&
          account.eligibility === "eligible",
      );
      return {
        ...state,
        phase: "ready",
        discovery: action.discovery,
        selectedAccountId:
          selectedAccount?.id ??
          action.discovery.accounts.find(
            (account) => account.eligibility === "eligible",
          )?.id ??
          null,
        error: null,
      };
    }
    case "failed":
      return { ...state, phase: "error", error: action.message };
    case "theme_changed":
      return { ...state, theme: action.theme };
    case "account_selected": {
      const account = state.discovery?.accounts.find(
        (item) => item.id === action.accountId,
      );
      if (account && account.eligibility !== "eligible") {
        return {
          ...state,
          error: `Account ${account.id} is not eligible for migration.`,
        };
      }
      return invalidatePlan({
        ...state,
        selectedAccountId: action.accountId,
        error: null,
      });
    }
    case "sources_loaded":
      return invalidatePlan({
        ...state,
        sourceCatalogId: action.catalogId,
        sources: action.sources,
        phase: "ready",
        error: null,
      });
    case "targets_loaded":
      return invalidatePlan({
        ...state,
        targetCatalogId: action.catalogId,
        printers: action.printers,
        phase: "ready",
        error: null,
      });
    case "source_toggled":
      return invalidatePlan({
        ...state,
        selectedSourceIds: toggleSetValue(
          state.selectedSourceIds,
          action.sourceId,
        ),
      });
    case "filters_changed":
      return { ...state, filters: { ...state.filters, ...action.filters } };
    case "select_visible": {
      const selectedSourceIds = new Set(state.selectedSourceIds);
      for (const source of visibleSources(state)) {
        if (action.selected) selectedSourceIds.add(source.id);
        else selectedSourceIds.delete(source.id);
      }
      return invalidatePlan({ ...state, selectedSourceIds });
    }
    case "clear_selection":
      return invalidatePlan({ ...state, selectedSourceIds: new Set() });
    case "show_custom_changed":
      return invalidatePlan({ ...state, showCustomPrinters: action.value });
    case "nozzle_toggled": {
      const selectedNozzles = { ...state.selectedNozzles };
      selectedNozzles[action.printerId] = toggleSetValue(
        selectedNozzles[action.printerId] ?? new Set(),
        action.diameter,
      );
      return invalidatePlan({ ...state, selectedNozzles });
    }
    case "select_printer_nozzles": {
      const printer = state.printers.find(
        (item) => item.id === action.printerId,
      );
      if (!printer) return state;
      const selectedNozzles = { ...state.selectedNozzles };
      selectedNozzles[action.printerId] = action.selected
        ? new Set(
            printer.nozzles
              .filter((item) => item.supported)
              .map((item) => item.diameter),
          )
        : new Set();
      return invalidatePlan({ ...state, selectedNozzles });
    }
    case "templates_changed":
      return invalidatePlan({
        ...state,
        presetTemplate: action.preset,
        amsTemplate: action.ams,
      });
    case "planning_started":
      return { ...state, phase: "planning", error: null };
    case "plan_built":
      return {
        ...state,
        phase: "ready",
        plan: action.plan,
        progress: {},
        error: null,
      };
    case "execution_started":
      return { ...state, phase: "executing", error: null };
    case "progress_updated":
      return {
        ...state,
        progress: {
          ...state.progress,
          [action.operationId]: {
            evidence: action.evidence,
            state: action.state,
          },
        },
      };
    case "execution_completed":
      return {
        ...state,
        phase: "complete",
        result: action.result,
        error: null,
      };
    case "ams_verified": {
      const progress = { ...state.progress };
      for (const operationId of action.operationIds) {
        progress[operationId] = {
          evidence: "ams_verified",
          state: "ams_verified",
        };
      }
      return { ...state, progress };
    }
  }
}

function includesCaseInsensitive(
  values: Set<string>,
  candidate: string,
): boolean {
  if (values.size === 0) return true;
  const normalized = candidate.toLocaleLowerCase();
  return [...values].some((value) => value.toLocaleLowerCase() === normalized);
}

export interface FilterFacet {
  value: string;
  count: number;
}

export interface SourceFacets {
  vendors: FilterFacet[];
  materials: FilterFacet[];
  families: FilterFacet[];
  variants: FilterFacet[];
  compatiblePrinters: FilterFacet[];
}

export interface ActiveFilterChip {
  id: string;
  label: string;
  dimension: keyof SourceFilters;
  value: string;
}

export function sourceFacets(sources: CatalogSource[]): SourceFacets {
  const facet = (values: string[]): FilterFacet[] => {
    const counts = new Map<string, number>();
    for (const value of values.filter(Boolean))
      counts.set(value, (counts.get(value) ?? 0) + 1);
    return [...counts]
      .map(([value, count]) => ({ value, count }))
      .sort((left, right) => left.value.localeCompare(right.value));
  };
  return {
    vendors: facet(sources.map((source) => source.vendor)),
    materials: facet(sources.map((source) => source.material)),
    families: facet(sources.map((source) => source.family)),
    variants: facet(sources.map((source) => source.variant)),
    compatiblePrinters: facet(
      sources.flatMap((source) => source.compatible_printers),
    ),
  };
}

const CHIP_LABELS: Record<string, string> = {
  orca_slicer: "OrcaSlicer",
  bambu_studio: "Bambu Studio",
  factory_system: "Factory / system",
  user_custom: "User / custom",
  already_migrated: "Already migrated",
  new: "New",
  incomplete: "Incomplete",
  conflicting: "Conflicting",
  unsupported: "Unsupported",
};

export function activeFilterChips(filters: SourceFilters): ActiveFilterChip[] {
  const chips: ActiveFilterChip[] = [];
  if (filters.search) {
    chips.push({
      id: "search",
      label: `Search: ${filters.search}`,
      dimension: "search",
      value: "",
    });
  }
  for (const dimension of [
    "sourceApps",
    "sourceKinds",
    "vendors",
    "materials",
    "families",
    "variants",
    "compatiblePrinters",
    "migrationStatuses",
  ] as const) {
    for (const value of filters[dimension]) {
      chips.push({
        id: `${dimension}:${value}`,
        label: CHIP_LABELS[value] ?? value,
        dimension,
        value,
      });
    }
  }
  if (filters.selectedOnly) {
    chips.push({
      id: "selectedOnly",
      label: "Selected only",
      dimension: "selectedOnly",
      value: "true",
    });
  }
  return chips;
}

export function resetFilters(): SourceFilters {
  return emptyFilters();
}

export function visibleSources(state: AppState): CatalogSource[] {
  const filters = state.filters;
  const search = filters.search.trim().toLocaleLowerCase();
  return state.sources.filter((source) => {
    if (filters.selectedOnly && !state.selectedSourceIds.has(source.id))
      return false;
    if (filters.sourceApps.size && !filters.sourceApps.has(source.source_app))
      return false;
    if (
      filters.sourceKinds.size &&
      !filters.sourceKinds.has(source.source_kind)
    )
      return false;
    if (!includesCaseInsensitive(filters.vendors, source.vendor)) return false;
    if (!includesCaseInsensitive(filters.materials, source.material))
      return false;
    if (!includesCaseInsensitive(filters.families, source.family)) return false;
    if (!includesCaseInsensitive(filters.variants, source.variant))
      return false;
    if (
      filters.migrationStatuses.size &&
      !filters.migrationStatuses.has(source.migration_status)
    ) {
      return false;
    }
    if (
      filters.compatiblePrinters.size &&
      !source.compatible_printers.some((printer) =>
        includesCaseInsensitive(filters.compatiblePrinters, printer),
      )
    ) {
      return false;
    }
    return (
      !search ||
      [
        source.name,
        source.vendor,
        source.material,
        source.family,
        source.variant,
      ].some((value) => value.toLocaleLowerCase().includes(search))
    );
  });
}

export function selectedNozzleRequests(state: AppState): NozzleSelection[] {
  return Object.entries(state.selectedNozzles)
    .map(([printer_id, values]) => ({
      printer_id,
      diameters: [...values].sort(numericSort),
    }))
    .filter((selection) => selection.diameters.length > 0)
    .sort((left, right) => left.printer_id.localeCompare(right.printer_id));
}

function numericSort(left: string, right: string): number {
  return Number(left) - Number(right);
}

export interface PreferencesStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function loadTheme(storage: PreferencesStorage): Theme {
  const value = storage.getItem("bfm.theme");
  return value === "light" || value === "dark" || value === "system"
    ? value
    : "system";
}

export function persistTheme(storage: PreferencesStorage, theme: Theme): void {
  storage.setItem("bfm.theme", theme);
}
