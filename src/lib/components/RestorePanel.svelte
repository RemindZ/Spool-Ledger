<script lang="ts">
  import { AlertTriangle, ArchiveRestore, CheckCircle2 } from "@lucide/svelte";
  import ModalDialog from "./ModalDialog.svelte";
  import type {
    LocalRunResult,
    RestorePreview,
    RollbackOutcome,
  } from "../types";

  let {
    result,
    preview = null,
    rollback = null,
    restoring = false,
    onPreview,
    onRestore,
  }: {
    result: LocalRunResult;
    preview?: RestorePreview | null;
    rollback?: RollbackOutcome | null;
    restoring?: boolean;
    onPreview: () => void;
    onRestore: () => void;
  } = $props();

  let confirmationOpen = $state(false);
  const safeCount = $derived(
    preview?.paths.filter((path) => path.safe_to_restore).length ?? 0,
  );
  const externalCount = $derived(
    preview?.paths.filter((path) => !path.safe_to_restore).length ?? 0,
  );
  const ownedPathText = $derived(
    `${safeCount} owned ${safeCount === 1 ? "path" : "paths"}`,
  );
  const externalPathText = $derived(
    `${externalCount} externally changed ${externalCount === 1 ? "path" : "paths"}`,
  );
</script>

<section class="restore-panel panel" aria-labelledby="restore-heading">
  <header class="panel-heading">
    <div>
      <p class="section-kicker">Journal-owned recovery</p>
      <h2 id="restore-heading">Restore local profile paths</h2>
      <p>
        Run <code>{result.run_id}</code> can restore only unchanged paths recorded
        by its transaction journal.
      </p>
    </div>
    <ArchiveRestore size={20} aria-hidden="true" />
  </header>

  {#if preview}
    <div class="restore-summary">
      <strong>{ownedPathText} can be restored</strong>
      <span>{externalPathText} will be preserved</span>
    </div>
    <ul class="restore-paths">
      {#each preview.paths as path (path.path)}
        <li class:external={!path.safe_to_restore}>
          <span class="restore-path-state" aria-hidden="true">
            {#if path.safe_to_restore}
              <CheckCircle2 size={16} />
            {:else}
              <AlertTriangle size={16} />
            {/if}
          </span>
          <div>
            <code>{path.path}</code>
            <small>{path.action}</small>
            <details>
              <summary>Path fingerprints</summary>
              <dl>
                <div>
                  <dt>Current</dt>
                  <dd><code>{path.current_sha256 ?? "not present"}</code></dd>
                </div>
                <div>
                  <dt>Committed</dt>
                  <dd>
                    <code
                      >{path.expected_committed_sha256 ?? "not present"}</code
                    >
                  </dd>
                </div>
              </dl>
            </details>
          </div>
          <strong>
            {path.safe_to_restore
              ? "Ready to restore"
              : "Externally changed - preserve"}
          </strong>
        </li>
      {/each}
    </ul>
    <div class="restore-actions">
      <button class="secondary-button" type="button" onclick={onPreview}>
        Refresh restore preview
      </button>
      <button
        class="danger-button"
        type="button"
        disabled={restoring || safeCount === 0}
        onclick={() => (confirmationOpen = true)}
      >
        {restoring ? "Restoring owned files…" : `Restore ${ownedPathText}`}
      </button>
    </div>
  {:else}
    <div class="view-empty compact">
      <h3>Preview current path safety</h3>
      <p>No files are changed when the restore preview is created.</p>
      <button class="secondary-button" type="button" onclick={onPreview}>
        Preview journal-owned restore
      </button>
    </div>
  {/if}

  {#if rollback}
    <p class="rollback-result" role="status">
      Restored {rollback.rolled_back}; preserved {rollback.external_conflicts}
      externally changed; {rollback.failed} failed.
    </p>
  {/if}

  <ModalDialog
    open={confirmationOpen}
    title={`Restore files from run ${result.run_id}?`}
    confirmLabel="Restore owned files"
    confirmTone="danger"
    busy={restoring}
    onClose={() => (confirmationOpen = false)}
    onConfirm={() => {
      confirmationOpen = false;
      onRestore();
    }}
  >
    <p>
      Only {safeCount} unchanged journal-owned {safeCount === 1
        ? "path"
        : "paths"}
      will be restored.
    </p>
    {#if externalCount > 0}
      <p>
        {externalPathText} will be preserved because their current bytes no longer
        match this run.
      </p>
    {/if}
  </ModalDialog>
</section>
