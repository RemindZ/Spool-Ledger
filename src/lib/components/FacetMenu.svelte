<script lang="ts">
  import type { Snippet } from "svelte";
  import { ChevronDown, ListFilter } from "@lucide/svelte";

  let {
    label,
    activeCount = 0,
    more = false,
    children,
  }: {
    label: string;
    activeCount?: number;
    more?: boolean;
    children: Snippet;
  } = $props();

  const menuId = $props.id();
  let open = $state(false);
  let root: HTMLDivElement;
  let chip: HTMLButtonElement;

  $effect(() => {
    if (!open) return;
    const closeOutside = (event: PointerEvent) => {
      if (!root.contains(event.target as Node)) open = false;
    };
    window.addEventListener("pointerdown", closeOutside);
    return () => window.removeEventListener("pointerdown", closeOutside);
  });

  function closeOnEscape(event: KeyboardEvent) {
    if (!open || event.key !== "Escape") return;
    event.preventDefault();
    open = false;
    chip?.focus();
  }
</script>

<svelte:window onkeydown={closeOnEscape} />

<div class="facet-menu" bind:this={root}>
  <button
    bind:this={chip}
    class="filter-chip"
    class:active={activeCount > 0}
    class:more
    type="button"
    aria-label={activeCount > 0 ? `${label}, ${activeCount} selected` : label}
    aria-expanded={open}
    aria-controls={menuId}
    onclick={() => (open = !open)}
  >
    {#if more}<ListFilter size={13} aria-hidden="true" />{/if}
    {label}
    {#if activeCount > 0}<span class="filter-chip-count" aria-hidden="true"
        >{activeCount}</span
      >{/if}
    {#if !more}<ChevronDown size={13} aria-hidden="true" />{/if}
  </button>
  {#if open}
    <div id={menuId} class="facet-popover" role="group" aria-label={label}>
      {@render children()}
    </div>
  {/if}
</div>
