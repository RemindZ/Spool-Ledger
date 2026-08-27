<script lang="ts">
  import { BadgeCheck, CircleDashed } from "@lucide/svelte";
  import type { PrinterTarget } from "../types";
  import PrinterArtwork from "./PrinterArtwork.svelte";

  let {
    catalogId,
    printers,
    enabledPrinterIds,
    selectedNozzles,
    onChange,
  }: {
    catalogId: string;
    printers: PrinterTarget[];
    enabledPrinterIds: Set<string>;
    selectedNozzles: Record<string, Set<string>>;
    onChange: (
      enabledPrinterIds: Set<string>,
      selectedNozzles: Record<string, Set<string>>,
    ) => void;
  } = $props();

  const officialPrinters = $derived(
    printers.filter((printer) => printer.kind === "official"),
  );

  function cloneNozzles(): Record<string, Set<string>> {
    return Object.fromEntries(
      Object.entries(selectedNozzles).map(([printerId, nozzles]) => [
        printerId,
        new Set(nozzles),
      ]),
    );
  }

  function togglePrinter(printer: PrinterTarget) {
    const willEnable = !enabledPrinterIds.has(printer.id);
    const enabled = willEnable
      ? new Set([...enabledPrinterIds, printer.id])
      : new Set([...enabledPrinterIds].filter((id) => id !== printer.id));
    const nozzles = cloneNozzles();
    if (!willEnable) {
      delete nozzles[printer.id];
    } else {
      const supported = printer.nozzles.filter((nozzle) => nozzle.supported);
      const defaultNozzle =
        supported.find((nozzle) => nozzle.diameter === "0.4") ?? supported[0];
      nozzles[printer.id] = defaultNozzle
        ? new Set([defaultNozzle.diameter])
        : new Set();
    }
    onChange(enabled, nozzles);
  }

  function toggleNozzle(printerId: string, diameter: string) {
    const nozzles = cloneNozzles();
    const current = nozzles[printerId] ?? new Set<string>();
    const selected = current.has(diameter)
      ? new Set([...current].filter((candidate) => candidate !== diameter))
      : new Set([...current, diameter]);
    nozzles[printerId] = selected;
    onChange(new Set(enabledPrinterIds), nozzles);
  }
</script>

<div class="printer-picker">
  {#each officialPrinters as printer (printer.id)}
    <article
      class:enabled={enabledPrinterIds.has(printer.id)}
      class="printer-pick-card"
    >
      <label class="printer-choice">
        <input
          type="checkbox"
          checked={enabledPrinterIds.has(printer.id)}
          aria-label={`Enable ${printer.name}`}
          onchange={() => togglePrinter(printer)}
        />
        <PrinterArtwork {catalogId} {printer} />
        <span class="printer-choice-copy">
          <strong>{printer.name}</strong>
          <small><BadgeCheck size={13} /> Official {printer.code}</small>
        </span>
      </label>

      <fieldset disabled={!enabledPrinterIds.has(printer.id)}>
        <legend>Default nozzles</legend>
        <div class="picker-nozzles">
          {#each printer.nozzles.filter((nozzle) => nozzle.supported) as nozzle (nozzle.id)}
            <label>
              <input
                type="checkbox"
                checked={selectedNozzles[printer.id]?.has(nozzle.diameter) ??
                  false}
                aria-label={`${printer.name} ${nozzle.diameter} mm nozzle`}
                onchange={() => toggleNozzle(printer.id, nozzle.diameter)}
              />
              <CircleDashed size={16} strokeWidth={1.7} />
              <span>{nozzle.diameter} mm</span>
            </label>
          {/each}
        </div>
      </fieldset>
    </article>
  {:else}
    <div class="panel-empty compact">
      <CircleDashed size={26} />
      <p>No official Bambu printers were discovered.</p>
    </div>
  {/each}
</div>

<style>
  .printer-picker {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 16rem), 1fr));
    gap: 0.75rem;
  }

  .printer-pick-card {
    display: grid;
    gap: 0.75rem;
    min-width: 0;
    padding: 0.85rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--surface-raised);
    transition:
      border-color 140ms ease,
      box-shadow 140ms ease;
  }

  .printer-pick-card.enabled {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent-soft);
  }

  .printer-choice {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    min-width: 0;
    cursor: pointer;
  }

  .printer-choice > input {
    flex: 0 0 auto;
  }

  .printer-choice-copy {
    display: grid;
    gap: 0.2rem;
    min-width: 0;
  }

  .printer-choice-copy strong {
    overflow-wrap: anywhere;
  }

  .printer-choice-copy small {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--text-muted);
  }

  fieldset {
    display: grid;
    gap: 0.45rem;
    min-width: 0;
    margin: 0;
    padding: 0;
    border: 0;
  }

  legend {
    margin-bottom: 0.35rem;
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--text-muted);
  }

  .picker-nozzles {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .picker-nozzles label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.35rem 0.45rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .picker-nozzles label:has(input:checked) {
    border-color: var(--accent);
    color: var(--accent-strong);
    background: var(--accent-soft);
  }

  fieldset:disabled {
    opacity: 0.55;
  }

  @media (prefers-reduced-motion: reduce) {
    .printer-pick-card {
      transition: none;
    }
  }
</style>
