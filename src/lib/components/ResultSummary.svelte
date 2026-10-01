<script lang="ts">
  import { ChevronDown, Coffee, FileCheck2 } from "@lucide/svelte";
  import type { LocalRunResult, SyncResult } from "../types";

  let {
    result,
    synchronization = null,
    syncError = null,
    syncing = false,
    onOpenSupport = async () => {},
  }: {
    result: LocalRunResult;
    synchronization?: SyncResult | null;
    syncError?: string | null;
    syncing?: boolean;
    onOpenSupport?: () => Promise<void>;
  } = $props();

  let supportError = $state<string | null>(null);

  async function openSupport() {
    supportError = null;
    try {
      await onOpenSupport();
    } catch {
      supportError = "Could not open the support page. Please try again.";
    }
  }

  const counts = $derived(
    [
      `${result.created_files} created`,
      `${result.updated_files} updated`,
      `${result.deleted_files} deleted`,
      `${result.skipped_operations} skipped`,
      `${result.backup_file_count} backup ${result.backup_file_count === 1 ? "file" : "files"}`,
    ].join(" · "),
  );
  const resolved = $derived(
    !syncing && !syncError && !synchronization?.timed_out,
  );
</script>

<section class="result-summary" aria-label="Run details">
  <details class="run-details">
    <summary>
      <strong>Run details</strong>
      <span class="run-counts">{counts}</span>
      <span class="run-details-toggle"
        >Backup hash and receipt <ChevronDown size={13} /></span
      >
    </summary>
    <dl class="result-facts">
      <div>
        <dt>Plan</dt>
        <dd><code>{result.plan_id}</code></dd>
      </div>
      <div>
        <dt>Backup SHA-256</dt>
        <dd class="backup-checksum"><code>{result.backup_sha256}</code></dd>
      </div>
      <div>
        <dt>Receipt</dt>
        <dd class="receipt-path">
          <FileCheck2 size={14} /> <code>{result.receipt_path}</code>
        </dd>
      </div>
    </dl>
  </details>

  {#if resolved}
    <aside class="support-card" aria-labelledby="support-heading">
      <span class="support-mark"><Coffee size={20} /></span>
      <div>
        <h3 id="support-heading">Support Spool Ledger</h3>
        <p>
          Spool Ledger is free and open source. If it saved you time, donations
          help keep it maintained.
        </p>
        {#if supportError}<p class="support-error" role="alert">
            {supportError}
          </p>{/if}
      </div>
      <button class="support-button" type="button" onclick={openSupport}>
        <Coffee size={15} /> Buy me a coffee
      </button>
    </aside>
  {/if}
</section>
