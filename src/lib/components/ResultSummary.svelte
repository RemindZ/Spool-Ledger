<script lang="ts">
  import { AlertTriangle, ArchiveRestore, CheckCircle2, FileCheck2 } from '@lucide/svelte';
  import type { LocalRunResult, RestorePreview, RollbackOutcome } from '../types';

  let {
    result,
    restorePreview = null,
    rollback = null,
    restoring = false,
    onPreviewRestore = () => {},
    onRestore = () => {},
  }: {
    result: LocalRunResult;
    restorePreview?: RestorePreview | null;
    rollback?: RollbackOutcome | null;
    restoring?: boolean;
    onPreviewRestore?: () => void;
    onRestore?: () => void;
  } = $props();

  const safeCount = $derived(restorePreview?.paths.filter((path) => path.safe_to_restore).length ?? 0);
  const externalCount = $derived(restorePreview?.paths.filter((path) => !path.safe_to_restore).length ?? 0);
</script>

<section class="result-summary panel" aria-labelledby="result-heading">
  <header class="panel-heading result-heading">
    <span class="result-mark"><CheckCircle2 size={24} /></span>
    <div>
      <p class="section-kicker">Local transaction complete</p>
      <h2 id="result-heading">{result.committed_files} local files committed</h2>
      <p>Run <code>{result.run_id}</code> has a checksum-verified backup and per-file journal.</p>
    </div>
  </header>

  <dl class="result-facts">
    <div><dt>Plan</dt><dd>{result.plan_id}</dd></div>
    <div><dt>Receipt</dt><dd><FileCheck2 size={14} /> Saved</dd></div>
  </dl>

  {#if restorePreview}
    <div class="restore-preview" aria-live="polite">
      <header>
        <div>
          <p class="section-kicker">Restore preview</p>
          <h3>{safeCount} owned paths can be restored</h3>
        </div>
        {#if externalCount}<span class="warning-count"><AlertTriangle size={14} /> {externalCount} preserved</span>{/if}
      </header>
      <ul>
        {#each restorePreview.paths as path (path.path)}
          <li class:external={!path.safe_to_restore}>
            {#if path.safe_to_restore}<ArchiveRestore size={15} />{:else}<AlertTriangle size={15} />{/if}
            <span><code>{path.path}</code><small>{path.action}</small></span>
            <strong>{path.safe_to_restore ? 'Ready to restore' : 'Externally changed - preserve'}</strong>
          </li>
        {/each}
      </ul>
      <button class="danger-button" type="button" disabled={restoring || safeCount === 0} onclick={onRestore}>
        {restoring ? 'Restoring owned files…' : `Restore ${safeCount} owned paths`}
      </button>
    </div>
  {:else}
    <button class="secondary-button" type="button" onclick={onPreviewRestore}>
      <ArchiveRestore size={15} /> Preview restore
    </button>
  {/if}

  {#if rollback}
    <p class="rollback-result" role="status">
      Restored {rollback.rolled_back}; preserved {rollback.external_conflicts} externally changed; {rollback.failed} failed.
    </p>
  {/if}
</section>
