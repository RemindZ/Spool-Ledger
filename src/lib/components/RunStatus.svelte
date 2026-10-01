<script lang="ts">
  import {
    AlertTriangle,
    ArchiveRestore,
    CheckCircle2,
    LoaderCircle,
  } from "@lucide/svelte";
  import ExecutionPhases from "./ExecutionPhases.svelte";
  import type {
    EvidenceLevel,
    ExecutionPhase,
    LocalRunResult,
    SyncPhase,
    SyncResult,
  } from "../types";

  let {
    result,
    synchronization = null,
    syncError = null,
    syncing = false,
    localPhase = null,
    syncPhase = null,
    syncExpected = true,
    onRetrySync = () => {},
    onOpenRestore = null,
  }: {
    result: LocalRunResult | null;
    synchronization?: SyncResult | null;
    syncError?: string | null;
    syncing?: boolean;
    localPhase?: ExecutionPhase | null;
    syncPhase?: SyncPhase | null;
    syncExpected?: boolean;
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
    tone: "progress" | "warning" | "success";
    title: string;
    detail: string;
  };

  const verdict = $derived.by((): Verdict => {
    if (!result) {
      return {
        tone: "progress",
        title: "Committing local profiles",
        detail:
          "The local transaction is running. Evidence appears below as each step finishes.",
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
  class="run-status"
  data-tone={verdict.tone}
  aria-labelledby="run-status-heading"
>
  <div class="run-status-main">
    <span class="run-status-mark" aria-hidden="true">
      {#if verdict.tone === "progress"}<LoaderCircle
          size={22}
          class="spin"
        />{:else if verdict.tone === "warning"}<AlertTriangle
          size={22}
        />{:else}<CheckCircle2 size={22} />{/if}
    </span>
    <div class="run-status-copy" role="status">
      <p class="section-kicker">
        {#if result}Run <code>{result.run_id}</code>{:else}Local transaction{/if}
      </p>
      <h2 id="run-status-heading">{verdict.title}</h2>
      <p>{verdict.detail}</p>
    </div>
    {#if (result && onOpenRestore) || canRetry}
      <div class="run-status-actions">
        {#if result && onOpenRestore}
          <button
            class="secondary-button"
            type="button"
            onclick={onOpenRestore}
          >
            <ArchiveRestore size={15} /> Open journal-owned restore
          </button>
        {/if}
        {#if canRetry}
          <button class="primary-button" type="button" onclick={onRetrySync}>
            Retry synchronization
          </button>
        {/if}
      </div>
    {/if}
  </div>
  <ExecutionPhases {localPhase} {syncPhase} {syncExpected} />
</section>
