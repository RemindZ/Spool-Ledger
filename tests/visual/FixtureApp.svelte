<script lang="ts">
  import { onMount } from "svelte";
  import AppChrome from "../../src/lib/components/AppChrome.svelte";
  import FilamentPicker from "../../src/lib/components/FilamentPicker.svelte";
  import FirstRunSetup from "../../src/lib/components/FirstRunSetup.svelte";
  import NamingPanel from "../../src/lib/components/NamingPanel.svelte";
  import PlanDependencyDialog from "../../src/lib/components/PlanDependencyDialog.svelte";
  import PlanTable from "../../src/lib/components/PlanTable.svelte";
  import PrinterManagerDialog from "../../src/lib/components/PrinterManagerDialog.svelte";
  import RestorePanel from "../../src/lib/components/RestorePanel.svelte";
  import ResultSummary from "../../src/lib/components/ResultSummary.svelte";
  import RouteBar from "../../src/lib/components/RouteBar.svelte";
  import RunProgress from "../../src/lib/components/RunProgress.svelte";
  import RunRail from "../../src/lib/components/RunRail.svelte";
  import RunStatus from "../../src/lib/components/RunStatus.svelte";
  import StepRail from "../../src/lib/components/StepRail.svelte";
  import TargetPanel from "../../src/lib/components/TargetPanel.svelte";
  import UpdateNotice from "../../src/lib/components/UpdateNotice.svelte";
  import { runRail, type PhaseMark } from "../../src/lib/run-rail";
  import { defaultFilters, type OperationProgress } from "../../src/lib/state";
  import { STEP_IDS, WORKSPACE_STEPS } from "../../src/lib/steps";
  import type {
    CatalogSource,
    ConflictDecision,
    DiscoveryResponse,
    EvidenceLevel,
    ExecutionPhase,
    MigrationPlan,
    OperationState,
    PlanOperation,
    PrinterTarget,
    SourceApp,
    SourceKind,
    SyncPhase,
    SyncResult,
  } from "../../src/lib/types";

  const states = [
    ["initial", "1 Initial inventory"],
    ["no-account", "2 No eligible account"],
    ["ready", "3 Filament materials"],
    ["manufacturer-menu", "4 Manufacturer menu"],
    ["presets", "5 Filament presets"],
    ["naming", "6 Naming step"],
    ["printers", "7 Printers step"],
    ["plan", "8 Mixed plan"],
    ["conflict", "9 Conflict selected"],
    ["run-idle", "10 Run idle"],
    ["execution", "11 Local execution"],
    ["commit-failed", "12 Commit failed"],
    ["monitoring", "13 Synchronization"],
    ["timeout", "14 Timeout"],
    ["cloud", "15 Cloud IDs"],
    ["ams", "16 AMS verified"],
    ["restore", "17 Restore preview"],
    ["narrow", "18 Narrow layout"],
    ["setup", "19 First-run setup"],
    ["manager", "20 Printer manager"],
    ["dependency", "21 Target dependency"],
    ["artwork-fallback", "22 Artwork fallback"],
    ["update", "23 Update available"],
  ] as const;
  const query = new URLSearchParams(location.search);
  let state = $state(query.get("state") ?? "ready");
  const theme = query.get("theme") === "dark" ? "dark" : "light";
  document.documentElement.dataset.theme = theme;

  // Synthetic demonstration catalog. Vendors and products are fictional or
  // documentation-approved; nothing here is read from a real profile root.
  function synthetic(
    vendor: string,
    product: string,
    archetype: string,
    type: string,
    app: SourceApp = "orca_slicer",
    kind: SourceKind = "factory_system",
  ): CatalogSource {
    const slug = `${vendor}-${product}`
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-");
    return {
      id: `fixture:source:${slug}`,
      name: `${vendor} ${product}`,
      vendor,
      material: archetype,
      material_archetype: archetype,
      material_type: type,
      family: product,
      variant: type === "Standard" ? "" : type,
      source_app: app,
      source_kind: kind,
      compatible_printers: ["Bambu Lab H2C 0.4 nozzle"],
      migration_status: "new",
      warnings: [],
    };
  }

  const sources: CatalogSource[] = [
    synthetic("Fiberlogy", "Easy PLA Satin", "PLA", "Standard"),
    synthetic("Overture", "PLA", "PLA", "Standard"),
    synthetic("Northstar", "PLA Aurora", "PLA", "Standard"),
    synthetic("Fiberlogy", "PLA Matte", "PLA", "Matte"),
    {
      ...synthetic(
        "Overture",
        "Matte PLA",
        "PLA",
        "Matte",
        "orca_slicer",
        "user_custom",
      ),
      migration_status: "conflicting",
      warnings: ["Existing destination settings differ"],
    },
    synthetic("Northstar", "PLA Matte", "PLA", "Matte", "bambu_studio"),
    synthetic("Fiberlogy", "Silk PLA", "PLA", "Silk"),
    synthetic("Overture", "Silk PLA", "PLA", "Silk"),
    synthetic("Overture", "Rapid PLA", "PLA", "High speed"),
    synthetic("Northstar", "PLA HS", "PLA", "High speed", "bambu_studio"),
    synthetic("Fiberlogy", "Easy PETG", "PETG", "Standard"),
    synthetic("Overture", "PETG", "PETG", "Standard"),
    synthetic(
      "Overture",
      "PETG HS",
      "PETG",
      "High speed",
      "orca_slicer",
      "user_custom",
    ),
    synthetic("Northstar", "PETG-CF", "PETG", "Carbon fiber", "bambu_studio"),
    synthetic("Fiberlogy", "PCTG", "PCTG", "Standard"),
    synthetic("Overture", "ABS", "ABS", "Standard"),
    synthetic("Fiberlogy", "ABS", "ABS", "Standard"),
    synthetic("Northstar", "ABS-GF", "ABS", "Glass fiber", "bambu_studio"),
    synthetic("Overture", "ASA", "ASA", "Standard"),
    synthetic("Fiberlogy", "ASA", "ASA", "Standard"),
    synthetic("Overture", "TPU 95A", "TPU", "Shore 95A"),
    synthetic("Northstar", "TPU for AMS", "TPU", "For AMS", "bambu_studio"),
    synthetic("Fiberlogy", "PA12-CF", "PA", "Carbon fiber"),
    synthetic("Northstar", "PA6-GF", "PA", "Glass fiber", "bambu_studio"),
    synthetic("Overture", "PC", "PC", "Standard"),
  ];
  const selectedSources = new Set([
    sources[0].id,
    sources[3].id,
    sources[11].id,
  ]);

  const printers: PrinterTarget[] = [
    {
      id: "official:H2C",
      name: "Bambu Lab H2C",
      code: "H2C",
      kind: "official",
      verified: true,
      artwork_available: false,
      extruder_variants: ["Direct Drive Standard", "Direct Drive High Flow"],
      nozzles: ["0.2", "0.4", "0.6", "0.8"].map((diameter) => ({
        id: `official:H2C:${diameter}`,
        diameter,
        printer_preset_name: `Bambu Lab H2C ${diameter} nozzle`,
        selected: diameter === "0.4",
        supported: true,
      })),
    },
    {
      id: "official:P1S",
      name: "Bambu Lab P1S",
      code: "P1S",
      kind: "official",
      verified: true,
      artwork_available: false,
      extruder_variants: ["Direct Drive Standard"],
      nozzles: ["0.2", "0.4", "0.6", "0.8"].map((diameter) => ({
        id: `official:P1S:${diameter}`,
        diameter,
        printer_preset_name: `Bambu Lab P1S ${diameter} nozzle`,
        selected: false,
        supported: diameter !== "0.2",
      })),
    },
  ];
  const enabledPrinters = new Set(["official:H2C", "official:P1S"]);

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
  const selectedNozzles = {
    "official:H2C": new Set(["0.4", "0.6"]),
  };
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

  const runStates = [
    "run-idle",
    "execution",
    "commit-failed",
    "monitoring",
    "timeout",
    "cloud",
    "ams",
  ];
  const workspaceStates = [
    "ready",
    "manufacturer-menu",
    "presets",
    "naming",
    "printers",
    "narrow",
    "manager",
    "dependency",
    "artwork-fallback",
    "update",
  ];
  const step = ["naming"].includes(state)
    ? 1
    : ["printers", "manager", "artwork-fallback"].includes(state)
      ? 2
      : 0;
  const synced = ["timeout", "cloud", "ams"].includes(state);
  const committed = ["monitoring", ...["timeout", "cloud", "ams"]].includes(
    state,
  );

  // Fixture phase starts in milliseconds; they label a synthetic run only.
  const localTimeline: PhaseMark<ExecutionPhase>[] = (
    [
      ["validating", 0],
      ["closing_bambu", 140],
      ["staging", 1240],
      ["validating_output", 1610],
      ["backing_up", 1830],
      ["committing", 2420],
      ["writing_receipt", 2690],
      ["finished", 2860],
    ] as const
  )
    .slice(
      0,
      state === "run-idle"
        ? 0
        : state === "execution"
          ? 5
          : state === "commit-failed"
            ? 6
            : 8,
    )
    .map(([phase, elapsed_ms]) => ({ phase, elapsed_ms }));
  const syncTimeline: PhaseMark<SyncPhase>[] = committed
    ? (
        [
          ["launching", 0],
          ["monitoring", 2310],
          ["finished", 9480],
        ] as const
      )
        .slice(0, synced && state !== "timeout" ? 3 : 2)
        .map(([phase, elapsed_ms]) => ({ phase, elapsed_ms }))
    : [];
  const rail = runRail({
    local: localTimeline,
    sync: syncTimeline,
    executing: state === "execution",
    localFailed: state === "commit-failed",
    syncing: state === "monitoring",
    syncExpected: true,
    syncStopped: state === "timeout",
  });
  const fixtureSynchronization: SyncResult | null = synced
    ? {
        timed_out: state === "timeout",
        highest_evidence:
          state === "timeout"
            ? "created_local"
            : state === "ams"
              ? "ams_verified"
              : "cloud_id_assigned",
        observations: [],
      }
    : null;
  const evidence: EvidenceLevel =
    state === "ams"
      ? "ams_verified"
      : state === "cloud"
        ? "cloud_id_assigned"
        : state === "monitoring"
          ? "loaded_by_bambu"
          : "created_local";
  const fixtureProgress: Record<string, OperationProgress> =
    state === "run-idle" || state === "execution" || state === "commit-failed"
      ? {}
      : Object.fromEntries(
          plan.operations
            .filter((item) => item.action !== "block" && item.action !== "skip")
            .map((item) => [
              item.id,
              { evidence, state: evidence as OperationState },
            ]),
        );

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
    if (state === "manufacturer-menu") {
      document
        .querySelector<HTMLDetailsElement>("details.vendor-menu")
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
      activeView={state === "plan" || state === "conflict"
        ? "plan"
        : state === "restore"
          ? "restore"
          : runStates.includes(state)
            ? "activity"
            : "setup"}
      planCount={plan.operations.length}
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
      {:else if workspaceStates.includes(state)}
        <div class="workspace-view setup-view">
          <h1 class="sr-only">Migration workspace</h1>
          <div class="route-shell" aria-label="Migration setup steps">
            <StepRail
              id={STEP_IDS[0]}
              index={0}
              class="source-rail"
              title={WORKSPACE_STEPS[0].title}
              description={WORKSPACE_STEPS[0].description}
              icon={WORKSPACE_STEPS[0].icon}
              count={selectedSources.size}
              unit="selected"
              open={step === 0}
              done={step > 0}
              onOpen={() => {}}
            >
              <FilamentPicker
                {sources}
                filters={defaultFilters()}
                selectedIds={selectedSources}
                initialPath={state === "presets" ? ["PLA", "Matte"] : []}
                {discovery}
                onFiltersChanged={() => {}}
                onToggle={() => {}}
                onSelect={() => {}}
                onClearSelection={() => {}}
                onSavePreset={() => {}}
                onLoadPreset={() => {}}
                onDeletePreset={() => {}}
                onRefresh={() => {}}
                onAddSource={() => {}}
                onRemoveSource={() => {}}
              />
            </StepRail>
            <StepRail
              id={STEP_IDS[1]}
              index={1}
              class="naming-rail"
              title={WORKSPACE_STEPS[1].title}
              description={WORKSPACE_STEPS[1].description}
              icon={WORKSPACE_STEPS[1].icon}
              count={2}
              unit="outputs"
              open={step === 1}
              done={step > 1}
              onOpen={() => {}}
            >
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
                  preset_before: "Easy PLA Satin - H2C",
                  preset_name: "Easy PLA Satin - H2C",
                  ams_before: "Fiberlogy PLA Easy PLA Satin",
                  ams_name:
                    state === "naming"
                      ? "Easy PLA Satin"
                      : "Fiberlogy PLA Easy PLA Satin",
                }}
                previewSubject={`${sources[0].name} on Bambu Lab H2C 0.4 mm`}
                onTemplatesChanged={() => {}}
              />
            </StepRail>
            <StepRail
              id={STEP_IDS[2]}
              index={2}
              class="destination-rail"
              title={WORKSPACE_STEPS[2].title}
              description={WORKSPACE_STEPS[2].description}
              icon={WORKSPACE_STEPS[2].icon}
              count={2}
              unit="targets"
              open={step === 2}
              done={false}
              onOpen={() => {}}
            >
              <TargetPanel
                catalogId="fixture-targets"
                printers={state === "artwork-fallback"
                  ? printers.map((printer) => ({
                      ...printer,
                      artwork_available: true,
                    }))
                  : printers}
                enabledPrinterIds={enabledPrinters}
                {selectedNozzles}
                onManage={() => {}}
                onNozzleToggle={() => {}}
                onSelectAll={() => {}}
              />
            </StepRail>
          </div>
          <RouteBar
            {step}
            steps={WORKSPACE_STEPS}
            sources={selectedSources.size}
            targets={2}
            outputs={2}
            ready={true}
            readinessLabel={step === 2 ? "Ready to build" : "Ready to continue"}
            onBack={() => {}}
            onContinue={() => {}}
          />
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
        <div class="workspace-view document-view">
          <RestorePanel
            {result}
            preview={restorePreview}
            onPreview={() => {}}
            onRestore={() => {}}
          />
        </div>
      {:else}
        <div id="activity-view" class="workspace-view document-view">
          <div class="section-head">
            <h2>Follow every step</h2>
            <p>Local execution and cloud-facing evidence stay separate.</p>
          </div>
          <div class="run-layout">
            <RunRail {rail} />
            <div class="run-main">
              <RunStatus
                result={committed ? result : null}
                executing={state === "execution"}
                localError={state === "commit-failed"
                  ? "Committing the staged files failed. The journal restored every changed path."
                  : null}
                synchronization={fixtureSynchronization}
                syncing={state === "monitoring"}
                onRetrySync={() => {}}
                onOpenRestore={() => {}}
              />
              <RunProgress
                plan={state === "run-idle" ? null : plan}
                running={state === "execution" || state === "monitoring"}
                cancellable={state === "monitoring"}
                progress={fixtureProgress}
              />
              {#if synced}<ResultSummary
                  {result}
                  synchronization={fixtureSynchronization}
                  onOpenSupport={async () => {}}
                />{/if}
            </div>
          </div>
        </div>
      {/if}
    </main>

    {#if state === "update"}
      <UpdateNotice
        update={{
          status: "available",
          current: "0.9.0",
          latest: "0.10.0",
          tag: "v0.10.0",
          url: "https://github.com/RemindZ/Spool-Ledger/releases/tag/v0.10.0",
        }}
        onUpdate={() => {}}
        onDismiss={() => {}}
        onSkip={() => {}}
      />
    {/if}

    {#if state === "manager"}
      <PrinterManagerDialog
        open={true}
        catalogId="fixture-targets"
        {printers}
        enabledPrinterIds={enabledPrinters}
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
