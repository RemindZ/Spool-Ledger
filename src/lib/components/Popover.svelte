<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    class: className = "",
    bodyClass = "",
    bodyLabel,
    filtered = false,
    open = $bindable(false),
    summary,
    children,
    onOpen = () => {},
  }: {
    class?: string;
    bodyClass?: string;
    bodyLabel?: string;
    filtered?: boolean;
    open?: boolean;
    summary: Snippet;
    children: Snippet;
    onOpen?: () => void;
  } = $props();

  let root: HTMLDetailsElement;

  $effect(() => {
    if (!open) return;
    const closeOutside = (event: PointerEvent) => {
      if (!root.contains(event.target as Node)) open = false;
    };
    window.addEventListener("pointerdown", closeOutside);
    window.addEventListener("keydown", closeOnEscape);
    return () => {
      window.removeEventListener("pointerdown", closeOutside);
      window.removeEventListener("keydown", closeOnEscape);
    };
  });

  function closeOnEscape(event: KeyboardEvent) {
    if (!open || event.key !== "Escape") return;
    event.preventDefault();
    open = false;
    root.querySelector("summary")?.focus();
  }
</script>

<details
  bind:this={root}
  class={["popover", className, filtered && "is-filtered"]}
  bind:open
  ontoggle={(event) => {
    if (event.currentTarget.open) onOpen();
  }}
>
  <summary>{@render summary()}</summary>
  <div
    class={["popover-body", bodyClass]}
    role={bodyLabel ? "group" : undefined}
    aria-label={bodyLabel}
  >
    {@render children()}
  </div>
</details>
