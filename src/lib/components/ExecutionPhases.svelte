<script lang="ts">
  import {
    Check,
    ChevronDown,
    Circle,
    LoaderCircle,
    Minus,
  } from "@lucide/svelte";
  import type { ExecutionPhase, SyncPhase } from "../types";

  let {
    localPhase = null,
    syncPhase = null,
    syncExpected = true,
  }: {
    localPhase?: ExecutionPhase | null;
    syncPhase?: SyncPhase | null;
    syncExpected?: boolean;
  } = $props();

  const localSteps: Array<{ phase: ExecutionPhase; label: string }> = [
    { phase: "validating", label: "Validate frozen plan" },
    { phase: "closing_bambu", label: "Close Bambu Studio safely" },
    { phase: "staging", label: "Stage local profiles" },
    { phase: "validating_output", label: "Validate generated output" },
    { phase: "backing_up", label: "Create path-owned backup" },
    { phase: "committing", label: "Commit local profiles" },
    { phase: "writing_receipt", label: "Write journal and receipt" },
    { phase: "finished", label: "Finish local transaction" },
  ];
  const syncSteps: Array<{ phase: SyncPhase; label: string }> = [
    { phase: "launching", label: "Launch Bambu Studio" },
    { phase: "monitoring", label: "Observe Bambu evidence" },
    { phase: "finished", label: "Finish synchronization" },
  ];

  type StepState = "pending" | "active" | "complete" | "skipped";

  function phaseState<T extends string>(
    phases: readonly T[],
    current: T | null,
    index: number,
  ): StepState {
    if (current === null) return "pending";
    const currentIndex = phases.indexOf(current);
    if (currentIndex === phases.length - 1 || index < currentIndex) {
      return "complete";
    }
    return index === currentIndex ? "active" : "pending";
  }

  function localState(index: number): StepState {
    return phaseState(
      localSteps.map((step) => step.phase),
      localPhase,
      index,
    );
  }

  function syncState(index: number): StepState {
    if (!syncExpected && localPhase === "finished") return "skipped";
    if (localPhase !== "finished") return "pending";
    return phaseState(
      syncSteps.map((step) => step.phase),
      syncPhase,
      index,
    );
  }

  const steps = $derived([
    ...localSteps.map((step, index) => ({
      label: step.label,
      state: localState(index),
    })),
    ...syncSteps.map((step, index) => ({
      label: step.label,
      state: syncState(index),
    })),
  ]);
  const applicable = $derived(steps.filter((step) => step.state !== "skipped"));
  const summary = $derived.by(() => {
    const active = applicable.findIndex((step) => step.state === "active");
    if (active >= 0) {
      return `Step ${active + 1} of ${applicable.length}: ${applicable[active].label}`;
    }
    const complete = applicable.filter(
      (step) => step.state === "complete",
    ).length;
    if (complete === applicable.length) {
      return `All ${applicable.length} steps complete`;
    }
    return complete === 0
      ? "Waiting to start"
      : `${complete} of ${applicable.length} steps complete`;
  });
</script>

<section class="execution-phases" aria-label="Execution phases">
  <div class="phase-summary">
    <strong>{summary}</strong>
    <span class="phase-segments" aria-hidden="true">
      {#each steps as step, index (index)}<i data-state={step.state}></i>{/each}
    </span>
  </div>
  <details class="phase-details">
    <summary
      ><span class="phase-details-closed">Show steps</span><span
        class="phase-details-open">Hide steps</span
      ><ChevronDown size={13} /></summary
    >
    <ol>
      {#each localSteps as step, index (step.phase)}
        {@const state = localState(index)}
        <li
          data-state={state}
          aria-current={state === "active" ? "step" : undefined}
        >
          <span class="phase-mark" aria-hidden="true">
            {#if state === "complete"}<Check
                size={13}
              />{:else if state === "active"}<LoaderCircle
                size={13}
                class="spin"
              />{:else}<Circle size={10} />{/if}
          </span>
          <span>{step.label}</span>
          <small>{state}</small>
        </li>
      {/each}
      {#each syncSteps as step, index (step.phase)}
        {@const state = syncState(index)}
        <li
          data-state={state}
          aria-current={state === "active" ? "step" : undefined}
        >
          <span class="phase-mark" aria-hidden="true">
            {#if state === "complete"}<Check
                size={13}
              />{:else if state === "active"}<LoaderCircle
                size={13}
                class="spin"
              />{:else if state === "skipped"}<Minus size={11} />{:else}<Circle
                size={10}
              />{/if}
          </span>
          <span>{step.label}</span>
          <small>{state}</small>
        </li>
      {/each}
    </ol>
  </details>
</section>
