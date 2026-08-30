<script lang="ts">
  import {
    AlertTriangle,
    ArchiveRestore,
    CheckCircle2,
    Coffee,
    FileCheck2,
    RefreshCw,
  } from "@lucide/svelte";
  import type { LocalRunResult, SyncResult } from "../types";

  let {
    result,
    synchronization = null,
    syncError = null,
    syncing = false,
    onRetrySync = () => {},
    onOpenRestore = null,
    onOpenSupport = async () => {},
  }: {
    result: LocalRunResult;
    synchronization?: SyncResult | null;
    syncError?: string | null;
    syncing?: boolean;
    onRetrySync?: () => void;
    onOpenRestore?: (() => void) | null;
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

  const committedFileText = $derived(
    `${result.committed_files} local ${result.committed_files === 1 ? "file" : "files"}`,
  );
  const backupFileText = $derived(
    `${result.backup_file_count} backup ${result.backup_file_count === 1 ? "file" : "files"}`,
  );
</script>

<section class="result-summary panel" aria-labelledby="result-heading">
  <header class="panel-heading result-heading">
    <span class="result-mark"><CheckCircle2 size={24} /></span>
    <div>
      <p class="section-kicker">Local transaction complete</p>
      <h2 id="result-heading">{committedFileText} committed</h2>
      <p>
        Run <code>{result.run_id}</code> has a path-owned backup and per-file journal.
      </p>
    </div>
  </header>

  <dl class="result-facts">
    <div>
      <dt>Plan</dt>
      <dd><code>{result.plan_id}</code></dd>
    </div>
    <div>
      <dt>Created</dt>
      <dd>{result.created_files} created</dd>
    </div>
    <div>
      <dt>Updated</dt>
      <dd>{result.updated_files} updated</dd>
    </div>
    <div>
      <dt>Deleted</dt>
      <dd>{result.deleted_files} deleted</dd>
    </div>
    <div>
      <dt>Skipped</dt>
      <dd>
        {result.skipped_operations}
        {result.skipped_operations === 1 ? " operation" : " operations"}
        skipped
      </dd>
    </div>
    <div>
      <dt>Backup files</dt>
      <dd>{backupFileText}</dd>
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

  {#if syncing}
    <div class="sync-summary" role="status">
      <RefreshCw size={16} class="spin" />
      <span>
        <strong>Synchronizing with Bambu Studio</strong>
        <small>Waiting for local acknowledgement and unique PFUS IDs.</small>
      </span>
    </div>
  {:else if syncError}
    <div class="sync-summary warning" role="alert">
      <AlertTriangle size={16} />
      <span>
        <strong>Synchronization stopped</strong>
        <small>{syncError}</small>
      </span>
      <button class="secondary-button" type="button" onclick={onRetrySync}>
        Retry synchronization
      </button>
    </div>
  {:else if synchronization?.timed_out}
    <div class="sync-summary warning" role="status">
      <AlertTriangle size={16} />
      <span>
        <strong>Synchronization timed out</strong>
        <small>
          Local profiles remain committed. The highest observed evidence is
          {synchronization.highest_evidence.replace(/_/g, " ")}.
        </small>
      </span>
      <button class="secondary-button" type="button" onclick={onRetrySync}>
        Retry synchronization
      </button>
    </div>
  {:else if synchronization}
    <div class="sync-summary" role="status">
      <CheckCircle2 size={16} />
      <span>
        <strong>Synchronization observed</strong>
        <small>
          Highest evidence: {synchronization.highest_evidence.replace(
            /_/g,
            " ",
          )}. AMS verification remains operator-only.
        </small>
      </span>
    </div>
  {/if}

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

  {#if onOpenRestore}
    <button class="secondary-button" type="button" onclick={onOpenRestore}>
      <ArchiveRestore size={15} /> Open journal-owned restore
    </button>
  {/if}
</section>
