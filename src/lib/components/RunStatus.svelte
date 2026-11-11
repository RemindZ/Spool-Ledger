<script lang="ts">
  import { ArchiveRestore, RotateCw } from "@lucide/svelte";
  import type { EvidenceLevel, LocalRunResult, SyncResult } from "../types";

  let {
    result,
    executing = false,
    localError = null,
    synchronization = null,
    syncError = null,
    syncing = false,
    onRetrySync = () => {},
    onOpenRestore = null,
  }: {
    result: LocalRunResult | null;
    executing?: boolean;
    localError?: string | null;
    synchronization?: SyncResult | null;
    syncError?: string | null;
    syncing?: boolean;
    onRetrySync?: () => void;
    onOpenRestore?: (() => void) | null;
  } = $props();

  const evidenceText: Record<EvidenceLevel, string> = {
    created_local: "created locally",
    loaded_by_bambu: "loaded by Bambu",
    cloud_id_assigned: "cloud ID assigned",
    ams_verified: "AMS verified",
  };

  type Verdict = {
    tone: "idle" | "progress" | "warning" | "success";
    title: string;
    detail: string;
  };

  const verdict = $derived.by((): Verdict => {
    if (!result) {
      if (executing) {
        return {
          tone: "progress",
          title: "Committing local profiles",
          detail:
            "The local transaction is running. Each step reports when it finishes.",
        };
      }
      if (localError) {
        return {
          tone: "warning",
          title: "Local transaction stopped",
          detail: `${localError} Review the plan, then commit again.`,
        };
      }
      return {
        tone: "idle",
        title: "Waiting for a committed run",
        detail: "Commit a frozen plan to begin local execution.",
      };
    }
    const count = result.committed_files;
    const files = `${count} local ${count === 1 ? "file" : "files"}`;
    const committed = `${files} committed`;
    const remain = `${files} ${count === 1 ? "remains" : "remain"} committed`;
    if (syncing) {
      return {
        tone: "progress",
        title: "Synchronizing with Bambu Studio",
        detail: `${committed}. Waiting for local acknowledgement and unique PFUS IDs.`,
      };
    }
    if (syncError) {
      return {
        tone: "warning",
        title: "Synchronization stopped",
        detail: `${remain}. ${syncError}`,
      };
    }
    if (synchronization?.timed_out) {
      return {
        tone: "warning",
        title: "Synchronization timed out",
        detail: `${remain}. The highest observed evidence is ${evidenceText[synchronization.highest_evidence]}.`,
      };
    }
    if (synchronization) {
      return {
        tone: "success",
        title: "Synchronization observed",
        detail: `${committed}. The highest observed evidence is ${evidenceText[synchronization.highest_evidence]}. AMS verification remains operator-only.`,
      };
    }
    return {
      tone: "success",
      title: committed,
      detail: "The run has a path-owned backup and per-file journal.",
    };
  });

  const canRetry = $derived(
    Boolean(result) &&
      !syncing &&
      (Boolean(syncError) || Boolean(synchronization?.timed_out)),
  );
</script>

<section
  class="panel run-status"
  data-tone={verdict.tone}
  aria-labelledby="run-status-heading"
>
  <span class="tag run-tag"
    >{result
      ? result.run_id
      : executing
        ? "Local transaction"
        : "No committed run"}</span
  >
  {#key verdict.title}
    <h2 id="run-status-heading" class="run-status-headline">
      {verdict.title}
    </h2>
  {/key}
  <p role="status">{verdict.detail}</p>
  {#if (result && onOpenRestore) || canRetry}
    <div class="run-status-actions">
      {#if canRetry}
        <button class="btn btn-secondary" type="button" onclick={onRetrySync}>
          <RotateCw size={15} /> Retry synchronization
        </button>
      {/if}
      {#if result && onOpenRestore}
        <button class="btn btn-secondary" type="button" onclick={onOpenRestore}>
          <ArchiveRestore size={15} /> Open journal-owned restore
        </button>
      {/if}
    </div>
  {/if}
</section>
