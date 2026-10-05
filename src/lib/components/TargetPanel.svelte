<script lang="ts">
  import { CircleDashed, Settings } from "@lucide/svelte";
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

  function allSelected(printer: PrinterTarget): boolean {
    const supported = printer.nozzles.filter((nozzle) => nozzle.supported);
    return (
      supported.length > 0 &&
      supported.every((nozzle) =>
        selectedNozzles[printer.id]?.has(nozzle.diameter),
      )
    );
  }
</script>

<div class="printer-tools">
  <p
    aria-label={`${visiblePrinters.length} of ${officialPrinters.length} official printers enabled`}
  >
    <strong>{visiblePrinters.length}</strong> of {officialPrinters.length} official
    printers enabled
  </p>
  <button type="button" class="btn btn-secondary" onclick={onManage}>
    <Settings size={15} /> Manage enabled printers
  </button>
</div>

<div class="printer-grid">
  {#each visiblePrinters as printer (printer.id)}
    {@const all = allSelected(printer)}
    <div class="printer">
      <div class="printer-head">
        <PrinterArtwork {catalogId} {printer} />
        <div class="printer-copy">
          <div class="printer-name">{printer.name}</div>
          <div class="source-meta">
            {printer.code} · {printer.extruder_variants.length}
            {printer.extruder_variants.length === 1
              ? "drive mode"
              : "drive modes"}
          </div>
        </div>
        <button
          class="btn btn-quiet select-nozzles"
          type="button"
          aria-label={`${all ? "Clear" : "Select"} all nozzles for ${printer.name}`}
          onclick={() => onSelectAll(printer.id, !all)}
          >{all ? "Clear all" : "Select all"}</button
        >
      </div>
      <div class="nozzles">
        {#each printer.nozzles as nozzle (nozzle.id)}
          <label
            class="nozzle"
            title={nozzle.supported
              ? undefined
              : "Unavailable for this official printer"}
          >
            <input
              type="checkbox"
              class="nozzle-check"
              aria-label={`${nozzle.diameter} mm`}
              checked={selectedNozzles[printer.id]?.has(nozzle.diameter) ??
                false}
              disabled={!nozzle.supported}
              onchange={() => onNozzleToggle(printer.id, nozzle.diameter)}
            />
            <span
              >{nozzle.diameter}{#if !nozzle.supported}<span class="sr-only"
                  >&#32;Unavailable for this official printer</span
                >{/if}</span
            >
          </label>
        {/each}
      </div>
    </div>
  {:else}
    <div class="panel-empty compact">
      <CircleDashed size={26} />
      <p>
        No printers enabled. Use Manage enabled printers to add an official
        Bambu printer.
      </p>
    </div>
  {/each}
</div>
