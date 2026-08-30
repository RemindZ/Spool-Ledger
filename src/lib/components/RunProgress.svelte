<script lang="ts">
  import {
    CheckCircle2,
    CircleDashed,
    Cloud,
    HardDrive,
    ShieldCheck,
  } from "@lucide/svelte";
  import type { ExecutionPhase, MigrationPlan } from "../types";
  import type { OperationProgress } from "../state";

  let {
    plan,
    progress,
    running,
    localPhase = null,
    cancellable = false,
    onCancel = () => {},
    onVerify = () => {},
  }: {
    plan: MigrationPlan;
    progress: Record<string, OperationProgress>;
    running: boolean;
    localPhase?: ExecutionPhase | null;
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

  const localPhaseLabels: Record<ExecutionPhase, string> = {
    validating: "Validating frozen plan",
    closing_bambu: "Closing Bambu Studio",
    staging: "Staging files",
    validating_output: "Output validation complete",
    backing_up: "Creating ZIP backup",
    committing: "Committing journaled files",
    writing_receipt: "Writing journal and receipt",
    finished: "Local commit finished",
  };

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
    <div>
      <p class="section-kicker">Evidence ledger</p>
      <h2 id="progress-heading">Migration progress</h2>
    </div>
    {#if running}<span class="live-indicator" aria-live="polite"
        ><i></i> {localPhase ? localPhaseLabels[localPhase] : "Running"}</span
      >{/if}
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
      {@const terminal =
        operation.action === "skip" || operation.action === "block"}
      <div class="operation-progress-row">
        {#if item || terminal}<CheckCircle2 size={15} />{:else}<CircleDashed
            size={15}
          />{/if}
        <span
          ><strong>{operation.ams_name}</strong><small
            >{operation.printer_preset_name}</small
          ></span
        >
        <span class="evidence-label"
          >{item
            ? evidenceLabels[item.evidence]
            : operation.action === "skip"
              ? "Skipped"
              : operation.action === "block"
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
