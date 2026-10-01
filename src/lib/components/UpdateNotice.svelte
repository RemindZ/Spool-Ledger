<script lang="ts">
  import { ArrowUpCircle } from "@lucide/svelte";
  import type { AvailableUpdate } from "../types";

  let {
    update,
    error = null,
    onUpdate,
    onDismiss,
    onSkip,
  }: {
    update: AvailableUpdate;
    error?: string | null;
    onUpdate: () => void;
    onDismiss: () => void;
    onSkip: () => void;
  } = $props();
</script>

<aside
  class="update-notice"
  aria-labelledby="update-heading"
  aria-live="polite"
>
  <span class="update-mark" aria-hidden="true"><ArrowUpCircle size={20} /></span
  >
  <div class="update-copy">
    <h2 id="update-heading">Update available</h2>
    <p>Spool Ledger {update.latest} is available. You have {update.current}.</p>
    <small>Update opens the release page in your browser.</small>
    {#if error}<p class="update-error" role="alert">{error}</p>{/if}
  </div>
  <div class="update-actions">
    <button class="text-button muted" type="button" onclick={onSkip}
      >Skip this version</button
    >
    <button class="secondary-button" type="button" onclick={onDismiss}
      >Not now</button
    >
    <button class="primary-button" type="button" onclick={onUpdate}
      >Update</button
    >
  </div>
</aside>
