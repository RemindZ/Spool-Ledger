<script lang="ts">
  import { onMount } from "svelte";
  import { ArrowRight, CheckCircle2, Route } from "@lucide/svelte";
  import AppChrome from "../../src/lib/components/AppChrome.svelte";
  import ExecutionPhases from "../../src/lib/components/ExecutionPhases.svelte";
  import FilterPanel from "../../src/lib/components/FilterPanel.svelte";
  import FirstRunSetup from "../../src/lib/components/FirstRunSetup.svelte";
  import NamingPanel from "../../src/lib/components/NamingPanel.svelte";
  import PlanDependencyDialog from "../../src/lib/components/PlanDependencyDialog.svelte";
  import PlanTable from "../../src/lib/components/PlanTable.svelte";
  import PrinterManagerDialog from "../../src/lib/components/PrinterManagerDialog.svelte";
  import RestorePanel from "../../src/lib/components/RestorePanel.svelte";
  import ResultSummary from "../../src/lib/components/ResultSummary.svelte";
  import RunProgress from "../../src/lib/components/RunProgress.svelte";
  import SourceTable from "../../src/lib/components/SourceTable.svelte";
  import TargetPanel from "../../src/lib/components/TargetPanel.svelte";
  import { createInitialState, sourceFacets } from "../../src/lib/state";
  import type {
    CatalogSource,
    ConflictDecision,
    DiscoveryResponse,
    MigrationPlan,
    PlanOperation,
    PrinterTarget,
  } from "../../src/lib/types";

  const states = [
    ["initial", "1 Initial inventory"],
    ["no-account", "2 No eligible account"],
    ["ready", "3 Ready workspace"],
    ["naming", "4 Advanced naming"],
    ["plan", "5 Mixed plan"],
    ["conflict", "6 Conflict selected"],
    ["execution", "7 Local execution"],
    ["monitoring", "8 Synchronization"],
    ["timeout", "9 Timeout"],
    ["cloud", "10 Cloud IDs"],
    ["ams", "11 AMS verified"],
    ["restore", "12 Restore preview"],
    ["narrow", "13 Narrow layout"],
    ["setup", "14 First-run setup"],
    ["manager", "15 Printer manager"],
    ["dependency", "16 Target dependency"],
    ["artwork-fallback", "17 Artwork fallback"],
  ] as const;
  const query = new URLSearchParams(location.search);
  let state = $state(query.get("state") ?? "ready");
  const theme = query.get("theme") === "dark" ? "dark" : "light";
  document.documentElement.dataset.theme = theme;

  const sources: CatalogSource[] = [
    {
      id: "fixture:source:fiberlogy-satin",
      name: "Fiberlogy Easy PLA Satin @BBL X1C",
      vendor: "Fiberlogy",
      material: "PLA",
      family: "Easy PLA",
      variant: "Satin",
      source_app: "orca_slicer",
      source_kind: "factory_system",
      compatible_printers: ["Bambu Lab H2C"],
      migration_status: "new",
      warnings: [],
    },
    {
      id: "fixture:source:overture-matte",
      name: "Overture Matte PLA @BBL H2C",
      vendor: "Overture",
      material: "PLA",
      family: "Matte PLA",
      variant: "Black",
      source_app: "orca_slicer",
      source_kind: "user_custom",
      compatible_printers: ["Bambu Lab H2C"],
      migration_status: "conflicting",
      warnings: ["Existing destination settings differ"],
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
      extruder_variants: ["Direct Drive Standard"],
      nozzles: ["0.2", "0.4", "0.6", "0.8"].map((diameter) => ({
        id: `official:H2C:${diameter}`,
        diameter,
        printer_preset_name: `Bambu Lab H2C ${diameter} nozzle`,
        selected: diameter === "0.4",
        supported: true,
      })),
    },
  ];

  const discovery: DiscoveryResponse = {
    source_root_ids: ["fixture-orca"],
    manual_source_roots: [],
    target_catalog_ids: ["fixture-bambu"],
    orca_slicer_detected: true,
    bambu_studio_detected: true,
    platform: "windows",
    accounts: [
      {
        id: "fixture-account",
        eligibility: "eligible",
        filament_profile_count: 8,
      },
      {
        id: "fixture-backup",
        eligibility: "backup",
        filament_profile_count: 31,
      },
    ],
  };

  function operation(
    id: string,
    action: PlanOperation["action"],
    nozzle: string,
  ): PlanOperation {
    return {
      id,
      source_id: sources[0].id,
      source_name: sources[0].name,
      preset_name: `Easy PLA Satin - H2C ${nozzle}`,
      ams_name: "Fiberlogy PLA Easy PLA Satin",
      filament_id: `fixture-filament-${nozzle}`,
      printer_id: printers[0].id,
      printer_name: printers[0].name,
      printer_preset_name: `Bambu Lab H2C ${nozzle} nozzle`,
      nozzle,
      custom_unverified: false,
      source_precondition_fingerprint: `fixture-source-${nozzle}`,
      material_settings_fingerprint: `fixture-material-${nozzle}`,
      precondition_fingerprint: null,
      identity_fingerprint: {
        name: "fiberlogy pla easy pla satin",
        printer: `bambu lab h2c ${nozzle} nozzle`,
        nozzle,
      },
      action,
      conflict:
        action === "block"
          ? { kind: "settings_mismatch", message: "Existing settings differ" }
          : null,
    };
  }

  const plan: MigrationPlan = {
    id: "fixture-plan-2026-08-21",
    operations: [
      operation("fixture-op-create", "create", "0.2"),
      operation("fixture-op-target", "add_target", "0.4"),
      operation("fixture-op-skip", "skip", "0.6"),
      operation("fixture-op-block", "block", "0.8"),
    ],
  };
  const conflictDecisions: ConflictDecision[] =
    state === "conflict"
      ? [
          {
            source_id: sources[0].id,
            printer_id: printers[0].id,
            nozzle: "0.8",
            choice: "update",
          },
        ]
      : [];
  const result = {
    run_id: "fixture-run-2026-08-21",
    plan_id: plan.id,
    committed_files: 6,
    created_files: 4,
    updated_files: 2,
    deleted_files: 0,
    skipped_operations: 1,
    backup_sha256:
      "4ae91dbcb7c65eff3b276ac29a01360d5d33905fcfd3944ed83ac356516785ed",
    backup_file_count: 6,
    receipt_path: "fixture/account/.bambu-filament-migrator/receipt.json",
  };
  const restorePreview = {
    paths: [
      {
        path: "fixture/account/filament/Fiberlogy Easy PLA Satin.json",
        action: "create" as const,
        safe_to_restore: true,
        current_sha256: "b7c65eff3b276ac29a01360d5d33905f",
        expected_committed_sha256: "b7c65eff3b276ac29a01360d5d33905f",
      },
      {
        path: "fixture/account/filament/base/Fiberlogy Easy PLA Satin.info",
        action: "update" as const,
        safe_to_restore: false,
        current_sha256: "operator-change",
        expected_committed_sha256: "committed-by-fixture-run",
      },
    ],
  };
  const initial = createInitialState();
  const selectedSources = new Set(sources.map((source) => source.id));
  const selectedNozzles = { "official:H2C": new Set(["0.4", "0.6"]) };
  const dependencyIssue = {
    id: "fixture-pet-cf-h2c",
    expected_name: "Generic PET-CF @BBL H2C",
    material: "PET-CF",
    printer_id: "official:H2C",
    printer_name: "Bambu Lab H2C",
    nozzle: "0.4",
    affected_sources: [
      { id: "fixture:elegoo-pet-cf", name: "Elegoo PET-CF" },
      { id: "fixture:other-pet-cf", name: "Fiberlogy PET-CF" },
    ],
    installed_candidates: [
      { profile_name: "Bambu PET-CF @BBL H2C", recommended: true },
    ],
    source_candidates: [],
    diagnostic:
      "Generic PET-CF @BBL H2C is not installed for Bambu Lab H2C 0.4 mm",
  };
  const facets = sourceFacets(sources);

  function chooseState(next: string) {
    const url = new URL(location.href);
    url.searchParams.set("state", next);
    location.href = url.toString();
  }

  onMount(() => {
    if (state === "naming") {
      document
        .querySelector<HTMLDetailsElement>(".advanced-rules")
        ?.setAttribute("open", "");
    }
  });
</script>

<div class="visual-fixture-controls">
  <label>
    Visual state
    <select
      value={state}
      onchange={(event) => chooseState(event.currentTarget.value)}
    >
      {#each states as item (item[0])}
        <option value={item[0]}>{item[1]}</option>
      {/each}
    </select>
  </label>
</div>

<div class="app-frame">
  {#if state === "initial" || state === "setup"}
    <header class="boot-brand">
      <img
        class="boot-brand-lockup"
        src="/spool-ledger-lockup-transparent.png"
        alt="Spool Ledger · Bambu Filament Migrator"
      />
    </header>
  {/if}
  {#if state !== "initial" && state !== "setup"}
    <AppChrome
      discovery={state === "no-account"
        ? {
            ...discovery,
            accounts: [
              {
                id: "fixture-backup",
                eligibility: "backup",
                filament_profile_count: 31,
              },
            ],
          }
        : discovery}
      selectedAccountId={state === "no-account" ? null : "fixture-account"}
      {theme}
      busy={false}
      activeView={state === "plan" || state === "conflict"
        ? "plan"
        : state === "restore"
          ? "restore"
          : ["execution", "monitoring", "timeout", "cloud", "ams"].includes(
                state,
              )
            ? "activity"
            : "setup"}
      planCount={plan.operations.length}
      onRefresh={() => {}}
      onAccountChanged={() => {}}
      onThemeChanged={() => {}}
    />
  {/if}

  {#if state === "initial"}
    <main class="catalog-load-state">
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
        <p class="section-kicker">Local profile inventory</p>
        <h2>Reading OrcaSlicer profiles</h2>
        <div class="catalog-progress-copy">
          <strong>642 / 903 profiles</strong><span>71%</span>
        </div>
        <div
          class="catalog-progress-track"
          role="progressbar"
          aria-label="Profile loading progress"
          aria-valuenow="642"
          aria-valuemin="0"
          aria-valuemax="903"
        >
          <span style="width:71%"></span>
        </div>
        <p>No profile files are changed while this inventory is built.</p>
      </div>
    </main>
  {:else if state === "setup"}
    <FirstRunSetup
      catalogId="fixture-targets"
      {sources}
      {printers}
      onComplete={() => {}}
    />
  {:else}
    <main class="workspace-shell">
      {#if state === "no-account"}
        <div class="view-empty">
          <h2>No eligible destination account</h2>
          <p>
            Bambu Studio was detected, but no active account profile root is
            eligible for local migration.
          </p>
        </div>
      {:else if ["ready", "naming", "narrow", "manager", "dependency", "artwork-fallback"].includes(state)}
        <div class="workspace-view">
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
              <li class="ready"><span>1</span>Sources</li>
              <li class="ready"><span>2</span>Select</li>
              <li class="ready"><span>3</span>Name</li>
              <li class="ready"><span>4</span>Hardware</li>
            </ol>
            <p class="discovery-status">
              <CheckCircle2 size={15} /> Profile inventory ready
            </p>
          </header>
          <div class="routing-bench" aria-label="Filament migration route">
            <article class="route-rail source-rail">
              <div class="source-rail-body">
                <FilterPanel
                  filters={initial.filters}
                  {facets}
                  onFiltersChanged={() => {}}
                  onReset={() => {}}
                />
                <div class="source-workbench">
                  <SourceTable
                    {sources}
                    selectedIds={selectedSources}
                    totalCount={sources.length}
                    onToggle={() => {}}
                    onSelectVisible={() => {}}
                    onClearSelection={() => {}}
                  />
                </div>
              </div>
            </article>
            <div class="route-thread" aria-hidden="true">
              <span></span><ArrowRight size={15} />
            </div>
            <article class="route-rail naming-rail">
              <NamingPanel
                presetTemplate={"{clean_name} - {printer_code}"}
                amsTemplate={"{vendor} {material} {clean_name}"}
                presetRules={[]}
                amsRules={state === "naming"
                  ? [
                      {
                        id: "fixture-rule",
                        kind: "wildcard",
                        pattern: "Fiberlogy PLA *",
                        replacement: "$1",
                        case_sensitive: false,
                        condition: null,
                      },
                    ]
                  : []}
                preview={{
                  preset_before: sources[0].name,
                  preset_name: "Easy PLA Satin - H2C",
                  ams_before: sources[0].name,
                  ams_name: "Fiberlogy PLA Easy PLA Satin",
                }}
                onTemplatesChanged={() => {}}
              />
            </article>
            <div class="route-thread destination" aria-hidden="true">
              <span></span><Route size={15} />
            </div>
            <article class="route-rail destination-rail">
              <TargetPanel
                catalogId="fixture-targets"
                printers={state === "artwork-fallback"
                  ? printers.map((printer) => ({
                      ...printer,
                      artwork_available: true,
                    }))
                  : printers}
                enabledPrinterIds={new Set(["official:H2C"])}
                {selectedNozzles}
                onManage={() => {}}
                onNozzleToggle={() => {}}
                onSelectAll={() => {}}
              />
            </article>
          </div>
          <div class="setup-actionbar">
            <div class="setup-counts">
              <span><strong>2</strong> sources</span><span
                ><strong>2</strong> nozzles</span
              ><span><strong>2</strong> outputs</span>
            </div>
            <span class="plan-readiness ready">Ready to build</span><button
              class="primary-button"
              type="button">Build migration plan</button
            >
          </div>
        </div>
      {:else if state === "plan" || state === "conflict"}
        <div class="workspace-view">
          <PlanTable
            {plan}
            {conflictDecisions}
            onOverride={() => {}}
            onDecision={() => {}}
          />
          <div class="plan-actionbar">
            <div>
              <strong>Frozen plan <code>{plan.id}</code></strong>
              <p>
                Local generation cannot guarantee cloud persistence or AMS
                visibility.
              </p>
            </div>
            <button class="primary-button" type="button" disabled
              >Commit migration</button
            >
          </div>
        </div>
      {:else if state === "restore"}
        <div class="workspace-view">
          <RestorePanel
            {result}
            preview={restorePreview}
            onPreview={() => {}}
            onRestore={() => {}}
          />
        </div>
      {:else}
        <div class="workspace-view">
          <ExecutionPhases
            localPhase={state === "execution" ? "backing_up" : "finished"}
            syncPhase={state === "monitoring"
              ? "monitoring"
              : ["timeout", "cloud", "ams"].includes(state)
                ? "finished"
                : null}
          />
          <RunProgress
            {plan}
            running={state === "execution" || state === "monitoring"}
            localPhase={state === "execution" ? "backing_up" : null}
            cancellable={state === "monitoring"}
            progress={Object.fromEntries(
              plan.operations
                .filter(
                  (item) => item.action !== "block" && item.action !== "skip",
                )
                .map((item) => [
                  item.id,
                  {
                    evidence:
                      state === "ams"
                        ? "ams_verified"
                        : state === "cloud"
                          ? "cloud_id_assigned"
                          : state === "monitoring"
                            ? "loaded_by_bambu"
                            : "created_local",
                    state:
                      state === "ams"
                        ? "ams_verified"
                        : state === "cloud"
                          ? "cloud_id_assigned"
                          : state === "monitoring"
                            ? "loaded_by_bambu"
                            : "created_local",
                  },
                ]),
            )}
          />
          {#if ["timeout", "cloud", "ams"].includes(state)}<ResultSummary
              {result}
              synchronization={{
                timed_out: state === "timeout",
                highest_evidence:
                  state === "timeout"
                    ? "created_local"
                    : state === "ams"
                      ? "ams_verified"
                      : "cloud_id_assigned",
                observations: [],
              }}
              onRetrySync={() => {}}
              onOpenRestore={() => {}}
              onOpenSupport={async () => {}}
            />{/if}
        </div>
      {/if}
    </main>

    {#if state === "manager"}
      <PrinterManagerDialog
        open={true}
        catalogId="fixture-targets"
        {printers}
        enabledPrinterIds={new Set(["official:H2C"])}
        {selectedNozzles}
        onSave={() => {}}
        onCancel={() => {}}
      />
    {/if}

    {#if state === "dependency"}
      <PlanDependencyDialog
        open={true}
        issues={[dependencyIssue]}
        busy={false}
        onCancel={() => {}}
        onResolve={() => {}}
        onRemove={() => {}}
      />
    {/if}
  {/if}
</div>

<style>
  .visual-fixture-controls {
    position: fixed;
    z-index: 100;
    right: 10px;
    bottom: 10px;
    padding: 7px;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    background: var(--surface-raised);
    box-shadow: var(--shadow);
  }
  .visual-fixture-controls label {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-soft);
    font-size: 0.65rem;
  }
</style>
