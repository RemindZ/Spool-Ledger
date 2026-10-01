<script lang="ts">
  import {
    AlertOctagon,
    CheckCircle2,
    CircleDashed,
    Cloud,
    HardDrive,
    MinusCircle,
    ShieldCheck,
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
    plan: MigrationPlan;
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

  const trackedOperations = $derived(
    plan.operations.filter(
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
  const verifiedIds = $derived(
    plan.operations
      .filter(
        (operation) => progress[operation.id]?.evidence === "cloud_id_assigned",
      )
      .map((operation) => operation.id),
  );
</script>

<section class="run-progress panel" aria-labelledby="progress-heading">
  <header class="panel-heading">
    <h2 id="progress-heading">Evidence</h2>
    <p>Each level is recorded separately. Only you can confirm AMS.</p>
  </header>

  <ol class="evidence-steps">
    <li class:active={localCount > 0}>
      <HardDrive size={17} /><span
        ><strong>Local</strong><small>Files committed and parsed</small><small
          class="evidence-count">{localCount} / {trackedCount} committed</small
        ></span
      >
    </li>
    <li class:active={loadedCount > 0}>
      <CheckCircle2 size={17} /><span
        ><strong>Bambu</strong><small>Profile acknowledged locally</small><small
          class="evidence-count"
          >{loadedCount} / {trackedCount} acknowledged</small
        ></span
      >
    </li>
    <li class:active={cloudCount > 0}>
      <Cloud size={17} /><span
        ><strong>Cloud</strong><small>Unique PFUS ID observed</small><small
          class="evidence-count"
          >{cloudCount} / {trackedCount} IDs assigned</small
        ></span
      >
    </li>
    <li class:active={amsCount > 0}>
      <ShieldCheck size={17} /><span
        ><strong>AMS</strong><small>Operator-confirmed only</small><small
          class="evidence-count"
          >{amsCount} / {trackedCount} operator verified</small
        ></span
      >
    </li>
  </ol>

  <div class="operation-progress-list">
    {#each plan.operations as operation (operation.id)}
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
  </div>

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
