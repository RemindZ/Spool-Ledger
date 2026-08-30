<script lang="ts">
  import { untrack } from "svelte";
  import { ArrowRight, Database, SlidersHorizontal } from "@lucide/svelte";
  import type {
    CatalogSource,
    PrinterTarget,
    SourceApp,
    SourceKind,
  } from "../types";
  import PrinterPicker from "./PrinterPicker.svelte";

  let {
    catalogId,
    sources,
    printers,
    onComplete,
  }: {
    catalogId: string;
    sources: CatalogSource[];
    printers: PrinterTarget[];
    onComplete: (selection: {
      sourceApps: Set<SourceApp>;
      sourceKinds: Set<SourceKind>;
      enabledPrinterIds: Set<string>;
      selectedNozzles: Record<string, Set<string>>;
    }) => void;
  } = $props();

  let sourceApps = $state(
    untrack(
      () => new Set<SourceApp>(sources.map((source) => source.source_app)),
    ),
  );
  let sourceKinds = $state(
    untrack(
      () => new Set<SourceKind>(sources.map((source) => source.source_kind)),
    ),
  );
  let enabledPrinterIds = $state(new Set<string>());
  let selectedNozzles = $state<Record<string, Set<string>>>({});

  const matchingSources = $derived(
    sources.filter(
      (source) =>
        sourceApps.has(source.source_app) &&
        sourceKinds.has(source.source_kind),
    ),
  );
  const everyEnabledPrinterHasNozzle = $derived(
    [...enabledPrinterIds].every(
      (printerId) => (selectedNozzles[printerId]?.size ?? 0) > 0,
    ),
  );
  const canContinue = $derived(
    sourceApps.size > 0 &&
      sourceKinds.size > 0 &&
      matchingSources.length > 0 &&
      enabledPrinterIds.size > 0 &&
      everyEnabledPrinterHasNozzle,
  );

  function countApp(app: SourceApp): number {
    return sources.filter((source) => source.source_app === app).length;
  }

  function countKind(kind: SourceKind): number {
    return sources.filter((source) => source.source_kind === kind).length;
  }

  function countLabel(count: number): string {
    return `${count} ${count === 1 ? "profile" : "profiles"}`;
  }

  function toggleApp(app: SourceApp) {
    sourceApps = sourceApps.has(app)
      ? new Set([...sourceApps].filter((candidate) => candidate !== app))
      : new Set([...sourceApps, app]);
  }

  function toggleKind(kind: SourceKind) {
    sourceKinds = sourceKinds.has(kind)
      ? new Set([...sourceKinds].filter((candidate) => candidate !== kind))
      : new Set([...sourceKinds, kind]);
  }

  function changePrinters(
    enabled: Set<string>,
    nozzles: Record<string, Set<string>>,
  ) {
    enabledPrinterIds = enabled;
    selectedNozzles = nozzles;
  }

  function complete() {
    if (!canContinue) return;
    onComplete({
      sourceApps: new Set(sourceApps),
      sourceKinds: new Set(sourceKinds),
      enabledPrinterIds: new Set(enabledPrinterIds),
      selectedNozzles: Object.fromEntries(
        Object.entries(selectedNozzles).map(([printerId, nozzles]) => [
          printerId,
          new Set(nozzles),
        ]),
      ),
    });
  }
</script>

<main class="first-run-setup" aria-labelledby="first-run-title">
  <header class="first-run-hero">
    <div class="setup-route-mark" aria-hidden="true">
      <span><Database size={20} /></span>
      <i></i>
      <span><SlidersHorizontal size={20} /></span>
    </div>
    <div>
      <p class="section-kicker">First-run defaults</p>
      <h1 id="first-run-title">Set your migration route</h1>
      <p>
        Choose what appears in your workspace and which official Bambu printers
        receive migrated profiles. You can change these printer defaults later.
      </p>
    </div>
  </header>

  <div class="setup-sections">
    <section aria-labelledby="source-app-heading">
      <header>
        <span>1</span>
        <div>
          <h2 id="source-app-heading">Migrate from</h2>
          <p>Use only the profile libraries you want to browse.</p>
        </div>
      </header>
      <div class="setup-choice-grid">
        <label>
          <input
            type="checkbox"
            checked={sourceApps.has("orca_slicer")}
            aria-label={`OrcaSlicer, ${countLabel(countApp("orca_slicer"))}`}
            onchange={() => toggleApp("orca_slicer")}
          />
          <span
            ><strong>OrcaSlicer</strong><small
              >{countLabel(countApp("orca_slicer"))}</small
            ></span
          >
        </label>
        <label>
          <input
            type="checkbox"
            checked={sourceApps.has("bambu_studio")}
            aria-label={`Bambu Studio, ${countLabel(countApp("bambu_studio"))}`}
            onchange={() => toggleApp("bambu_studio")}
          />
          <span
            ><strong>Bambu Studio</strong><small
              >{countLabel(countApp("bambu_studio"))}</small
            ></span
          >
        </label>
      </div>
    </section>

    <section aria-labelledby="source-kind-heading">
      <header>
        <span>2</span>
        <div>
          <h2 id="source-kind-heading">Profile types</h2>
          <p>Include factory presets, your custom profiles, or both.</p>
        </div>
      </header>
      <div class="setup-choice-grid">
        <label>
          <input
            type="checkbox"
            checked={sourceKinds.has("factory_system")}
            aria-label={`Factory / system, ${countLabel(countKind("factory_system"))}`}
            onchange={() => toggleKind("factory_system")}
          />
          <span
            ><strong>Factory / system</strong><small
              >{countLabel(countKind("factory_system"))}</small
            ></span
          >
        </label>
        <label>
          <input
            type="checkbox"
            checked={sourceKinds.has("user_custom")}
            aria-label={`User / custom, ${countLabel(countKind("user_custom"))}`}
            onchange={() => toggleKind("user_custom")}
          />
          <span
            ><strong>User / custom</strong><small
              >{countLabel(countKind("user_custom"))}</small
            ></span
          >
        </label>
      </div>
      {#if sourceApps.size > 0 && sourceKinds.size > 0 && matchingSources.length === 0}
        <p class="setup-validation" role="status">
          No discovered profiles match this source and profile-type combination.
        </p>
      {/if}
    </section>

    <section
      class="printer-setup-section"
      aria-labelledby="default-printers-heading"
    >
      <header>
        <span>3</span>
        <div>
          <h2 id="default-printers-heading">Default printers</h2>
          <p>Enable at least one official printer and its default nozzle.</p>
        </div>
      </header>
      <PrinterPicker
        {catalogId}
        {printers}
        {enabledPrinterIds}
        {selectedNozzles}
        onChange={changePrinters}
      />
    </section>
  </div>

  <footer>
    <p>
      {matchingSources.length} matching {matchingSources.length === 1
        ? "profile"
        : "profiles"}
      · {enabledPrinterIds.size} enabled {enabledPrinterIds.size === 1
        ? "printer"
        : "printers"}
    </p>
    <button
      type="button"
      class="primary-button"
      disabled={!canContinue}
      onclick={complete}
    >
      Continue to migration <ArrowRight size={16} />
    </button>
  </footer>
</main>

<style>
  .first-run-setup {
    display: grid;
    gap: 1rem;
    width: min(70rem, calc(100% - 2rem));
    margin: 1.25rem auto 2rem;
  }

  .first-run-hero {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    align-items: center;
    gap: 1rem;
    padding: 1.15rem 1.25rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
  }

  .first-run-hero h1,
  .first-run-hero p,
  .setup-sections h2,
  .setup-sections p {
    margin: 0;
  }

  .first-run-hero > div:last-child {
    display: grid;
    gap: 0.35rem;
  }

  .first-run-hero > div:last-child > p:last-child,
  .setup-sections section > header p {
    color: var(--text-muted);
  }

  .setup-route-mark {
    display: flex;
    align-items: center;
  }

  .setup-route-mark span {
    display: grid;
    width: 2.6rem;
    height: 2.6rem;
    place-items: center;
    border: 1px solid var(--accent);
    border-radius: 50%;
    color: var(--accent-strong);
    background: var(--accent-soft);
  }

  .setup-route-mark i {
    width: 1.4rem;
    height: 1px;
    background: var(--orca-accent);
  }

  .setup-sections {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
  }

  .setup-sections section {
    display: grid;
    align-content: start;
    gap: 0.85rem;
    padding: 1rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
  }

  .setup-sections section > header {
    display: flex;
    gap: 0.7rem;
  }

  .setup-sections section > header > span {
    display: grid;
    width: 1.75rem;
    height: 1.75rem;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 50%;
    font-size: 0.75rem;
    font-weight: 800;
    color: var(--surface-raised);
    background: var(--accent);
  }

  .setup-sections section > header > div {
    display: grid;
    gap: 0.2rem;
  }

  .printer-setup-section {
    grid-column: 1 / -1;
  }

  .setup-choice-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.55rem;
  }

  .setup-choice-grid label {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    min-width: 0;
    padding: 0.75rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .setup-choice-grid label:has(input:checked) {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .setup-choice-grid span {
    display: grid;
    gap: 0.15rem;
    min-width: 0;
  }

  .setup-choice-grid small {
    color: var(--text-muted);
  }

  .setup-validation {
    padding: 0.6rem 0.7rem;
    border-radius: var(--radius-sm);
    color: var(--warning-strong);
    background: var(--warning-soft);
  }

  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    position: sticky;
    bottom: 0.75rem;
    padding: 0.8rem 1rem;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    box-shadow: var(--shadow-lg);
  }

  footer p {
    margin: 0;
    color: var(--text-muted);
  }

  footer button {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  @media (max-width: 46rem) {
    .first-run-setup {
      width: min(100% - 1rem, 70rem);
      margin-top: 0.5rem;
    }

    .first-run-hero,
    .setup-sections {
      grid-template-columns: 1fr;
    }

    .setup-route-mark {
      display: none;
    }

    .printer-setup-section {
      grid-column: auto;
    }
  }

  @media (max-width: 24rem) {
    .setup-choice-grid {
      grid-template-columns: 1fr;
    }

    footer {
      align-items: stretch;
      flex-direction: column;
    }

    footer button {
      justify-content: center;
      width: 100%;
    }
  }
</style>
