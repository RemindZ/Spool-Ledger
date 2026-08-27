<script module lang="ts">
  export type WorkspaceView = "setup" | "plan" | "activity" | "restore";
</script>

<script lang="ts">
  import {
    Activity,
    ArchiveRestore,
    ClipboardList,
    SlidersHorizontal,
  } from "@lucide/svelte";

  let {
    active,
    planCount = 0,
    onChange,
  }: {
    active: WorkspaceView;
    planCount?: number;
    onChange: (view: WorkspaceView) => void;
  } = $props();

  const tabs = [
    { view: "setup", label: "Setup", icon: SlidersHorizontal },
    { view: "plan", label: "Review plan", icon: ClipboardList },
    { view: "activity", label: "Run & evidence", icon: Activity },
    { view: "restore", label: "Restore", icon: ArchiveRestore },
  ] as const;

  function select(view: WorkspaceView) {
    onChange(view);
  }

  function handleKeydown(event: KeyboardEvent, index: number) {
    const target =
      event.key === "ArrowLeft"
        ? index - 1
        : event.key === "ArrowRight"
          ? index + 1
          : event.key === "Home"
            ? 0
            : event.key === "End"
              ? tabs.length - 1
              : null;
    if (target === null) return;
    event.preventDefault();
    const next = tabs[(target + tabs.length) % tabs.length];
    select(next.view);
    document.getElementById(`${next.view}-tab`)?.focus();
  }
</script>

<div class="workspace-tabs" role="tablist" aria-label="Application sections">
  {#each tabs as tab, index (tab.view)}
    <button
      type="button"
      id={`${tab.view}-tab`}
      role="tab"
      aria-selected={active === tab.view}
      aria-controls={`${tab.view}-view`}
      tabindex={active === tab.view ? 0 : -1}
      onclick={() => select(tab.view)}
      onkeydown={(event) => handleKeydown(event, index)}
    >
      <tab.icon size={15} aria-hidden="true" />
      {tab.label}
      {#if tab.view === "plan" && planCount > 0}
        <span class="tab-count">{planCount}</span>
      {/if}
    </button>
  {/each}
</div>
