<script lang="ts">
  import {
    AlertTriangle,
    ChevronDown,
    CircleDashed,
    Settings,
  } from "@lucide/svelte";
  import type { PrinterTarget } from "../types";
  import PrinterArtwork from "./PrinterArtwork.svelte";

  let {
    catalogId,
    printers,
    enabledPrinterIds,
    selectedNozzles,
    onManage,
    onNozzleToggle,
    onSelectAll,
  }: {
    catalogId: string;
    printers: PrinterTarget[];
    enabledPrinterIds: Set<string>;
    selectedNozzles: Record<string, Set<string>>;
    onManage: () => void;
    onNozzleToggle: (printerId: string, diameter: string) => void;
    onSelectAll: (printerId: string, selected: boolean) => void;
  } = $props();

  const officialPrinters = $derived(
    printers.filter((printer) => printer.kind === "official"),
  );
  const visiblePrinters = $derived(
    officialPrinters.filter((printer) => enabledPrinterIds.has(printer.id)),
  );
</script>

<section class="panel target-panel" aria-labelledby="target-heading">
  <header class="panel-heading">
    <div>
      <p class="section-kicker">Destination</p>
      <h2 id="target-heading">Printers & nozzles</h2>
    </div>
    <div class="target-heading-actions">
      <span
        class="panel-count"
        aria-label={`${visiblePrinters.length} of ${officialPrinters.length} official printers enabled`}
        >{visiblePrinters.length}/{officialPrinters.length}</span
      >
      <button
        type="button"
        class="icon-button"
        aria-label="Manage enabled printers"
        title="Manage enabled printers"
        onclick={onManage}
      >
        <Settings size={16} />
      </button>
    </div>
  </header>

  <div class="printer-list">
    {#each visiblePrinters as printer (printer.id)}
      <details class="printer-card" open={printer.kind === "official"}>
        <summary>
          <PrinterArtwork {catalogId} {printer} />
          <span class="printer-title">
            <strong>{printer.name}</strong>
            <small
              >{printer.code} · {printer.extruder_variants.length} drive modes</small
            >
          </span>
          <ChevronDown class="summary-chevron" size={16} />
        </summary>
        <div class="nozzle-actions">
          <button
            class="text-button"
            type="button"
            aria-label={`Select all nozzles for ${printer.name}`}
            onclick={() => onSelectAll(printer.id, true)}>Select all</button
          >
          <button
            class="text-button muted"
            type="button"
            onclick={() => onSelectAll(printer.id, false)}>Clear</button
          >
        </div>
        <div class="nozzle-grid">
          {#each printer.nozzles as nozzle (nozzle.id)}
            <label class="nozzle-option" class:unsupported={!nozzle.supported}>
              <input
                type="checkbox"
                aria-label={`${nozzle.diameter} mm`}
                checked={selectedNozzles[printer.id]?.has(nozzle.diameter) ??
                  false}
                disabled={!nozzle.supported}
                onchange={() => onNozzleToggle(printer.id, nozzle.diameter)}
              />
              <span class="nozzle-glyph"
                ><CircleDashed size={19} strokeWidth={1.7} /></span
              >
              <span><strong>{nozzle.diameter}</strong><small>mm</small></span>
              {#if !nozzle.supported}
                <small class="unsupported-reason"
                  ><AlertTriangle size={12} /> Unavailable for this official printer</small
                >
              {/if}
            </label>
          {/each}
        </div>
      </details>
    {:else}
      <div class="panel-empty compact">
        <CircleDashed size={26} />
        <p>
          No printers enabled. Use the settings button to add an official Bambu
          printer.
        </p>
      </div>
    {/each}
  </div>
</section>

<style>
  .target-heading-actions {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .target-heading-actions .icon-button {
    display: grid;
    width: 2rem;
    height: 2rem;
    place-items: center;
    padding: 0;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    background: var(--surface-raised);
  }

  .target-heading-actions .icon-button:hover {
    color: var(--accent-strong);
    border-color: var(--accent);
  }

  .printer-list {
    min-height: 0;
    max-height: 68vh;
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-gutter: stable;
    overscroll-behavior: contain;
  }

  @media (max-height: 48rem) {
    .printer-list {
      max-height: 48vh;
    }
  }

  @media (max-width: 52rem) {
    .printer-list {
      max-height: 54vh;
    }
  }
</style>
