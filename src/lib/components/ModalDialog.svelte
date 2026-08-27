<script lang="ts">
  import { tick, type Snippet } from "svelte";

  let {
    open,
    title,
    confirmLabel = null,
    confirmTone = "primary",
    cancelLabel = "Cancel",
    busy = false,
    onClose,
    onConfirm,
    children,
  }: {
    open: boolean;
    title: string;
    confirmLabel?: string | null;
    confirmTone?: "primary" | "danger";
    cancelLabel?: string;
    busy?: boolean;
    onClose: () => void;
    onConfirm?: () => void;
    children?: Snippet;
  } = $props();

  const componentId = $props.id();
  const titleId = `${componentId}-title`;
  let dialog = $state<HTMLDialogElement>();
  let cancelButton = $state<HTMLButtonElement>();
  let returnFocus: HTMLElement | null = null;

  $effect(() => {
    if (!open) return;
    returnFocus = document.activeElement as HTMLElement | null;
    void tick().then(() => {
      if (!dialog) return;
      if (!dialog.open) dialog.showModal();
      cancelButton?.focus();
    });
  });

  async function restoreFocus() {
    await tick();
    returnFocus?.focus();
    returnFocus = null;
  }

  function close() {
    if (dialog?.open) dialog.close();
    onClose();
    void restoreFocus();
  }

  function confirm() {
    if (dialog?.open) dialog.close();
    onConfirm?.();
    void restoreFocus();
  }

  function handleCancel(event: Event) {
    event.preventDefault();
    close();
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget) close();
  }
</script>

{#if open}
  <dialog
    bind:this={dialog}
    class="modal-dialog"
    aria-labelledby={titleId}
    oncancel={handleCancel}
    onclick={handleBackdrop}
  >
    <div class="modal-dialog-surface">
      <header>
        <h2 id={titleId}>{title}</h2>
      </header>
      <div class="modal-dialog-content">
        {@render children?.()}
      </div>
      <footer>
        <button
          bind:this={cancelButton}
          type="button"
          class="secondary-button"
          disabled={busy}
          onclick={close}
        >
          {cancelLabel}
        </button>
        {#if confirmLabel}
          <button
            type="button"
            class={confirmTone === "danger"
              ? "danger-button"
              : "primary-button"}
            disabled={busy}
            onclick={confirm}
          >
            {busy ? "Working…" : confirmLabel}
          </button>
        {/if}
      </footer>
    </div>
  </dialog>
{/if}
