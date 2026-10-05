<script lang="ts">
  import { formatSeconds, type RailStepState, type RunRail } from "../run-rail";

  let { rail }: { rail: RunRail } = $props();

  const STATE_TEXT: Record<RailStepState, string> = {
    pending: "Pending",
    active: "In progress",
    done: "Complete",
    skipped: "Skipped",
    failed: "Failed",
    stopped: "Stopped",
  };

  const doneCount = $derived(
    rail.steps.filter((step) => step.state === "done").length,
  );
  const barWidth = $derived(
    rail.finished ? 100 : Math.round((doneCount / rail.steps.length) * 100),
  );

  function stateText(state: RailStepState, index: number): string {
    if (rail.finished && index === rail.steps.length - 1)
      return rail.stopped ? "Stopped" : "Complete";
    return STATE_TEXT[state];
  }
</script>

<aside class="panel run-rail" aria-label="Transaction steps">
  <div class="run-rail-head dark-island">
    <span class="run-rail-kicker">Transaction</span>
    <p class="phase-summary" aria-live="polite">{rail.summary}</p>
    <div class="progress" aria-hidden="true">
      <span style:width={`${barWidth}%`}></span>
    </div>
  </div>
  <ol
    class="phase-list"
    class:is-finished={rail.finished}
    class:is-stopped={rail.stopped}
    style:--phase-progress={rail.progress}
  >
    {#each rail.steps as step, index (step.name)}
      <li
        class={["phase", step.state !== "pending" && step.state]}
        aria-current={index === rail.activeIndex ? "step" : undefined}
      >
        <span class="phase-node" aria-hidden="true">
          <span class="phase-num">{String(index + 1).padStart(2, "0")}</span>
          <svg class="phase-check" viewBox="0 0 16 16"
            ><path d="M3.5 8.5l3 3 6-7" /></svg
          >
        </span>
        <span class="phase-copy">
          <span class="phase-name">{step.name}</span>
          <span class="phase-hint"><small>{step.hint}</small></span>
          <span class="sr-only">{stateText(step.state, index)}</span>
        </span>
        <span class="phase-time"
          >{#if step.durationMs !== null}{formatSeconds(
              step.durationMs,
            )}{:else if step.state === "skipped"}Skipped{/if}</span
        >
      </li>
    {/each}
  </ol>
</aside>
