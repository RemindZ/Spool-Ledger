<script lang="ts">
  import {
    AlertOctagon,
    CheckCircle2,
    CircleDashed,
    MinusCircle,
  } from "@lucide/svelte";
  import type { MigrationPlan } from "../types";
  import type { OperationProgress } from "../state";

  let {
    plan,
    progress,
    running,
    cancellable = false,
    onCancel = () => {},
    onVerify = () => {},
  }: {
    plan: MigrationPlan | null;
    progress: Record<string, OperationProgress>;
    running: boolean;
    cancellable?: boolean;
    onCancel?: () => void;
    onVerify?: (operationIds: string[]) => void;
  } = $props();

  const evidenceLabels = {
    created_local: "Created locally",
    loaded_by_bambu: "Loaded by Bambu",
    cloud_id_assigned: "Cloud ID assigned",
    ams_verified: "AMS verified",
  } as const;

  const operations = $derived(plan?.operations ?? []);
  const trackedOperations = $derived(
    operations.filter(
      (operation) =>
        operation.action !== "block" && operation.action !== "skip",
    ),
  );
  const trackedCount = $derived(trackedOperations.length);
  const localCount = $derived(
    trackedOperations.filter((operation) => progress[operation.id]).length,
  );
  const loadedCount = $derived(
    trackedOperations.filter((operation) =>
      ["loaded_by_bambu", "cloud_id_assigned", "ams_verified"].includes(
        progress[operation.id]?.evidence ?? "",
      ),
    ).length,
  );
  const cloudCount = $derived(
    trackedOperations.filter((operation) =>
      ["cloud_id_assigned", "ams_verified"].includes(
        progress[operation.id]?.evidence ?? "",
      ),
    ).length,
  );
  const amsCount = $derived(
    trackedOperations.filter(
      (operation) => progress[operation.id]?.evidence === "ams_verified",
    ).length,
  );
  const tiles = $derived([
    {
      name: "Created locally",
      description: "Committed and parse-validated locally.",
      count: localCount,
      unit: "committed",
    },
    {
      name: "Loaded by Bambu",
      description: "Bambu Studio acknowledged the local artifact.",
      count: loadedCount,
      unit: "acknowledged",
    },
    {
      name: "Cloud ID assigned",
      description: "A unique setting ID appeared in its sidecar.",
      count: cloudCount,
      unit: "IDs assigned",
    },
    {
      name: "AMS confirmed",
      description: "Operator confirmed the intended AMS surface.",
      count: amsCount,
      unit: "operator verified",
    },
  ]);
  const verifiedIds = $derived(
    operations
      .filter(
        (operation) => progress[operation.id]?.evidence === "cloud_id_assigned",
      )
      .map((operation) => operation.id),
  );
</script>

<section class="run-progress panel" aria-labelledby="progress-heading">
  <header class="panel-heading">
    <h2 id="progress-heading">Migration evidence</h2>
    <p>
      Each level is recorded separately. Only you can confirm AMS visibility.
    </p>
  </header>

  <ol class="evidence-grid">
    {#each tiles as tile (tile.name)}
      <li class="evidence-tile" class:reached={tile.count > 0}>
        <span class="evidence-name"
          >{tile.name}<span class="sr-only"
            >{` ${tile.count > 0 ? "reached" : "not reached"}`}</span
          ></span
        >
        <p>{tile.description}</p>
        <small class="evidence-count"
          >{tile.count} / {trackedCount} {tile.unit}</small
        >
      </li>
    {/each}
  </ol>

  {#if operations.length}<div class="operation-progress-list">
      {#each operations as operation (operation.id)}
        {@const item = progress[operation.id]}
        {@const status = item
          ? "evidence"
          : operation.action === "skip" || operation.action === "block"
            ? operation.action
            : "waiting"}
        <div class="operation-progress-row" data-status={status}>
          {#if status === "evidence"}<CheckCircle2
              size={15}
            />{:else if status === "skip"}<MinusCircle
              size={15}
            />{:else if status === "block"}<AlertOctagon
              size={15}
            />{:else}<CircleDashed size={15} />{/if}
          <span
            ><strong>{operation.ams_name}</strong><small
              >{operation.printer_preset_name}</small
            ></span
          >
          <span class="evidence-label"
            >{item
              ? evidenceLabels[item.evidence]
              : status === "skip"
                ? "Skipped"
                : status === "block"
                  ? "Blocked"
                  : "Waiting"}</span
          >
          {#if item?.evidence === "cloud_id_assigned"}
            <span class="pending-label">AMS check pending</span>
          {/if}
        </div>
      {/each}
    </div>{/if}

  <footer class="panel-actions">
    {#if running && cancellable}
      <button class="secondary-button" type="button" onclick={onCancel}
        >Cancel monitoring</button
      >
    {:else if !running && verifiedIds.length}
      <button
        class="secondary-button"
        type="button"
        onclick={() => onVerify(verifiedIds)}>Record AMS verification</button
      >
    {/if}
  </footer>
</section>
