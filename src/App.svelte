<script lang="ts">
  import { onMount } from "svelte";
  import {
    AlertOctagon,
    ArrowRight,
    CheckCircle2,
    LoaderCircle,
    Route,
  } from "@lucide/svelte";
  import { api } from "./lib/api";
  import { scopeViolations } from "./lib/template-composer";
  import AppChrome from "./lib/components/AppChrome.svelte";
  import ExecutionPhases from "./lib/components/ExecutionPhases.svelte";
  import type { WorkspaceView } from "./lib/components/WorkspaceTabs.svelte";
  import FilterPanel from "./lib/components/FilterPanel.svelte";
  import FirstRunSetup from "./lib/components/FirstRunSetup.svelte";
  import NamingPanel from "./lib/components/NamingPanel.svelte";
  import ModalDialog from "./lib/components/ModalDialog.svelte";
  import PlanDependencyDialog from "./lib/components/PlanDependencyDialog.svelte";
  import PrinterManagerDialog from "./lib/components/PrinterManagerDialog.svelte";
  import PlanTable from "./lib/components/PlanTable.svelte";
  import ResultSummary from "./lib/components/ResultSummary.svelte";
  import RestorePanel from "./lib/components/RestorePanel.svelte";
  import RunProgress from "./lib/components/RunProgress.svelte";
  import SourceTable from "./lib/components/SourceTable.svelte";
  import TargetPanel from "./lib/components/TargetPanel.svelte";
  import {
    activeFilterChips,
    appReducer,
    createInitialState,
    loadFilterPresets,
    loadNamingPresets,
    loadTheme,
    loadWorkspacePreferences,
    persistFilterPresets,
    persistNamingPresets,
    persistTheme,
    persistWorkspacePreferences,
    removeSetValue,
    resetFilters,
    selectedNozzleRequests,
    sourceFacets,
    visibleSources,
    type ActiveFilterChip,
    type AppAction,
    type AppState,
    type FilterPreset,
    type SourceFilters,
    type Theme,
  } from "./lib/state";
  import type {
    BuildPlanRequest,
    BuildPlanResponse,
    CatalogProgressEvent,
    ConflictChoice,
    ConflictDecision,
    NameOverride,
    NamePreview,
    NamingPreset,
    NamingRule,
    NamingRuleSpec,
    OutputSelection,
    RestorePreview,
    RollbackOutcome,
    SourceApp,
    SourceKind,
    TargetTemplateDecision,
    TargetTemplateIssue,
  } from "./lib/types";

  const storage =
    typeof localStorage === "undefined"
      ? { getItem: () => null, setItem: () => undefined }
      : localStorage;

  let appState: AppState = $state(
    createInitialState(loadTheme(storage), loadWorkspacePreferences(storage)),
  );
  let savedNamingPresets = $state<NamingPreset[]>(loadNamingPresets(storage));
  let savedFilterPresets = $state<FilterPreset[]>(loadFilterPresets(storage));
  let namePreview = $state<NamePreview | null>(null);
  let namePreviewError = $state<string | null>(null);
  let restorePreview = $state<RestorePreview | null>(null);
  let rollback = $state<RollbackOutcome | null>(null);
  let restoring = $state(false);
  let catalogProgress = $state<
    (CatalogProgressEvent & { scope: "sources" | "targets" }) | null
  >(null);
  let activeView = $state<WorkspaceView>("setup");
  let commitConfirmationOpen = $state(false);
  let dependencyIssues = $state<TargetTemplateIssue[]>([]);
  let pendingPlanRequest = $state<BuildPlanRequest | null>(null);
  let resolvingDependencies = $state(false);
  let printerManagerOpen = $state(false);

  const shownSources = $derived(visibleSources(appState));
  const facets = $derived(sourceFacets(appState.sources));
  const filterChips = $derived(activeFilterChips(appState.filters));
  const nozzleRequests = $derived(selectedNozzleRequests(appState));
  const selectedNozzleCount = $derived(
    nozzleRequests.reduce((sum, item) => sum + item.diameters.length, 0),
  );
  const outputCount = $derived(
    Number(appState.outputs.slicing_presets) +
      Number(appState.outputs.custom_filaments),
  );
  const amsScopeViolations = $derived(
    scopeViolations(appState.amsTemplate, "ams"),
  );
  const hasInputs = $derived(
    appState.selectedSourceIds.size > 0 &&
      nozzleRequests.length > 0 &&
      appState.selectedAccountId !== null &&
      appState.sourceCatalogId !== null &&
      appState.targetCatalogId !== null &&
      amsScopeViolations.length === 0,
  );
  const planBlocked = $derived(
    appState.plan?.operations.some((item) => item.action === "block") ?? false,
  );

  function dispatch(action: AppAction) {
    appState = appReducer(appState, action);
    persistWorkspacePreferences(storage, appState);
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function updateCatalogProgress(
    scope: "sources" | "targets",
    event: CatalogProgressEvent,
  ) {
    catalogProgress = { scope, ...event };
  }

  function catalogProgressTitle(
    progress: CatalogProgressEvent & { scope: "sources" | "targets" },
  ): string {
    if (progress.scope === "sources") {
      return progress.phase === "loading"
        ? "Reading source profile files"
        : progress.phase === "resolving"
          ? "Resolving source inheritance"
          : "Source catalog ready";
    }
    return progress.phase === "loading"
      ? "Reading Bambu target metadata"
      : progress.phase === "resolving"
        ? "Indexing Bambu target templates"
        : "Bambu target catalog ready";
  }

  function applyTheme(theme: Theme) {
    if (theme === "system") delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = theme;
    persistTheme(storage, theme);
    dispatch({ type: "theme_changed", theme });
  }

  async function discover() {
    dispatch({ type: "discovery_started" });
    try {
      const discovery = await api.discover();
      dispatch({ type: "discovery_loaded", discovery });
      const targetCatalogId = discovery.target_catalog_ids[0];
      if (!targetCatalogId) {
        throw new Error("No Bambu target profile catalog was discovered.");
      }
      catalogProgress = {
        scope: "sources",
        phase: "loading",
        processed: 0,
        total: 0,
      };
      const sourceCatalog = await api.catalogSources(
        discovery.source_root_ids,
        (event) => updateCatalogProgress("sources", event),
      );
      dispatch({
        type: "sources_loaded",
        catalogId: sourceCatalog.catalog_id,
        sources: sourceCatalog.sources,
      });
      catalogProgress = {
        scope: "targets",
        phase: "loading",
        processed: 0,
        total: 0,
      };
      const targetCatalog = await api.catalogTargets(targetCatalogId, (event) =>
        updateCatalogProgress("targets", event),
      );
      dispatch({
        type: "targets_loaded",
        catalogId: targetCatalog.catalog_id,
        printers: targetCatalog.printers,
      });
      catalogProgress = null;
    } catch (error) {
      catalogProgress = null;
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  async function chooseManualSourceFolder(
    sourceApp: SourceApp,
    sourceKind: SourceKind,
  ) {
    const previous = appState.discovery;
    dispatch({ type: "discovery_started" });
    try {
      const discovery = await api.chooseManualSourceFolder({
        source_app: sourceApp,
        source_kind: sourceKind,
      });
      if (!discovery) {
        if (previous)
          dispatch({ type: "discovery_loaded", discovery: previous });
        return;
      }
      dispatch({ type: "discovery_loaded", discovery });
      const sourceCatalog = await api.catalogSources(
        discovery.source_root_ids,
        (event) => updateCatalogProgress("sources", event),
      );
      dispatch({
        type: "sources_loaded",
        catalogId: sourceCatalog.catalog_id,
        sources: sourceCatalog.sources,
      });
      catalogProgress = null;
    } catch (error) {
      catalogProgress = null;
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  async function removeManualSourceFolder(id: string) {
    dispatch({ type: "discovery_started" });
    try {
      const discovery = await api.removeManualSourceFolder(id);
      dispatch({ type: "discovery_loaded", discovery });
      const sourceCatalog = await api.catalogSources(
        discovery.source_root_ids,
        (event) => updateCatalogProgress("sources", event),
      );
      dispatch({
        type: "sources_loaded",
        catalogId: sourceCatalog.catalog_id,
        sources: sourceCatalog.sources,
      });
      catalogProgress = null;
    } catch (error) {
      catalogProgress = null;
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  function changeFilters(filters: Partial<SourceFilters>) {
    dispatch({ type: "filters_changed", filters });
  }

  function removeFilter(chip: ActiveFilterChip) {
    if (chip.dimension === "search") return changeFilters({ search: "" });
    if (chip.dimension === "selectedOnly")
      return changeFilters({ selectedOnly: false });
    const current = appState.filters[chip.dimension];
    changeFilters({
      [chip.dimension]: removeSetValue(current, chip.value as never),
    });
  }

  function ruleSpecs(rules: NamingRule[]): NamingRuleSpec[] {
    return rules.map((rule) => ({
      kind: rule.kind,
      pattern: rule.pattern,
      replacement: rule.replacement,
      case_sensitive: rule.case_sensitive,
      condition: rule.condition,
    }));
  }

  function presetId(name: string): string {
    return (
      name
        .toLocaleLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-|-$/g, "") || "preset"
    );
  }

  function cloneFilters(filters: SourceFilters): SourceFilters {
    return structuredClone(filters);
  }

  function saveFilterPreset(name: string) {
    const id = presetId(name);
    const preset: FilterPreset = {
      id,
      name,
      filters: cloneFilters(appState.filters),
    };
    savedFilterPresets = [
      ...savedFilterPresets.filter((item) => item.id !== id),
      preset,
    ].sort((left, right) => left.name.localeCompare(right.name));
    persistFilterPresets(storage, savedFilterPresets);
  }

  function loadFilterPreset(id: string) {
    const preset = savedFilterPresets.find((item) => item.id === id);
    if (preset) changeFilters(cloneFilters(preset.filters));
  }

  function deleteFilterPreset(id: string) {
    savedFilterPresets = savedFilterPresets.filter((item) => item.id !== id);
    persistFilterPresets(storage, savedFilterPresets);
  }

  async function refreshPreview(
    preset = appState.presetTemplate,
    ams = appState.amsTemplate,
    presetRules = appState.presetRules,
    amsRules = appState.amsRules,
  ) {
    namePreviewError = null;
    const source = appState.sources.find((item) =>
      appState.selectedSourceIds.has(item.id),
    );
    const target = appState.printers.find(
      (item) => appState.selectedNozzles[item.id]?.size,
    );
    const nozzle = target
      ? [...(appState.selectedNozzles[target.id] ?? [])][0]
      : undefined;
    if (!source || !target || !nozzle) {
      namePreview = null;
      return;
    }
    try {
      const rows = await api.previewNames({
        preset_template: preset,
        ams_template: ams,
        preset_rules: ruleSpecs(presetRules),
        ams_rules: ruleSpecs(amsRules),
        rows: [
          {
            source_name: source.name,
            vendor: source.vendor,
            material: source.material,
            family: source.family,
            variant: source.variant,
            source_app: source.source_app,
            source_kind: source.source_kind,
            printer: target.name,
            printer_code: target.code,
            nozzle,
          },
        ],
      });
      namePreview = rows[0] ?? null;
    } catch (error) {
      namePreviewError = errorMessage(error);
    }
  }

  function changeTemplates(preset: string, ams: string) {
    const rebuild = appState.plan !== null;
    dispatch({ type: "templates_changed", preset, ams });
    void refreshPreview(preset, ams);
    if (rebuild) void buildPlan({ presetTemplate: preset, amsTemplate: ams });
  }

  function changeRules(presetRules: NamingRule[], amsRules: NamingRule[]) {
    const rebuild = appState.plan !== null;
    dispatch({ type: "rules_changed", presetRules, amsRules });
    void refreshPreview(
      appState.presetTemplate,
      appState.amsTemplate,
      presetRules,
      amsRules,
    );
    if (rebuild) void buildPlan({ presetRules, amsRules });
  }

  function changeOutputs(outputs: OutputSelection) {
    const rebuild = appState.plan !== null;
    dispatch({ type: "outputs_changed", outputs });
    if (rebuild) void buildPlan({ outputs });
  }

  function saveNamingPreset(name: string) {
    const id = presetId(name);
    const preset: NamingPreset = {
      id,
      name,
      preset_template: appState.presetTemplate,
      ams_template: appState.amsTemplate,
      preset_rules: appState.presetRules,
      ams_rules: appState.amsRules,
    };
    savedNamingPresets = [
      ...savedNamingPresets.filter((item) => item.id !== id),
      preset,
    ].sort((left, right) => left.name.localeCompare(right.name));
    persistNamingPresets(storage, savedNamingPresets);
  }

  function loadNamingPreset(id: string) {
    const preset = savedNamingPresets.find((item) => item.id === id);
    if (!preset) return;
    const rebuild = appState.plan !== null;
    dispatch({
      type: "templates_changed",
      preset: preset.preset_template,
      ams: preset.ams_template,
    });
    dispatch({
      type: "rules_changed",
      presetRules: preset.preset_rules,
      amsRules: preset.ams_rules,
    });
    void refreshPreview(
      preset.preset_template,
      preset.ams_template,
      preset.preset_rules,
      preset.ams_rules,
    );
    if (rebuild) {
      void buildPlan({
        presetTemplate: preset.preset_template,
        amsTemplate: preset.ams_template,
        presetRules: preset.preset_rules,
        amsRules: preset.ams_rules,
      });
    }
  }

  function deleteNamingPreset(id: string) {
    savedNamingPresets = savedNamingPresets.filter((item) => item.id !== id);
    persistNamingPresets(storage, savedNamingPresets);
  }

  function createBuildPlanRequest(
    options: {
      presetTemplate?: string;
      amsTemplate?: string;
      presetRules?: NamingRule[];
      amsRules?: NamingRule[];
      outputs?: OutputSelection;
      overrides?: NameOverride[];
      conflictDecisions?: ConflictDecision[];
    } = {},
  ): BuildPlanRequest | null {
    if (
      !hasInputs ||
      !appState.sourceCatalogId ||
      !appState.targetCatalogId ||
      !appState.selectedAccountId
    )
      return null;
    return {
      source_catalog_id: appState.sourceCatalogId,
      source_ids: [...appState.selectedSourceIds].sort(),
      target_catalog_id: appState.targetCatalogId,
      nozzles: nozzleRequests,
      destination_account_id: appState.selectedAccountId,
      preset_template: options.presetTemplate ?? appState.presetTemplate,
      ams_template: options.amsTemplate ?? appState.amsTemplate,
      outputs: options.outputs ?? appState.outputs,
      naming: {
        preset_rules: ruleSpecs(options.presetRules ?? appState.presetRules),
        ams_rules: ruleSpecs(options.amsRules ?? appState.amsRules),
        overrides: options.overrides ?? appState.nameOverrides,
        conflict_decisions:
          options.conflictDecisions ?? appState.conflictDecisions,
      },
    };
  }

  function applyBuildPlanResponse(
    response: BuildPlanResponse,
    request: BuildPlanRequest,
  ) {
    if (response.status === "ready") {
      dependencyIssues = [];
      pendingPlanRequest = null;
      dispatch({ type: "plan_built", plan: response.plan });
      activeView = "plan";
      return;
    }
    dependencyIssues = response.issues;
    pendingPlanRequest = request;
    dispatch({ type: "planning_stopped" });
  }

  async function buildPlan(
    options: {
      presetTemplate?: string;
      amsTemplate?: string;
      presetRules?: NamingRule[];
      amsRules?: NamingRule[];
      outputs?: OutputSelection;
      overrides?: NameOverride[];
      conflictDecisions?: ConflictDecision[];
    } = {},
  ) {
    const request = createBuildPlanRequest(options);
    if (!request) return;
    dispatch({ type: "planning_started" });
    try {
      applyBuildPlanResponse(await api.buildPlan(request), request);
    } catch (error) {
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  function cancelDependencyResolution() {
    dependencyIssues = [];
    pendingPlanRequest = null;
    dispatch({ type: "planning_stopped" });
  }

  async function resolvePlanDependencies(decisions: TargetTemplateDecision[]) {
    if (!pendingPlanRequest) return;
    resolvingDependencies = true;
    const sourceIds = new Set([
      ...pendingPlanRequest.source_ids,
      ...decisions.flatMap((decision) =>
        decision.action === "use_source" && decision.source_id
          ? [decision.source_id]
          : [],
      ),
    ]);
    for (const decision of decisions) {
      if (decision.action === "use_source" && decision.source_id) {
        if (!appState.selectedSourceIds.has(decision.source_id)) {
          dispatch({ type: "source_toggled", sourceId: decision.source_id });
        }
      }
    }
    const request = {
      ...pendingPlanRequest,
      source_ids: [...sourceIds].sort(),
    };
    try {
      applyBuildPlanResponse(
        await api.resolvePlanDependencies(request, decisions),
        request,
      );
    } catch (error) {
      dependencyIssues = [];
      pendingPlanRequest = null;
      dispatch({ type: "failed", message: errorMessage(error) });
    } finally {
      resolvingDependencies = false;
    }
  }

  async function removeDependencySources(sourceIds: string[]) {
    if (!pendingPlanRequest) return;
    const removed = new Set(sourceIds);
    const request = {
      ...pendingPlanRequest,
      source_ids: pendingPlanRequest.source_ids.filter(
        (id) => !removed.has(id),
      ),
    };
    dispatch({ type: "sources_removed", sourceIds: removed });
    dependencyIssues = [];
    pendingPlanRequest = null;
    dispatch({ type: "planning_stopped" });
    if (request.source_ids.length === 0 || request.nozzles.length === 0) return;
    dispatch({ type: "planning_started" });
    try {
      applyBuildPlanResponse(await api.buildPlan(request), request);
    } catch (error) {
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  function overrideOperation(
    operationId: string,
    presetName: string,
    amsName: string,
  ) {
    const operation = appState.plan?.operations.find(
      (item) => item.id === operationId,
    );
    if (!operation) return;
    const next: NameOverride = {
      source_id: operation.source_id,
      printer_id: operation.printer_id,
      nozzle: operation.nozzle,
      preset_name: presetName === operation.preset_name ? null : presetName,
      ams_name: amsName === operation.ams_name ? null : amsName,
    };
    const overrides = [
      ...appState.nameOverrides.filter(
        (item) =>
          item.source_id !== next.source_id ||
          item.printer_id !== next.printer_id ||
          item.nozzle !== next.nozzle,
      ),
      next,
    ];
    dispatch({ type: "name_overrides_changed", overrides });
    void buildPlan({ overrides });
  }

  function decideConflict(operationId: string, choice: ConflictChoice | null) {
    const operation = appState.plan?.operations.find(
      (item) => item.id === operationId,
    );
    if (!operation) return;
    const matchesOperation = (decision: ConflictDecision) =>
      decision.source_id === operation.source_id &&
      decision.printer_id === operation.printer_id &&
      decision.nozzle === operation.nozzle;
    const decisions = appState.conflictDecisions.filter(
      (decision) => !matchesOperation(decision),
    );
    if (choice) {
      decisions.push({
        source_id: operation.source_id,
        printer_id: operation.printer_id,
        nozzle: operation.nozzle,
        choice,
      });
    }
    dispatch({ type: "conflict_decisions_changed", decisions });
    void buildPlan({ conflictDecisions: decisions });
  }

  async function synchronizeRun(runId = appState.result?.run_id) {
    if (!runId) return;
    dispatch({ type: "synchronization_started" });
    try {
      const synchronization = await api.synchronizeRun(runId, (event) => {
        if (event.run_id === runId) {
          dispatch({ type: "synchronization_phase", phase: event.phase });
        }
      });
      dispatch({ type: "synchronization_completed", result: synchronization });
    } catch (error) {
      dispatch({
        type: "synchronization_failed",
        message: errorMessage(error),
      });
    }
  }

  async function cancelSynchronization() {
    if (!appState.result || appState.phase !== "synchronizing") return;
    try {
      await api.cancelRun(appState.result.run_id);
    } catch (error) {
      dispatch({
        type: "synchronization_failed",
        message: errorMessage(error),
      });
    }
  }

  async function executePlan() {
    if (!appState.plan || planBlocked) return;
    activeView = "activity";
    dispatch({ type: "execution_started" });
    try {
      const planId = appState.plan.id;
      const result = await api.executePlan(planId, (event) => {
        if (event.plan_id === planId) {
          dispatch({ type: "execution_phase", phase: event.phase });
        }
      });
      for (const operation of appState.plan.operations) {
        if (operation.action !== "block" && operation.action !== "skip") {
          dispatch({
            type: "progress_updated",
            operationId: operation.id,
            evidence: "created_local",
            state: "created_local",
          });
        }
      }
      dispatch({ type: "execution_completed", result });
      if (result.committed_files > 0) {
        await synchronizeRun(result.run_id);
      }
    } catch (error) {
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  async function previewRestore() {
    if (!appState.result) return;
    activeView = "restore";
    try {
      restorePreview = await api.restorePreview(appState.result.run_id);
    } catch (error) {
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  async function restoreOwned() {
    if (!appState.result) return;
    restoring = true;
    try {
      rollback = await api.restoreOwned(appState.result.run_id);
      restorePreview = await api.restorePreview(appState.result.run_id);
    } catch (error) {
      dispatch({ type: "failed", message: errorMessage(error) });
    } finally {
      restoring = false;
    }
  }

  async function verifyAms(operationIds: string[]) {
    if (!appState.result) return;
    try {
      await api.recordAmsVerification(
        appState.result.run_id,
        operationIds,
        "Verified in Bambu Studio AMS device view",
      );
      dispatch({ type: "ams_verified", operationIds });
    } catch (error) {
      dispatch({ type: "failed", message: errorMessage(error) });
    }
  }

  function saveEnabledPrinters(
    enabledPrinterIds: Set<string>,
    selectedNozzles: Record<string, Set<string>>,
  ) {
    printerManagerOpen = false;
    dispatch({
      type: "enabled_printers_changed",
      enabledPrinterIds,
      selectedNozzles,
    });
    void refreshPreview();
  }

  function completeFirstRun(selection: {
    sourceApps: Set<SourceApp>;
    sourceKinds: Set<SourceKind>;
    enabledPrinterIds: Set<string>;
    selectedNozzles: Record<string, Set<string>>;
  }) {
    dispatch({ type: "setup_completed", ...selection });
  }

  onMount(() => {
    applyTheme(appState.theme);
    void discover();
  });
</script>

<svelte:head>
  <title>Spool Ledger · Bambu Filament Migrator</title>
  <meta
    name="description"
    content="Migrate OrcaSlicer and Bambu Studio filament profiles into Bambu custom filaments."
  />
</svelte:head>

<div class="app-frame">
  {#if appState.discovery && appState.setupComplete}
    <AppChrome
      discovery={appState.discovery}
      selectedAccountId={appState.selectedAccountId}
      theme={appState.theme}
      busy={appState.phase === "discovering" || catalogProgress !== null}
      {activeView}
      planCount={appState.plan?.operations.length ?? 0}
      onRefresh={discover}
      onAddSource={chooseManualSourceFolder}
      onRemoveSource={removeManualSourceFolder}
      onAccountChanged={(accountId) =>
        dispatch({ type: "account_selected", accountId })}
      onThemeChanged={applyTheme}
      onViewChanged={(view) => (activeView = view)}
    />
  {:else}
    <header class="boot-brand">
      <img
        class="boot-brand-lockup"
        src="/spool-ledger-lockup-transparent.png"
        alt="Spool Ledger · Bambu Filament Migrator"
      />
    </header>
  {/if}

  {#if appState.error}
    <div class="error-banner" role="alert">
      <AlertOctagon size={18} />
      <span><strong>Migration stopped</strong>{appState.error}</span>
      {#if !appState.discovery}<button type="button" onclick={discover}
          >Retry discovery</button
        >{/if}
    </div>
  {/if}

  {#if catalogProgress}
    <main class="catalog-load-state" aria-live="polite" aria-busy="true">
      <div class="catalog-load-card">
        <span class="scan-spool" aria-hidden="true">
          <img
            class="catalog-logo catalog-logo-light"
            src="/spool-ledger-logo-light.png"
            alt=""
          />
          <img
            class="catalog-logo catalog-logo-dark"
            src="/spool-ledger-logo-dark.png"
            alt=""
          />
        </span>
        <p class="section-kicker">
          {catalogProgress.scope === "sources"
            ? "Source profile inventory"
            : "Bambu target inventory"}
        </p>
        <h2>{catalogProgressTitle(catalogProgress)}</h2>
        <div class="catalog-progress-copy">
          <strong
            >{catalogProgress.processed.toLocaleString()} / {catalogProgress.total.toLocaleString()}
            profiles</strong
          >
          <span
            >{catalogProgress.total > 0
              ? `${Math.round((catalogProgress.processed / catalogProgress.total) * 100)}%`
              : "Counting profiles…"}</span
          >
        </div>
        <div
          class:indeterminate={catalogProgress.total === 0}
          class="catalog-progress-track"
          role="progressbar"
          aria-label="Profile loading progress"
          aria-valuemin="0"
          aria-valuemax={Math.max(catalogProgress.total, 1)}
          aria-valuenow={catalogProgress.processed}
        >
          <span
            style={`width: ${catalogProgress.total > 0 ? Math.min(100, (catalogProgress.processed / catalogProgress.total) * 100) : 35}%`}
          ></span>
        </div>
        <p>No profile files are changed while this inventory is built.</p>
      </div>
    </main>
  {:else if !appState.discovery && appState.phase !== "error"}
    <main class="boot-state" aria-live="polite">
      <span class="scan-spool"><LoaderCircle size={28} /></span>
      <p class="section-kicker">Local profile inventory</p>
      <h2>Reading slicer manifests</h2>
      <p>No profile files are changed during discovery.</p>
    </main>
  {:else if appState.discovery && !appState.setupComplete && appState.sourceCatalogId && appState.targetCatalogId}
    <FirstRunSetup
      catalogId={appState.targetCatalogId}
      sources={appState.sources}
      printers={appState.printers}
      onComplete={completeFirstRun}
    />
  {:else if appState.discovery && appState.setupComplete}
    <main class="workspace-shell">
      {#if activeView === "setup"}
        <div
          id="setup-view"
          class="workspace-view"
          role="tabpanel"
          aria-labelledby="setup-tab"
          tabindex="-1"
        >
          <header class="setup-intro">
            <div>
              <p class="section-kicker">Migration setup</p>
              <h2>Prepare migration</h2>
              <p>
                Route resolved source profiles through naming into official
                Bambu printer targets.
              </p>
            </div>
            <ol class="workflow-steps" aria-label="Migration setup steps">
              <li class:ready={appState.discovery !== null}>
                <span>1</span>Sources
              </li>
              <li class:ready={appState.selectedSourceIds.size > 0}>
                <span>2</span>Select
              </li>
              <li class:ready={namePreview !== null}><span>3</span>Name</li>
              <li class:ready={nozzleRequests.length > 0}>
                <span>4</span>Hardware
              </li>
            </ol>
            <p class="discovery-status" role="status">
              <CheckCircle2 size={15} /> Profile inventory ready
            </p>
          </header>

          <div class="routing-bench" aria-label="Filament migration route">
            <article class="route-rail source-rail">
              <div class="source-rail-body">
                <FilterPanel
                  filters={appState.filters}
                  {facets}
                  savedPresets={savedFilterPresets}
                  onFiltersChanged={changeFilters}
                  onReset={() => changeFilters(resetFilters())}
                  onSavePreset={saveFilterPreset}
                  onLoadPreset={loadFilterPreset}
                  onDeletePreset={deleteFilterPreset}
                />

                <div class="source-workbench">
                  {#if filterChips.length}
                    <div class="filter-chips" aria-label="Active filters">
                      {#each filterChips as chip (chip.id)}<button
                          type="button"
                          onclick={() => removeFilter(chip)}
                          >{chip.label}<span aria-hidden="true">×</span></button
                        >{/each}
                    </div>
                  {/if}
                  <SourceTable
                    sources={shownSources}
                    selectedIds={appState.selectedSourceIds}
                    totalCount={appState.sources.length}
                    onToggle={(sourceId) => {
                      dispatch({ type: "source_toggled", sourceId });
                      void refreshPreview();
                    }}
                    onSelectVisible={(selected) =>
                      dispatch({ type: "select_visible", selected })}
                    onClearSelection={() =>
                      dispatch({ type: "clear_selection" })}
                  />
                </div>
              </div>
            </article>

            <div class="route-thread" aria-hidden="true">
              <span></span><ArrowRight size={15} />
            </div>

            <article class="route-rail naming-rail">
              <NamingPanel
                presetTemplate={appState.presetTemplate}
                amsTemplate={appState.amsTemplate}
                presetRules={appState.presetRules}
                amsRules={appState.amsRules}
                savedPresets={savedNamingPresets}
                preview={namePreview}
                error={namePreviewError}
                outputs={appState.outputs}
                onTemplatesChanged={changeTemplates}
                onRulesChanged={changeRules}
                onOutputsChanged={changeOutputs}
                onSavePreset={saveNamingPreset}
                onLoadPreset={loadNamingPreset}
                onDeletePreset={deleteNamingPreset}
              />
            </article>

            <div class="route-thread destination" aria-hidden="true">
              <span></span><Route size={15} />
            </div>

            <article class="route-rail destination-rail">
              <TargetPanel
                catalogId={appState.targetCatalogId ?? ""}
                printers={appState.printers}
                enabledPrinterIds={appState.enabledPrinterIds}
                selectedNozzles={appState.selectedNozzles}
                onManage={() => (printerManagerOpen = true)}
                onNozzleToggle={(printerId, diameter) => {
                  dispatch({ type: "nozzle_toggled", printerId, diameter });
                  void refreshPreview();
                }}
                onSelectAll={(printerId, selected) => {
                  dispatch({
                    type: "select_printer_nozzles",
                    printerId,
                    selected,
                  });
                  void refreshPreview();
                }}
              />
            </article>
          </div>

          <div class="setup-actionbar">
            <div class="setup-counts">
              <span
                aria-label={`${appState.selectedSourceIds.size} ${appState.selectedSourceIds.size === 1 ? "source" : "sources"}`}
                ><strong>{appState.selectedSourceIds.size}</strong>
                {appState.selectedSourceIds.size === 1
                  ? "source"
                  : "sources"}</span
              >
              <span
                aria-label={`${selectedNozzleCount} ${selectedNozzleCount === 1 ? "nozzle" : "nozzles"}`}
                ><strong>{selectedNozzleCount}</strong>
                {selectedNozzleCount === 1 ? "nozzle" : "nozzles"}</span
              >
              <span aria-label={`${outputCount} outputs`}
                ><strong>{outputCount}</strong> outputs</span
              >
            </div>
            <span class:ready={hasInputs} class="plan-readiness">
              {amsScopeViolations.length > 0
                ? "Fix the AMS template"
                : hasInputs
                  ? "Ready to build"
                  : "Select a source and nozzle"}
            </span>
            <button
              class="primary-button"
              type="button"
              disabled={!hasInputs || appState.phase === "planning"}
              onclick={() => void buildPlan()}
              >{appState.phase === "planning"
                ? "Building plan…"
                : "Build migration plan"}</button
            >
          </div>
        </div>
      {:else if activeView === "plan"}
        <div
          id="plan-view"
          class="workspace-view"
          role="tabpanel"
          aria-labelledby="plan-tab"
          tabindex="-1"
        >
          {#if appState.plan}
            <PlanTable
              plan={appState.plan}
              conflictDecisions={appState.conflictDecisions}
              onOverride={overrideOperation}
              onDecision={decideConflict}
            />
            <div class="plan-actionbar">
              <div>
                <strong>Frozen plan <code>{appState.plan.id}</code></strong>
                <p>
                  Local generation cannot guarantee cloud persistence or AMS
                  visibility.
                </p>
              </div>
              <button
                class="primary-button"
                type="button"
                disabled={planBlocked || appState.phase === "executing"}
                onclick={() => (commitConfirmationOpen = true)}
                >{appState.phase === "executing"
                  ? "Committing…"
                  : "Commit migration"}</button
              >
            </div>
          {:else}
            <div class="view-empty">
              <h2>No migration plan yet</h2>
              <p>Build a migration plan to review exact output operations.</p>
            </div>
          {/if}
        </div>
      {:else if activeView === "activity"}
        <div
          id="activity-view"
          class="workspace-view"
          role="tabpanel"
          aria-labelledby="activity-tab"
          tabindex="-1"
        >
          {#if appState.plan && (appState.phase === "executing" || appState.result)}
            <ExecutionPhases
              localPhase={appState.executionPhase}
              syncPhase={appState.syncPhase}
              syncExpected={!appState.result ||
                appState.result.committed_files > 0}
            />
          {/if}

          {#if appState.plan && (appState.phase === "executing" || appState.phase === "synchronizing" || Object.keys(appState.progress).length)}
            <RunProgress
              plan={appState.plan}
              progress={appState.progress}
              running={appState.phase === "executing" ||
                appState.phase === "synchronizing"}
              localPhase={appState.phase === "executing"
                ? appState.executionPhase
                : null}
              cancellable={appState.phase === "synchronizing"}
              onCancel={cancelSynchronization}
              onVerify={verifyAms}
            />
          {/if}

          {#if appState.result}
            <ResultSummary
              result={appState.result}
              synchronization={appState.synchronization}
              syncError={appState.syncError}
              syncing={appState.phase === "synchronizing"}
              onRetrySync={synchronizeRun}
              onOpenRestore={previewRestore}
              onOpenSupport={api.openSupportPage}
            />
          {:else if !(appState.plan && (appState.phase === "executing" || appState.phase === "synchronizing" || Object.keys(appState.progress).length))}
            <div class="view-empty">
              <h2>No migration run yet</h2>
              <p>
                Review and commit a migration plan to see execution evidence.
              </p>
            </div>
          {/if}
        </div>
      {:else}
        <div
          id="restore-view"
          class="workspace-view"
          role="tabpanel"
          aria-labelledby="restore-tab"
          tabindex="-1"
        >
          {#if appState.result}
            <RestorePanel
              result={appState.result}
              preview={restorePreview}
              {rollback}
              {restoring}
              onPreview={previewRestore}
              onRestore={restoreOwned}
            />
          {:else}
            <div class="view-empty">
              <h2>No journal-owned restore available</h2>
              <p>
                Commit a migration before previewing journal-owned restore
                paths.
              </p>
            </div>
          {/if}
        </div>
      {/if}
    </main>

    <PlanDependencyDialog
      open={dependencyIssues.length > 0}
      issues={dependencyIssues}
      busy={resolvingDependencies}
      onCancel={cancelDependencyResolution}
      onResolve={resolvePlanDependencies}
      onRemove={removeDependencySources}
    />

    {#if appState.targetCatalogId}
      <PrinterManagerDialog
        open={printerManagerOpen}
        catalogId={appState.targetCatalogId}
        printers={appState.printers}
        enabledPrinterIds={appState.enabledPrinterIds}
        selectedNozzles={appState.selectedNozzles}
        onSave={saveEnabledPrinters}
        onCancel={() => (printerManagerOpen = false)}
      />
    {/if}

    {#if appState.plan}
      <ModalDialog
        open={commitConfirmationOpen}
        title={`Commit plan ${appState.plan.id}?`}
        confirmLabel="Commit migration"
        onClose={() => (commitConfirmationOpen = false)}
        onConfirm={() => {
          commitConfirmationOpen = false;
          void executePlan();
        }}
      >
        <p>
          This writes the reviewed files to the selected local Bambu account. A
          backup and journal are created first.
        </p>
        <p>
          Cloud persistence and AMS visibility are not guaranteed. Those are
          verified separately after Bambu Studio loads the local profiles.
        </p>
      </ModalDialog>
    {/if}
  {/if}
</div>
