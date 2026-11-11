<script lang="ts">
  import { ArrowLeft } from "@lucide/svelte";

  let {
    step,
    steps,
    sources,
    targets,
    outputs,
    ready,
    readinessLabel,
    building = false,
    onBack,
    onContinue,
  }: {
    step: number;
    steps: ReadonlyArray<{ title: string; hint: string; next: string }>;
    sources: number;
    targets: number;
    outputs: number;
    ready: boolean;
    readinessLabel: string;
    building?: boolean;
    onBack: () => void;
    onContinue: () => void;
  } = $props();

  const plural = (count: number, one: string) =>
    count === 1 ? one : `${one}s`;
</script>

<div class="actionbar route-bar">
  <div class="step-progress">
    <ol class="route-strand" aria-hidden="true">
      {#each steps as item, index (item.title)}
        <li class:current={index === step} class:done={index < step}></li>
      {/each}
    </ol>
    <p>
      <strong>Step {step + 1} of {steps.length} · {steps[step].title}</strong>
      <small>{steps[step].hint}</small>
    </p>
  </div>
  <div class="summary">
    <span><strong>{sources}</strong> {plural(sources, "source")}</span>
    <span><strong>{targets}</strong> {plural(targets, "target")}</span>
    <span><strong>{outputs}</strong> {plural(outputs, "output")}</span>
  </div>
  <span class="tag plan-state" class:warning={!ready}>{readinessLabel}</span>
  {#if step > 0}
    <button type="button" class="btn btn-quiet route-back" onclick={onBack}
      ><ArrowLeft size={15} aria-hidden="true" /> Back</button
    >
  {/if}
  <button
    class="btn btn-primary route-next"
    class:is-building={building}
    type="button"
    aria-busy={building ? "true" : undefined}
    disabled={!ready || building}
    onclick={onContinue}
    ><span>{building ? "Building plan…" : steps[step].next}</span><span
      aria-hidden="true">→</span
    ></button
  >
</div>
