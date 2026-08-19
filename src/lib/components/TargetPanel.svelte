<script lang="ts">
  import { AlertTriangle, BadgeCheck, ChevronDown, CircleDashed } from '@lucide/svelte';
  import type { PrinterTarget } from '../types';

  let {
    printers,
    selectedNozzles,
    showCustom,
    onNozzleToggle,
    onSelectAll,
  }: {
    printers: PrinterTarget[];
    selectedNozzles: Record<string, Set<string>>;
    showCustom: boolean;
    onNozzleToggle: (printerId: string, diameter: string) => void;
    onSelectAll: (printerId: string, selected: boolean) => void;
  } = $props();

  const visiblePrinters = $derived(printers.filter((printer) => showCustom || printer.kind === 'official'));
</script>

<section class="panel target-panel" aria-labelledby="target-heading">
  <header class="panel-heading">
    <div>
      <p class="section-kicker">Destination</p>
      <h2 id="target-heading">Printers & nozzles</h2>
    </div>
    <span class="panel-count">{visiblePrinters.length}</span>
  </header>

  <div class="printer-list">
    {#each visiblePrinters as printer (printer.id)}
      <details class="printer-card" open={printer.kind === 'official'}>
        <summary>
          <span class="printer-mark" class:custom={printer.kind === 'custom'}>
            {#if printer.verified}<BadgeCheck size={16} />{:else}<AlertTriangle size={16} />{/if}
          </span>
          <span class="printer-title">
            <strong>{printer.name}</strong>
            <small>{printer.code} · {printer.extruder_variants.length} drive modes</small>
          </span>
          <ChevronDown class="summary-chevron" size={16} />
        </summary>
        {#if !printer.verified}
          <p class="inline-warning"><AlertTriangle size={14} /> Unverified custom printer</p>
        {/if}
        <div class="nozzle-actions">
          <button
            class="text-button"
            type="button"
            aria-label={`Select all nozzles for ${printer.name}`}
            onclick={() => onSelectAll(printer.id, true)}>Select all</button
          >
          <button class="text-button muted" type="button" onclick={() => onSelectAll(printer.id, false)}
            >Clear</button
          >
        </div>
        <div class="nozzle-grid">
          {#each printer.nozzles as nozzle (nozzle.id)}
            <label class="nozzle-option" class:unsupported={!nozzle.supported}>
              <input
                type="checkbox"
                aria-label={`${nozzle.diameter} mm`}
                checked={selectedNozzles[printer.id]?.has(nozzle.diameter) ?? false}
                onchange={() => onNozzleToggle(printer.id, nozzle.diameter)}
              />
              <span class="nozzle-glyph"><CircleDashed size={19} strokeWidth={1.7} /></span>
              <span><strong>{nozzle.diameter}</strong><small>mm</small></span>
            </label>
          {/each}
        </div>
      </details>
    {:else}
      <div class="panel-empty compact">
        <CircleDashed size={26} />
        <p>No target printers discovered.</p>
      </div>
    {/each}
  </div>
</section>
