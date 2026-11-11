<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    id,
    index,
    title,
    description,
    icon,
    count,
    unit,
    open,
    done,
    revealFrom = "18px",
    class: className = "",
    onOpen,
    children,
  }: {
    id: string;
    index: number;
    title: string;
    description: string;
    icon: string;
    count: number;
    unit: string;
    open: boolean;
    done: boolean;
    revealFrom?: string;
    class?: string;
    onOpen: () => void;
    children: Snippet;
  } = $props();

  const number = $derived(String(index + 1).padStart(2, "0"));
</script>

<article
  {id}
  class={["rail", "step-rail", className, open && "is-open", done && "is-done"]}
  style:--reveal-from={revealFrom}
  aria-labelledby={`${id}-heading`}
>
  <button
    type="button"
    class="rail-spine dark-island"
    aria-controls={id}
    aria-expanded={open}
    aria-label={`Open step ${index + 1}, ${title} · ${count} ${unit}`}
    onclick={onOpen}
  >
    <span class="spine-index">{number}</span>
    <img class="spine-icon" src={icon} width="1024" height="1024" alt="" />
    <span class="spine-title">{title}</span>
    <span class="spine-status"
      ><strong>{count}</strong><small>{unit}</small></span
    >
  </button>
  <header class="rail-head dark-island">
    <span class="rail-icon" aria-hidden="true"
      ><img src={icon} width="1024" height="1024" alt="" /></span
    >
    <div class="rail-head-copy">
      <div class="rail-title">
        <span class="rail-index">{number}</span>
        <h2 id={`${id}-heading`} tabindex="-1">{title}</h2>
      </div>
      <p>{description}</p>
    </div>
  </header>
  <div class="rail-body">
    {@render children()}
  </div>
</article>
