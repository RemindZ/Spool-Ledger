<script lang="ts">
  import { tick } from "svelte";
  import type { PrinterTarget } from "../types";
  import PrinterPicker from "./PrinterPicker.svelte";

  let {
    open,
    catalogId,
    printers,
    enabledPrinterIds,
    selectedNozzles,
    onSave,
    onCancel,
  }: {
    open: boolean;
    catalogId: string;
    printers: PrinterTarget[];
    enabledPrinterIds: Set<string>;
    selectedNozzles: Record<string, Set<string>>;
    onSave: (
      enabledPrinterIds: Set<string>,
      selectedNozzles: Record<string, Set<string>>,
    ) => void;
    onCancel: () => void;
  } = $props();

  const componentId = $props.id();
  const titleId = `${componentId}-title`;
  let dialog = $state<HTMLDialogElement>();
  let cancelButton = $state<HTMLButtonElement>();
  let draftEnabled = $state(new Set<string>());
  let draftNozzles = $state<Record<string, Set<string>>>({});
  let returnFocus: HTMLElement | null = null;

  const valid = $derived(
    [...draftEnabled].every(
      (printerId) => (draftNozzles[printerId]?.size ?? 0) > 0,
    ),
  );

  $effect(() => {
    if (!open) return;
    draftEnabled = new Set(enabledPrinterIds);
    draftNozzles = Object.fromEntries(
      Object.entries(selectedNozzles).map(([printerId, nozzles]) => [
        printerId,
        new Set(nozzles),
      ]),
    );
    returnFocus = document.activeElement as HTMLElement | null;
    void tick().then(() => {
      if (!dialog) return;
      if (!dialog.open) dialog.showModal();
      cancelButton?.focus();
    });
  });

  function updateDraft(
    enabled: Set<string>,
    nozzles: Record<string, Set<string>>,
  ) {
    draftEnabled = enabled;
    draftNozzles = nozzles;
  }

  async function restoreFocus() {
    await tick();
    returnFocus?.focus();
    returnFocus = null;
  }

  function cancel() {
    if (dialog?.open) dialog.close();
    onCancel();
    void restoreFocus();
  }

  function save() {
    if (!valid) return;
    if (dialog?.open) dialog.close();
    onSave(new Set(draftEnabled), draftNozzles);
    void restoreFocus();
  }

  function handleCancel(event: Event) {
    event.preventDefault();
    cancel();
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget) cancel();
  }
</script>

{#if open}
  <dialog
    bind:this={dialog}
    class="modal-dialog printer-manager-dialog"
    aria-labelledby={titleId}
    oncancel={handleCancel}
    onclick={handleBackdrop}
  >
    <div class="modal-dialog-surface">
      <header>
        <p class="section-kicker">Destination defaults</p>
        <h2 id={titleId}>Manage enabled printers</h2>
        <p>
          Enabled official printers appear in the destination rail. New printer
          models stay disabled until you add them here.
        </p>
      </header>
      <div class="modal-dialog-content manager-content">
        <PrinterPicker
          {catalogId}
          {printers}
          enabledPrinterIds={draftEnabled}
          selectedNozzles={draftNozzles}
          onChange={updateDraft}
        />
        {#if !valid}
          <p class="manager-validation" role="status">
            Choose at least one supported nozzle for every enabled printer.
          </p>
        {/if}
      </div>
      <footer>
        <span
          >{draftEnabled.size} of {printers.filter(
            (printer) => printer.kind === "official",
          ).length} enabled</span
        >
        <button
          bind:this={cancelButton}
          type="button"
          class="secondary-button"
          onclick={cancel}>Cancel</button
        >
        <button
          type="button"
          class="primary-button"
          disabled={!valid}
          onclick={save}>Save changes</button
        >
      </footer>
    </div>
  </dialog>
{/if}

<style>
  .printer-manager-dialog {
    width: min(62rem, calc(100vw - 2rem));
  }

  .printer-manager-dialog .modal-dialog-surface {
    width: 100%;
    max-height: min(48rem, calc(100vh - 2rem));
  }

  header {
    display: grid;
    gap: 0.25rem;
  }

  header h2,
  header p {
    margin: 0;
  }

  header > p:last-child,
  footer > span {
    color: var(--text-muted);
  }

  .manager-content {
    overflow-y: auto;
    overscroll-behavior: contain;
  }

  .manager-validation {
    margin: 0.75rem 0 0;
    padding: 0.6rem 0.7rem;
    border-radius: var(--radius-sm);
    color: var(--warning-strong);
    background: var(--warning-soft);
  }

  footer > span {
    margin-right: auto;
  }

  @media (max-width: 34rem) {
    .printer-manager-dialog {
      width: calc(100vw - 1rem);
    }

    .printer-manager-dialog .modal-dialog-surface {
      max-height: calc(100vh - 1rem);
    }

    footer > span {
      width: 100%;
    }
  }
</style>
