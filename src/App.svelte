<script lang="ts">
  import { onMount, tick } from "svelte";
  import { AlertOctagon, LoaderCircle } from "@lucide/svelte";
  import { api } from "./lib/api";
  import { scopeViolations } from "./lib/template-composer";
  import AppChrome from "./lib/components/AppChrome.svelte";
  import RunStatus from "./lib/components/RunStatus.svelte";
  import type { WorkspaceView } from "./lib/components/WorkspaceTabs.svelte";
  import FilamentPicker from "./lib/components/FilamentPicker.svelte";
  import FirstRunSetup from "./lib/components/FirstRunSetup.svelte";
  import NamingPanel from "./lib/components/NamingPanel.svelte";
  import ModalDialog from "./lib/components/ModalDialog.svelte";
  import PlanDependencyDialog from "./lib/components/PlanDependencyDialog.svelte";
  import PrinterManagerDialog from "./lib/components/PrinterManagerDialog.svelte";
  import PlanTable from "./lib/components/PlanTable.svelte";
  import ResultSummary from "./lib/components/ResultSummary.svelte";
  import RestorePanel from "./lib/components/RestorePanel.svelte";
  import RouteBar from "./lib/components/RouteBar.svelte";
  import RunProgress from "./lib/components/RunProgress.svelte";
  import RunRail from "./lib/components/RunRail.svelte";
  import StepRail from "./lib/components/StepRail.svelte";
  import TargetPanel from "./lib/components/TargetPanel.svelte";
  import UpdateNotice from "./lib/components/UpdateNotice.svelte";
  import { rememberSkippedVersion, shouldNotifyUpdate } from "./lib/updates";
  import { runRail } from "./lib/run-rail";
  import { STEP_IDS, WORKSPACE_STEPS as STEPS } from "./lib/steps";
  import {
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
    selectedNozzleRequests,
    type AppAction,
    type AppState,
    type FilterPreset,
    type SourceFilters,
    type Theme,
  } from "./lib/state";
  import type {
    AvailableUpdate,
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
  let availableUpdate = $state<AvailableUpdate | null>(null);
  let updateError = $state<string | null>(null);

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
  let step = $state(0);
  let revealFrom = $state("18px");

  /** Continue checks only the open step; the last step needs a complete plan. */
  const readiness = $derived.by((): { ready: boolean; label: string } => {
    const sources = appState.selectedSourceIds.size;
    if (step === 0) {
      return sources > 0
        ? { ready: true, label: "Ready to continue" }
        : { ready: false, label: "Select a preset" };
    }
    if (step === 1) {
      if (outputCount === 0) return { ready: false, label: "Enable an output" };
      if (amsScopeViolations.length > 0)
        return { ready: false, label: "Fix the AMS template" };
      return { ready: true, label: "Ready to continue" };
    }
    if (sources === 0) return { ready: false, label: "Select a preset" };
    if (selectedNozzleCount === 0)
      return { ready: false, label: "Select hardware" };
    if (outputCount === 0) return { ready: false, label: "Enable an output" };
    if (amsScopeViolations.length > 0)
      return { ready: false, label: "Fix the AMS template" };
    if (!hasInputs) return { ready: false, label: "Select an account" };
    return { ready: true, label: "Ready to build" };
  });
  const modalOpen = $derived(
    commitConfirmationOpen || printerManagerOpen || dependencyIssues.length > 0,
  );
  const rail = $derived(
    runRail({
      local: appState.executionTimeline,
      sync: appState.syncTimeline,
      executing: appState.phase === "executing",
      localFailed: appState.executionFailed,
      syncing: appState.phase === "synchronizing",
      syncExpected: !appState.result || appState.result.committed_files > 0,
      syncStopped:
        Boolean(appState.syncError) ||
        Boolean(appState.synchronization?.timed_out),
    }),
  );

  async function openStep(index: number) {
    const next = Math.max(0, Math.min(STEPS.length - 1, index));
    revealFrom = next >= step ? "18px" : "-18px";
    step = next;
    await tick();
    document
      .getElementById(`${STEP_IDS[next]}-heading`)
      ?.focus({ preventScroll: true });
  }

  function continueRoute() {
    if (step < STEPS.length - 1) void openStep(step + 1);
    else void buildPlan();
  }

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

  const previewInputs = $derived.by(() => {
    const source = appState.sources.find((item) =>
      appState.selectedSourceIds.has(item.id),
    );
    const target = appState.printers.find(
      (item) => appState.selectedNozzles[item.id]?.size,
    );
    const nozzle = target
      ? [...(appState.selectedNozzles[target.id] ?? [])][0]
      : undefined;
    return source && target && nozzle ? { source, target, nozzle } : null;
  });
  const previewSubject = $derived(
    previewInputs
      ? `${previewInputs.source.name} on ${previewInputs.target.name} ${previewInputs.nozzle} mm`
      : null,
  );

  async function refreshPreview(
    preset = appState.presetTemplate,
    ams = appState.amsTemplate,
    presetRules = appState.presetRules,
    amsRules = appState.amsRules,
  ) {
    namePreviewError = null;
    if (!previewInputs) {
      namePreview = null;
      return;
    }
    const { source, target, nozzle } = previewInputs;
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
          dispatch({
            type: "synchronization_phase",
            phase: event.phase,
            elapsedMs: event.elapsed_ms,
          });
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
          dispatch({
            type: "execution_phase",
            phase: event.phase,
            elapsedMs: event.elapsed_ms,
          });
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

  async function checkForUpdate() {
    try {
      const update = await api.checkForUpdate();
      if (
        update.status === "available" &&
        shouldNotifyUpdate(update, localStorage)
      ) {
        availableUpdate = update;
      }
    } catch (error) {
      // An unreachable release list must never interrupt a migration.
      console.warn("Update check failed:", errorMessage(error));
    }
  }

  async function openAvailableRelease() {
    if (!availableUpdate) return;
    updateError = null;
    try {
      await api.openReleasePage(availableUpdate.tag);
      availableUpdate = null;
    } catch {
      updateError = "Could not open the release page. Please try again.";
    }
  }

  function skipAvailableRelease() {
    if (!availableUpdate) return;
    rememberSkippedVersion(localStorage, availableUpdate.tag);
    availableUpdate = null;
  }

  onMount(() => {
    applyTheme(appState.theme);
    void discover();
    void checkForUpdate();
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
      {activeView}
      planCount={appState.plan?.operations.length ?? 0}
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

  {#if availableUpdate && appState.setupComplete}
    <UpdateNotice
      update={availableUpdate}
      error={updateError}
      onUpdate={() => void openAvailableRelease()}
      onDismiss={() => (availableUpdate = null)}
      onSkip={skipAvailableRelease}
    />
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
          class="workspace-view setup-view"
          role="tabpanel"
          aria-labelledby="setup-tab"
          tabindex="-1"
        >
          <h1 class="sr-only">Migration workspace</h1>
          <div class="route-shell" aria-label="Migration setup steps">
            <StepRail
              id={STEP_IDS[0]}
              index={0}
              class="source-rail"
              title={STEPS[0].title}
              description={STEPS[0].description}
              icon={STEPS[0].icon}
              count={appState.selectedSourceIds.size}
              unit="selected"
              open={step === 0}
              done={step > 0}
              {revealFrom}
              onOpen={() => openStep(0)}
            >
              <FilamentPicker
                sources={appState.sources}
                filters={appState.filters}
                selectedIds={appState.selectedSourceIds}
                historyActive={activeView === "setup" &&
                  step === 0 &&
                  !modalOpen}
                discovery={appState.discovery}
                busy={appState.phase === "discovering" ||
                  catalogProgress !== null}
                savedPresets={savedFilterPresets}
                onFiltersChanged={changeFilters}
                onToggle={(sourceId) => {
                  dispatch({ type: "source_toggled", sourceId });
                  void refreshPreview();
                }}
                onSelect={(sourceIds, selected) => {
                  dispatch({ type: "sources_selected", sourceIds, selected });
                  void refreshPreview();
                }}
                onClearSelection={() => {
                  dispatch({ type: "clear_selection" });
                  void refreshPreview();
                }}
                onSavePreset={saveFilterPreset}
                onLoadPreset={loadFilterPreset}
                onDeletePreset={deleteFilterPreset}
                onRefresh={discover}
                onAddSource={chooseManualSourceFolder}
                onRemoveSource={removeManualSourceFolder}
              />
            </StepRail>

            <StepRail
              id={STEP_IDS[1]}
              index={1}
              class="naming-rail"
              title={STEPS[1].title}
              description={STEPS[1].description}
              icon={STEPS[1].icon}
              count={outputCount}
              unit={outputCount === 1 ? "output" : "outputs"}
              open={step === 1}
              done={step > 1}
              {revealFrom}
              onOpen={() => openStep(1)}
            >
              <NamingPanel
                presetTemplate={appState.presetTemplate}
                amsTemplate={appState.amsTemplate}
                presetRules={appState.presetRules}
                amsRules={appState.amsRules}
                savedPresets={savedNamingPresets}
                preview={namePreview}
                {previewSubject}
                error={namePreviewError}
                outputs={appState.outputs}
                onTemplatesChanged={changeTemplates}
                onRulesChanged={changeRules}
                onOutputsChanged={changeOutputs}
                onSavePreset={saveNamingPreset}
                onLoadPreset={loadNamingPreset}
                onDeletePreset={deleteNamingPreset}
              />
            </StepRail>

            <StepRail
              id={STEP_IDS[2]}
              index={2}
              class="destination-rail"
              title={STEPS[2].title}
              description={STEPS[2].description}
              icon={STEPS[2].icon}
              count={selectedNozzleCount}
              unit={selectedNozzleCount === 1 ? "target" : "targets"}
              open={step === 2}
              done={false}
              {revealFrom}
              onOpen={() => openStep(2)}
            >
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
            </StepRail>
          </div>

          <RouteBar
            {step}
            steps={STEPS}
            sources={appState.selectedSourceIds.size}
            targets={selectedNozzleCount}
            outputs={outputCount}
            ready={readiness.ready}
            readinessLabel={readiness.label}
            building={appState.phase === "planning"}
            onBack={() => openStep(step - 1)}
            onContinue={continueRoute}
          />
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
          class="workspace-view document-view"
          role="tabpanel"
          aria-labelledby="activity-tab"
          tabindex="-1"
        >
          <div class="section-head">
            <h2>Follow every step</h2>
            <p>Local execution and cloud-facing evidence stay separate.</p>
          </div>
          <div class="run-layout">
            <RunRail {rail} />
            <div class="run-main">
              <RunStatus
                result={appState.result}
                executing={appState.phase === "executing"}
                localError={appState.executionFailed ? appState.error : null}
                synchronization={appState.synchronization}
                syncError={appState.syncError}
                syncing={appState.phase === "synchronizing"}
                onRetrySync={synchronizeRun}
                onOpenRestore={previewRestore}
              />
              <RunProgress
                plan={appState.plan}
                progress={appState.progress}
                running={appState.phase === "executing" ||
                  appState.phase === "synchronizing"}
                cancellable={appState.phase === "synchronizing"}
                onCancel={cancelSynchronization}
                onVerify={verifyAms}
              />
              {#if appState.result}
                <ResultSummary
                  result={appState.result}
                  synchronization={appState.synchronization}
                  syncError={appState.syncError}
                  syncing={appState.phase === "synchronizing"}
                  onOpenSupport={api.openSupportPage}
                />
              {/if}
            </div>
          </div>
        </div>
      {:else}
        <div
          id="restore-view"
          class="workspace-view document-view"
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
