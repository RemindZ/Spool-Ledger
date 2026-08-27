<script lang="ts">
  import { Check, Circle, LoaderCircle, Minus } from "@lucide/svelte";
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
</script>

<section class="execution-phases" aria-labelledby="execution-phases-heading">
  <header>
    <p class="section-kicker">Command evidence</p>
    <h2 id="execution-phases-heading">Execution phases</h2>
  </header>
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
</section>
