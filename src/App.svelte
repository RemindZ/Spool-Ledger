<script lang="ts">
  import { onMount } from 'svelte';
  import { AlertOctagon, ArrowRight, CheckCircle2, LoaderCircle, Route, ShieldCheck } from '@lucide/svelte';
  import { api } from './lib/api';
  import DiscoveryBar from './lib/components/DiscoveryBar.svelte';
  import FilterPanel from './lib/components/FilterPanel.svelte';
  import NamingPanel from './lib/components/NamingPanel.svelte';
  import PlanTable from './lib/components/PlanTable.svelte';
  import ResultSummary from './lib/components/ResultSummary.svelte';
  import RunProgress from './lib/components/RunProgress.svelte';
  import SourceTable from './lib/components/SourceTable.svelte';
  import TargetPanel from './lib/components/TargetPanel.svelte';
  import {
    activeFilterChips,
    appReducer,
    createInitialState,
    loadTheme,
    persistTheme,
    removeSetValue,
    resetFilters,
    selectedNozzleRequests,
    sourceFacets,
    visibleSources,
    type ActiveFilterChip,
    type AppAction,
    type AppState,
    type SourceFilters,
    type Theme,
  } from './lib/state';
  import type { RestorePreview, RollbackOutcome } from './lib/types';

  const storage = typeof localStorage === 'undefined'
    ? { getItem: () => null, setItem: () => undefined }
    : localStorage;

  let appState: AppState = $state(createInitialState(loadTheme(storage)));
  let namePreview = $state<{ preset_name: string; ams_name: string } | null>(null);
  let namePreviewError = $state<string | null>(null);
  let restorePreview = $state<RestorePreview | null>(null);
  let rollback = $state<RollbackOutcome | null>(null);
  let restoring = $state(false);

  const shownSources = $derived(visibleSources(appState));
  const facets = $derived(sourceFacets(appState.sources));
  const filterChips = $derived(activeFilterChips(appState.filters));
  const nozzleRequests = $derived(selectedNozzleRequests(appState));
  const hasInputs = $derived(
    appState.selectedSourceIds.size > 0 &&
      nozzleRequests.length > 0 &&
      appState.selectedAccountId !== null &&
      appState.sourceCatalogId !== null &&
      appState.targetCatalogId !== null,
  );
  const planBlocked = $derived(appState.plan?.operations.some((item) => item.action === 'block') ?? false);

  function dispatch(action: AppAction) {
    appState = appReducer(appState, action);
  }

  function errorMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }

  function applyTheme(theme: Theme) {
    if (theme === 'system') delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = theme;
    persistTheme(storage, theme);
    dispatch({ type: 'theme_changed', theme });
  }

  async function discover() {
    dispatch({ type: 'discovery_started' });
    try {
      const discovery = await api.discover();
      dispatch({ type: 'discovery_loaded', discovery });
      const [sourceCatalog, targetCatalog] = await Promise.all([
        api.catalogSources(discovery.source_root_ids),
        discovery.target_catalog_ids[0]
          ? api.catalogTargets(discovery.target_catalog_ids[0], appState.showCustomPrinters)
          : Promise.reject(new Error('No Bambu target profile catalog was discovered.')),
      ]);
      dispatch({ type: 'sources_loaded', catalogId: sourceCatalog.catalog_id, sources: sourceCatalog.sources });
      dispatch({ type: 'targets_loaded', catalogId: targetCatalog.catalog_id, printers: targetCatalog.printers });
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    }
  }

  async function setShowCustom(value: boolean) {
    dispatch({ type: 'show_custom_changed', value });
    const catalogId = appState.discovery?.target_catalog_ids[0];
    if (!catalogId) return;
    try {
      const targetCatalog = await api.catalogTargets(catalogId, value);
      dispatch({ type: 'targets_loaded', catalogId: targetCatalog.catalog_id, printers: targetCatalog.printers });
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    }
  }

  function changeFilters(filters: Partial<SourceFilters>) {
    dispatch({ type: 'filters_changed', filters });
  }

  function removeFilter(chip: ActiveFilterChip) {
    if (chip.dimension === 'search') return changeFilters({ search: '' });
    if (chip.dimension === 'selectedOnly') return changeFilters({ selectedOnly: false });
    const current = appState.filters[chip.dimension];
    changeFilters({ [chip.dimension]: removeSetValue(current, chip.value as never) });
  }

  async function updatePreview(preset: string, ams: string) {
    dispatch({ type: 'templates_changed', preset, ams });
    namePreviewError = null;
    const source = appState.sources.find((item) => appState.selectedSourceIds.has(item.id));
    const target = appState.printers.find((item) => appState.selectedNozzles[item.id]?.size);
    const nozzle = target ? [...(appState.selectedNozzles[target.id] ?? [])][0] : undefined;
    if (!source || !target || !nozzle) {
      namePreview = null;
      return;
    }
    try {
      const rows = await api.previewNames({
        preset_template: preset,
        ams_template: ams,
        rows: [{
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
        }],
      });
      namePreview = rows[0] ?? null;
    } catch (error) {
      namePreviewError = errorMessage(error);
    }
  }

  async function buildPlan() {
    if (!hasInputs || !appState.sourceCatalogId || !appState.targetCatalogId || !appState.selectedAccountId) return;
    dispatch({ type: 'planning_started' });
    try {
      const response = await api.buildPlan({
        source_catalog_id: appState.sourceCatalogId,
        source_ids: [...appState.selectedSourceIds].sort(),
        target_catalog_id: appState.targetCatalogId,
        nozzles: nozzleRequests,
        destination_account_id: appState.selectedAccountId,
        preset_template: appState.presetTemplate,
        ams_template: appState.amsTemplate,
      });
      dispatch({ type: 'plan_built', plan: response.plan });
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    }
  }

  async function executePlan() {
    if (!appState.plan || planBlocked) return;
    dispatch({ type: 'execution_started' });
    try {
      const result = await api.executePlan(appState.plan.id);
      for (const operation of appState.plan.operations) {
        if (operation.action !== 'block' && operation.action !== 'skip') {
          dispatch({ type: 'progress_updated', operationId: operation.id, evidence: 'created_local', state: 'created_local' });
        }
      }
      dispatch({ type: 'execution_completed', result });
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    }
  }

  async function previewRestore() {
    if (!appState.result) return;
    try {
      restorePreview = await api.restorePreview(appState.result.run_id);
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    }
  }

  async function restoreOwned() {
    if (!appState.result) return;
    restoring = true;
    try {
      rollback = await api.restoreOwned(appState.result.run_id);
      restorePreview = await api.restorePreview(appState.result.run_id);
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    } finally {
      restoring = false;
    }
  }

  async function verifyAms(operationIds: string[]) {
    if (!appState.result) return;
    try {
      await api.recordAmsVerification(appState.result.run_id, operationIds, 'Verified in Bambu Studio AMS device view');
      dispatch({ type: 'ams_verified', operationIds });
    } catch (error) {
      dispatch({ type: 'failed', message: errorMessage(error) });
    }
  }

  onMount(() => {
    applyTheme(appState.theme);
    void discover();
  });
</script>

<svelte:head>
  <title>Bambu Filament Migrator</title>
  <meta name="description" content="Migrate OrcaSlicer and Bambu Studio filament profiles into Bambu custom filaments." />
</svelte:head>

<div class="app-frame">
  {#if appState.discovery}
    <DiscoveryBar
      discovery={appState.discovery}
      selectedAccountId={appState.selectedAccountId}
      theme={appState.theme}
      busy={appState.phase === 'discovering'}
      onRefresh={discover}
      onAccountChanged={(accountId) => dispatch({ type: 'account_selected', accountId })}
      onThemeChanged={applyTheme}
    />
  {:else}
    <header class="boot-brand"><span class="brand-spool" aria-hidden="true"><i></i></span><h1>Bambu Filament Migrator</h1></header>
  {/if}

  {#if appState.error}
    <div class="error-banner" role="alert">
      <AlertOctagon size={18} />
      <span><strong>Migration stopped</strong>{appState.error}</span>
      {#if !appState.discovery}<button type="button" onclick={discover}>Retry discovery</button>{/if}
    </div>
  {/if}

  {#if !appState.discovery && appState.phase !== 'error'}
    <main class="boot-state" aria-live="polite">
      <span class="scan-spool"><LoaderCircle size={28} /></span>
      <p class="section-kicker">Local profile inventory</p>
      <h2>Reading slicer manifests</h2>
      <p>No profile files are changed during discovery.</p>
    </main>
  {:else if appState.discovery}
    <main class="workspace-shell">
      <div class="workspace-grid">
        <FilterPanel filters={appState.filters} {facets} onFiltersChanged={changeFilters} onReset={() => changeFilters(resetFilters())} />

        <div class="source-workbench">
          {#if filterChips.length}
            <div class="filter-chips" aria-label="Active filters">
              {#each filterChips as chip (chip.id)}<button type="button" onclick={() => removeFilter(chip)}>{chip.label}<span aria-hidden="true">×</span></button>{/each}
            </div>
          {/if}
          <SourceTable
            sources={shownSources}
            selectedIds={appState.selectedSourceIds}
            totalCount={appState.sources.length}
            onToggle={(sourceId) => dispatch({ type: 'source_toggled', sourceId })}
            onSelectVisible={(selected) => dispatch({ type: 'select_visible', selected })}
            onClearSelection={() => dispatch({ type: 'clear_selection' })}
          />
        </div>

        <aside class="routing-rail">
          <div class="filament-route" aria-label="Migration route">
            <span class="route-origin"><i></i>Orca / Bambu sources</span>
            <span class="route-line"><ArrowRight size={15} /></span>
            <span class="route-destination"><Route size={14} />Bambu AMS outputs</span>
          </div>
          <label class="custom-printer-toggle"><input type="checkbox" checked={appState.showCustomPrinters} onchange={(event) => void setShowCustom(event.currentTarget.checked)} /><span>Show custom printers</span><small>Unverified targets stay marked</small></label>
          <TargetPanel
            printers={appState.printers}
            selectedNozzles={appState.selectedNozzles}
            showCustom={appState.showCustomPrinters}
            onNozzleToggle={(printerId, diameter) => dispatch({ type: 'nozzle_toggled', printerId, diameter })}
            onSelectAll={(printerId, selected) => dispatch({ type: 'select_printer_nozzles', printerId, selected })}
          />
          <NamingPanel
            presetTemplate={appState.presetTemplate}
            amsTemplate={appState.amsTemplate}
            preview={namePreview}
            error={namePreviewError}
            onTemplatesChanged={(preset, ams) => void updatePreview(preset, ams)}
          />
        </aside>
      </div>

      {#if appState.plan}
        <PlanTable plan={appState.plan} />
      {/if}

      {#if appState.plan && (appState.phase === 'executing' || Object.keys(appState.progress).length)}
        <RunProgress plan={appState.plan} progress={appState.progress} running={appState.phase === 'executing'} onVerify={verifyAms} />
      {/if}

      {#if appState.result}
        <ResultSummary result={appState.result} {restorePreview} {rollback} {restoring} onPreviewRestore={previewRestore} onRestore={restoreOwned} />
      {/if}
    </main>

    <footer class="action-dock">
      <div class="dock-evidence">
        <ShieldCheck size={17} />
        <span><strong>Staged and journaled</strong><small>Nothing changes until you review and commit the plan.</small></span>
      </div>
      <div class="dock-counts"><span>{appState.selectedSourceIds.size} sources</span><span>{nozzleRequests.reduce((sum, item) => sum + item.diameters.length, 0)} nozzles</span></div>
      {#if !appState.plan}
        <div class="dock-action"><span>{hasInputs ? 'Ready to freeze a deterministic preview' : 'Select at least one source and one nozzle'}</span><button class="primary-button" type="button" disabled={!hasInputs || appState.phase === 'planning'} onclick={buildPlan}>{appState.phase === 'planning' ? 'Building plan…' : 'Build migration plan'}</button></div>
      {:else if !appState.result}
        <div class="dock-action"><span>{planBlocked ? 'Resolve blocked conflicts before writing' : `${appState.plan.operations.length} operations reviewed`}</span><button class="primary-button" type="button" disabled={planBlocked || appState.phase === 'executing'} onclick={executePlan}>{appState.phase === 'executing' ? 'Committing…' : 'Commit migration'}</button></div>
      {:else}
        <div class="dock-action complete"><span><CheckCircle2 size={15} /> Local transaction complete</span></div>
      {/if}
    </footer>
  {/if}
</div>
