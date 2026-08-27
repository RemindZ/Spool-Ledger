<script lang="ts">
  import { AlertOctagon, Check, CirclePlus, MinusCircle } from "@lucide/svelte";
  import type {
    ConflictChoice,
    ConflictDecision,
    MigrationPlan,
    PlanAction,
    PlanOperation,
  } from "../types";

  let {
    plan,
    conflictDecisions = [],
    onOverride = () => {},
    onDecision = () => {},
  }: {
    plan: MigrationPlan;
    conflictDecisions?: ConflictDecision[];
    onOverride?: (
      operationId: string,
      presetName: string,
      amsName: string,
    ) => void;
    onDecision?: (operationId: string, choice: ConflictChoice | null) => void;
  } = $props();

  const actionLabels: Record<PlanAction, string> = {
    create: "Create",
    add_target: "Add target",
    update: "Update",
    rename: "Rename",
    replace: "Replace",
    skip: "Skip",
    block: "Blocked",
  };

  function iconFor(action: PlanAction) {
    if (action === "block") return AlertOctagon;
    if (action === "skip") return MinusCircle;
    if (action === "create" || action === "add_target") return CirclePlus;
    return Check;
  }

  function decisionFor(operation: PlanOperation) {
    return conflictDecisions.find(
      (decision) =>
        decision.source_id === operation.source_id &&
        decision.printer_id === operation.printer_id &&
        decision.nozzle === operation.nozzle,
    );
  }
</script>

<section class="plan-panel" aria-labelledby="plan-heading">
  <header class="plan-header">
    <div>
      <p class="section-kicker">Frozen preview</p>
      <h2 id="plan-heading">Migration plan</h2>
    </div>
    <div class="plan-metrics">
      <span><strong>{plan.operations.length}</strong> operations</span>
      <span
        class:danger={plan.operations.some((item) => item.action === "block")}
        ><strong
          >{plan.operations.filter((item) => item.action === "block")
            .length}</strong
        > blocked</span
      >
    </div>
  </header>
  <div class="table-scroll">
    <table>
      <thead>
        <tr>
          <th>Source</th>
          <th>Slicing preset</th>
          <th>AMS identity</th>
          <th>Target</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {#each plan.operations as operation (operation.id)}
          {@const ActionIcon = iconFor(operation.action)}
          <tr class:blocked={operation.action === "block"}>
            <td><strong>{operation.source_name}</strong></td>
            <td>
              <input
                class="plan-name-input"
                aria-label={`Slicing name for ${operation.source_name}`}
                value={operation.preset_name}
                onchange={(event) =>
                  onOverride(
                    operation.id,
                    event.currentTarget.value,
                    operation.ams_name,
                  )}
              />
            </td>
            <td>
              <input
                class="plan-name-input"
                aria-label={`AMS name for ${operation.source_name}`}
                value={operation.ams_name}
                onchange={(event) =>
                  onOverride(
                    operation.id,
                    operation.preset_name,
                    event.currentTarget.value,
                  )}
              />
              <code>{operation.filament_id}</code>
            </td>
            <td>{operation.printer_preset_name}</td>
            <td>
              <span class="action-pill" data-action={operation.action}
                ><ActionIcon size={13} /> {actionLabels[operation.action]}</span
              >
              {#if operation.conflict}<small class="conflict-copy"
                  >{operation.conflict.message}</small
                >{/if}
              {#if operation.conflict || decisionFor(operation)}
                <select
                  class="conflict-decision"
                  aria-label={`Conflict decision for ${operation.source_name}`}
                  value={decisionFor(operation)?.choice ?? ""}
                  onchange={(event) => {
                    const choice = event.currentTarget.value as
                      ConflictChoice | "";
                    onDecision(operation.id, choice || null);
                  }}
                >
                  <option value="">Review required</option>
                  <option value="update">Update existing</option>
                  <option value="skip">Skip this output</option>
                  <option value="rename">Rename via destination names</option>
                  <option value="replace">Replace existing</option>
                </select>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
