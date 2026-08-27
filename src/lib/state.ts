import type {
  CatalogSource,
  ConflictDecision,
  DiscoveryResponse,
  EvidenceLevel,
  ExecutionPhase,
  LocalRunResult,
  MigrationPlan,
  MigrationStatus,
  NameOverride,
  NamingPreset,
  NamingRule,
  NozzleSelection,
  OperationState,
  OutputSelection,
  PrinterTarget,
  SourceApp,
  SourceKind,
  SyncPhase,
  SyncResult,
} from "./types";

export type Theme = "system" | "light" | "dark";
export type AppPhase =
  | "idle"
  | "discovering"
  | "ready"
  | "planning"
  | "executing"
  | "synchronizing"
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

export interface WorkspacePreferences {
  setupComplete: boolean;
  enabledPrinterIds: string[];
  selectedAccountId: string | null;
  selectedSourceIds: string[];
  selectedNozzles: Record<string, string[]>;
  filters: SourceFilters;
  presetTemplate: string;
  amsTemplate: string;
  presetRules: NamingRule[];
  amsRules: NamingRule[];
}

export interface FilterPreset {
  id: string;
  name: string;
  filters: SourceFilters;
}

export interface OperationProgress {
  evidence: EvidenceLevel;
  state: OperationState;
}

export interface AppState {
  phase: AppPhase;
  theme: Theme;
  discovery: DiscoveryResponse | null;
  setupComplete: boolean;
  enabledPrinterIds: Set<string>;
  selectedAccountId: string | null;
  sourceCatalogId: string | null;
  targetCatalogId: string | null;
  sources: CatalogSource[];
  printers: PrinterTarget[];
  selectedSourceIds: Set<string>;
  selectedNozzles: Record<string, Set<string>>;
  filters: SourceFilters;
  presetTemplate: string;
  amsTemplate: string;
  presetRules: NamingRule[];
  amsRules: NamingRule[];
  outputs: OutputSelection;
  nameOverrides: NameOverride[];
  conflictDecisions: ConflictDecision[];
  plan: MigrationPlan | null;
  executionPhase: ExecutionPhase | null;
  progress: Record<string, OperationProgress>;
  result: LocalRunResult | null;
  synchronization: SyncResult | null;
  syncPhase: SyncPhase | null;
  syncError: string | null;
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
  | {
      type: "setup_completed";
      sourceApps: Set<SourceApp>;
      sourceKinds: Set<SourceKind>;
      enabledPrinterIds: Set<string>;
      selectedNozzles: Record<string, Set<string>>;
    }
  | {
      type: "enabled_printers_changed";
      enabledPrinterIds: Set<string>;
      selectedNozzles: Record<string, Set<string>>;
    }
  | { type: "source_toggled"; sourceId: string }
  | { type: "sources_removed"; sourceIds: Set<string> }
  | { type: "filters_changed"; filters: Partial<SourceFilters> }
  | { type: "select_visible"; selected: boolean }
  | { type: "clear_selection" }
  | { type: "nozzle_toggled"; printerId: string; diameter: string }
  | { type: "select_printer_nozzles"; printerId: string; selected: boolean }
  | { type: "templates_changed"; preset: string; ams: string }
  | { type: "rules_changed"; presetRules: NamingRule[]; amsRules: NamingRule[] }
  | { type: "outputs_changed"; outputs: OutputSelection }
  | { type: "name_overrides_changed"; overrides: NameOverride[] }
  | { type: "conflict_decisions_changed"; decisions: ConflictDecision[] }
  | { type: "planning_started" }
  | { type: "planning_stopped" }
  | { type: "plan_built"; plan: MigrationPlan }
  | { type: "execution_started" }
  | { type: "execution_phase"; phase: ExecutionPhase }
  | {
      type: "progress_updated";
      operationId: string;
      evidence: EvidenceLevel;
      state: OperationState;
    }
  | { type: "execution_completed"; result: LocalRunResult }
  | { type: "synchronization_started" }
  | { type: "synchronization_phase"; phase: SyncPhase }
  | { type: "synchronization_completed"; result: SyncResult }
  | { type: "synchronization_failed"; message: string }
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

export function createInitialState(
  theme: Theme = "system",
  preferences: WorkspacePreferences | null = null,
): AppState {
  return {
    phase: "idle",
    theme,
    discovery: null,
    setupComplete: preferences?.setupComplete ?? false,
    enabledPrinterIds: new Set(preferences?.enabledPrinterIds ?? []),
    selectedAccountId: preferences?.selectedAccountId ?? null,
    sourceCatalogId: null,
    targetCatalogId: null,
    sources: [],
    printers: [],
    selectedSourceIds: new Set(preferences?.selectedSourceIds ?? []),
    selectedNozzles: Object.fromEntries(
      Object.entries(preferences?.selectedNozzles ?? {}).map(
        ([printer, nozzles]) => [printer, new Set(nozzles)],
      ),
    ),
    filters: preferences?.filters ?? emptyFilters(),
    presetTemplate: preferences?.presetTemplate ?? DEFAULT_PRESET_TEMPLATE,
    amsTemplate: preferences?.amsTemplate ?? DEFAULT_AMS_TEMPLATE,
    presetRules: preferences?.presetRules ?? [],
    amsRules: preferences?.amsRules ?? [],
    outputs: { slicing_presets: true, custom_filaments: true },
    nameOverrides: [],
    conflictDecisions: [],
    plan: null,
    executionPhase: null,
    progress: {},
    result: null,
    synchronization: null,
    syncPhase: null,
    syncError: null,
    error: null,
  };
}

function invalidatePlan(
  state: AppState,
  preserveConflictDecisions = false,
): AppState {
  return {
    ...state,
    plan: null,
    conflictDecisions: preserveConflictDecisions ? state.conflictDecisions : [],
    executionPhase: null,
    result: null,
    progress: {},
    synchronization: null,
    syncPhase: null,
    syncError: null,
  };
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

function sanitizePrinterSelection(
  printers: PrinterTarget[],
  requestedEnabled: Set<string>,
  requestedNozzles: Record<string, Set<string>>,
): {
  enabledPrinterIds: Set<string>;
  selectedNozzles: Record<string, Set<string>>;
} {
  const enabledPrinterIds = new Set<string>();
  const selectedNozzles: Record<string, Set<string>> = {};
  for (const printer of printers) {
    if (printer.kind !== "official" || !requestedEnabled.has(printer.id))
      continue;
    const supported = new Set(
      printer.nozzles
        .filter((nozzle) => nozzle.supported)
        .map((nozzle) => nozzle.diameter),
    );
    const nozzles = new Set(
      [...(requestedNozzles[printer.id] ?? [])].filter((diameter) =>
        supported.has(diameter),
      ),
    );
    enabledPrinterIds.add(printer.id);
    if (nozzles.size > 0) selectedNozzles[printer.id] = nozzles;
  }
  return { enabledPrinterIds, selectedNozzles };
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
    case "sources_loaded": {
      const available = new Set(action.sources.map((source) => source.id));
      return invalidatePlan({
        ...state,
        sourceCatalogId: action.catalogId,
        sources: action.sources,
        selectedSourceIds: new Set(
          [...state.selectedSourceIds].filter((id) => available.has(id)),
        ),
        phase: "ready",
        error: null,
      });
    }
    case "targets_loaded": {
      const selection = sanitizePrinterSelection(
        action.printers,
        state.enabledPrinterIds,
        state.selectedNozzles,
      );
      return invalidatePlan({
        ...state,
        targetCatalogId: action.catalogId,
        printers: action.printers,
        ...selection,
        phase: "ready",
        error: null,
      });
    }
    case "setup_completed": {
      const selection = sanitizePrinterSelection(
        state.printers,
        action.enabledPrinterIds,
        action.selectedNozzles,
      );
      return invalidatePlan({
        ...state,
        setupComplete: true,
        filters: {
          ...state.filters,
          sourceApps: new Set(action.sourceApps),
          sourceKinds: new Set(action.sourceKinds),
        },
        ...selection,
      });
    }
    case "enabled_printers_changed":
      return invalidatePlan({
        ...state,
        ...sanitizePrinterSelection(
          state.printers,
          action.enabledPrinterIds,
          action.selectedNozzles,
        ),
      });
    case "source_toggled":
      return invalidatePlan({
        ...state,
        selectedSourceIds: toggleSetValue(
          state.selectedSourceIds,
          action.sourceId,
        ),
      });
    case "sources_removed":
      return invalidatePlan({
        ...state,
        selectedSourceIds: new Set(
          [...state.selectedSourceIds].filter(
            (id) => !action.sourceIds.has(id),
          ),
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
    case "nozzle_toggled": {
      const supported =
        state.enabledPrinterIds.has(action.printerId) &&
        state.printers
          .find((printer) => printer.id === action.printerId)
          ?.nozzles.some(
            (nozzle) => nozzle.diameter === action.diameter && nozzle.supported,
          );
      if (!supported) return state;
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
      if (!printer || !state.enabledPrinterIds.has(printer.id)) return state;
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
    case "rules_changed":
      return invalidatePlan({
        ...state,
        presetRules: action.presetRules,
        amsRules: action.amsRules,
      });
    case "outputs_changed":
      return invalidatePlan({ ...state, outputs: action.outputs });
    case "name_overrides_changed":
      return invalidatePlan({ ...state, nameOverrides: action.overrides });
    case "conflict_decisions_changed":
      return invalidatePlan(
        { ...state, conflictDecisions: action.decisions },
        true,
      );
    case "planning_started":
      return { ...state, phase: "planning", error: null };
    case "planning_stopped":
      return { ...state, phase: "ready", error: null };
    case "plan_built":
      return {
        ...state,
        phase: "ready",
        plan: action.plan,
        progress: {},
        error: null,
      };
    case "execution_started":
      return {
        ...state,
        phase: "executing",
        progress: {},
        executionPhase: null,
        result: null,
        synchronization: null,
        syncPhase: null,
        syncError: null,
        error: null,
      };
    case "execution_phase":
      return { ...state, executionPhase: action.phase };
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
    case "synchronization_started":
      return {
        ...state,
        phase: "synchronizing",
        synchronization: null,
        syncPhase: null,
        syncError: null,
      };
    case "synchronization_phase":
      return { ...state, syncPhase: action.phase };
    case "synchronization_completed": {
      const progress = { ...state.progress };
      for (const observation of action.result.observations) {
        progress[observation.operation_id] = {
          evidence: observation.evidence,
          state: observation.state,
        };
      }
      return {
        ...state,
        phase: "complete",
        progress,
        synchronization: action.result,
        syncPhase: "finished",
        syncError: null,
      };
    }
    case "synchronization_failed":
      return {
        ...state,
        phase: "complete",
        syncPhase: null,
        syncError: action.message,
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
  const official = new Set(
    state.printers
      .filter((printer) => printer.kind === "official")
      .map((printer) => printer.id),
  );
  return Object.entries(state.selectedNozzles)
    .filter(
      ([printerId]) =>
        state.enabledPrinterIds.has(printerId) && official.has(printerId),
    )
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

export function loadWorkspacePreferences(
  storage: PreferencesStorage,
): WorkspacePreferences | null {
  const value = storage.getItem("bfm.workspace");
  if (!value) return null;
  try {
    const parsed = JSON.parse(value) as Record<string, unknown>;
    if (!parsed || (parsed.version !== 1 && parsed.version !== 2)) return null;
    const selectedSourceIds = stringArray(parsed.selected_source_ids);
    const selectedNozzles = stringArrayRecord(parsed.selected_nozzles);
    const filters = deserializeFilters(parsed.filters);
    if (!selectedSourceIds || !selectedNozzles || !filters) return null;
    const enabledPrinterIds =
      parsed.version === 1
        ? Object.keys(selectedNozzles)
        : stringArray(parsed.enabled_printer_ids);
    const setupComplete =
      parsed.version === 1
        ? false
        : typeof parsed.setup_complete === "boolean"
          ? parsed.setup_complete
          : null;
    if (!enabledPrinterIds || setupComplete === null) return null;
    const presetRules =
      Array.isArray(parsed.preset_rules) &&
      parsed.preset_rules.every(isNamingRule)
        ? parsed.preset_rules
        : [];
    const amsRules =
      Array.isArray(parsed.ams_rules) && parsed.ams_rules.every(isNamingRule)
        ? parsed.ams_rules
        : [];
    return {
      setupComplete,
      enabledPrinterIds,
      selectedAccountId:
        typeof parsed.selected_account_id === "string"
          ? parsed.selected_account_id
          : null,
      selectedSourceIds,
      selectedNozzles,
      filters,
      presetTemplate:
        typeof parsed.preset_template === "string"
          ? parsed.preset_template
          : DEFAULT_PRESET_TEMPLATE,
      amsTemplate:
        typeof parsed.ams_template === "string"
          ? parsed.ams_template
          : DEFAULT_AMS_TEMPLATE,
      presetRules,
      amsRules,
    };
  } catch {
    return null;
  }
}

export function persistWorkspacePreferences(
  storage: PreferencesStorage,
  state: AppState,
): void {
  storage.setItem(
    "bfm.workspace",
    JSON.stringify({
      version: 2,
      setup_complete: state.setupComplete,
      enabled_printer_ids: [...state.enabledPrinterIds].sort(),
      selected_account_id: state.selectedAccountId,
      selected_source_ids: [...state.selectedSourceIds].sort(),
      selected_nozzles: Object.fromEntries(
        Object.entries(state.selectedNozzles).map(([printer, nozzles]) => [
          printer,
          [...nozzles].sort(numericSort),
        ]),
      ),
      filters: serializeFilters(state.filters),
      preset_template: state.presetTemplate,
      ams_template: state.amsTemplate,
      preset_rules: state.presetRules,
      ams_rules: state.amsRules,
    }),
  );
}

export function loadFilterPresets(storage: PreferencesStorage): FilterPreset[] {
  const value = storage.getItem("bfm.filter-presets");
  if (!value) return [];
  try {
    const parsed: unknown = JSON.parse(value);
    if (!Array.isArray(parsed)) return [];
    return parsed.flatMap((item) => {
      if (!item || typeof item !== "object") return [];
      const record = item as Record<string, unknown>;
      const filters = deserializeFilters(record.filters);
      return typeof record.id === "string" &&
        typeof record.name === "string" &&
        filters
        ? [{ id: record.id, name: record.name, filters }]
        : [];
    });
  } catch {
    return [];
  }
}

export function persistFilterPresets(
  storage: PreferencesStorage,
  presets: FilterPreset[],
): void {
  storage.setItem(
    "bfm.filter-presets",
    JSON.stringify(
      presets.map((preset) => ({
        id: preset.id,
        name: preset.name,
        filters: serializeFilters(preset.filters),
      })),
    ),
  );
}

function serializeFilters(filters: SourceFilters): Record<string, unknown> {
  return {
    search: filters.search,
    source_apps: [...filters.sourceApps].sort(),
    source_kinds: [...filters.sourceKinds].sort(),
    vendors: [...filters.vendors].sort(),
    materials: [...filters.materials].sort(),
    families: [...filters.families].sort(),
    variants: [...filters.variants].sort(),
    compatible_printers: [...filters.compatiblePrinters].sort(),
    migration_statuses: [...filters.migrationStatuses].sort(),
    selected_only: filters.selectedOnly,
  };
}

function deserializeFilters(value: unknown): SourceFilters | null {
  if (!value || typeof value !== "object") return null;
  const record = value as Record<string, unknown>;
  const sourceApps = stringArray(record.source_apps);
  const sourceKinds = stringArray(record.source_kinds);
  const vendors = stringArray(record.vendors);
  const materials = stringArray(record.materials);
  const families = stringArray(record.families);
  const variants = stringArray(record.variants);
  const compatiblePrinters = stringArray(record.compatible_printers);
  const migrationStatuses = stringArray(record.migration_statuses);
  if (
    !sourceApps ||
    !sourceKinds ||
    !vendors ||
    !materials ||
    !families ||
    !variants ||
    !compatiblePrinters ||
    !migrationStatuses
  )
    return null;
  return {
    search: typeof record.search === "string" ? record.search : "",
    sourceApps: new Set(
      sourceApps.filter(
        (item): item is SourceApp =>
          item === "orca_slicer" || item === "bambu_studio",
      ),
    ),
    sourceKinds: new Set(
      sourceKinds.filter(
        (item): item is SourceKind =>
          item === "factory_system" || item === "user_custom",
      ),
    ),
    vendors: new Set(vendors),
    materials: new Set(materials),
    families: new Set(families),
    variants: new Set(variants),
    compatiblePrinters: new Set(compatiblePrinters),
    migrationStatuses: new Set(
      migrationStatuses.filter((item): item is MigrationStatus =>
        [
          "new",
          "already_migrated",
          "incomplete",
          "conflicting",
          "unsupported",
        ].includes(item),
      ),
    ),
    selectedOnly: record.selected_only === true,
  };
}

function stringArray(value: unknown): string[] | null {
  return Array.isArray(value) && value.every((item) => typeof item === "string")
    ? value
    : null;
}

function stringArrayRecord(value: unknown): Record<string, string[]> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const entries = Object.entries(value as Record<string, unknown>);
  if (!entries.every(([, nozzles]) => stringArray(nozzles) !== null))
    return null;
  return Object.fromEntries(entries) as Record<string, string[]>;
}

export function loadNamingPresets(storage: PreferencesStorage): NamingPreset[] {
  const value = storage.getItem("bfm.naming-presets");
  if (!value) return [];
  try {
    const parsed: unknown = JSON.parse(value);
    return Array.isArray(parsed) && parsed.every(isNamingPreset) ? parsed : [];
  } catch {
    return [];
  }
}

export function persistNamingPresets(
  storage: PreferencesStorage,
  presets: NamingPreset[],
): void {
  storage.setItem("bfm.naming-presets", JSON.stringify(presets));
}

function isNamingPreset(value: unknown): value is NamingPreset {
  if (!value || typeof value !== "object") return false;
  const preset = value as Partial<NamingPreset>;
  return (
    typeof preset.id === "string" &&
    typeof preset.name === "string" &&
    typeof preset.preset_template === "string" &&
    typeof preset.ams_template === "string" &&
    Array.isArray(preset.preset_rules) &&
    preset.preset_rules.every(isNamingRule) &&
    Array.isArray(preset.ams_rules) &&
    preset.ams_rules.every(isNamingRule)
  );
}

function isNamingRule(value: unknown): value is NamingRule {
  if (!value || typeof value !== "object") return false;
  const rule = value as Partial<NamingRule>;
  return (
    typeof rule.id === "string" &&
    (rule.kind === "wildcard" || rule.kind === "regex") &&
    typeof rule.pattern === "string" &&
    typeof rule.replacement === "string" &&
    typeof rule.case_sensitive === "boolean" &&
    (rule.condition === null ||
      (typeof rule.condition === "object" &&
        typeof rule.condition?.field === "string" &&
        typeof rule.condition?.value === "string"))
  );
}
