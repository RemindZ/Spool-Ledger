<script lang="ts">
  import {
    Check,
    FolderPlus,
    LoaderCircle,
    RefreshCw,
    Trash2,
  } from "@lucide/svelte";
  import type { DiscoveryResponse, SourceApp, SourceKind } from "../types";
  import Popover from "./Popover.svelte";

  let {
    discovery,
    presetCount,
    busy,
    onRefresh,
    onAddSource,
    onRemoveSource,
  }: {
    discovery: DiscoveryResponse;
    presetCount: number;
    busy: boolean;
    onRefresh: () => void;
    onAddSource: (sourceApp: SourceApp, sourceKind: SourceKind) => void;
    onRemoveSource: (id: string) => void;
  } = $props();

  let sourceApp = $state<SourceApp>("orca_slicer");
  let sourceKind = $state<SourceKind>("factory_system");
</script>

<Popover class="catalog-status" bodyClass="catalog-status-body">
  {#snippet summary()}Catalog{/snippet}
  <div class="setup-status" role="status" aria-live="polite">
    <span class="status-copy">
      {#if busy}<LoaderCircle size={14} class="spin" /> Refreshing catalog…{:else}<Check
          size={14}
        /> Ready · {presetCount}
        {presetCount === 1 ? "preset" : "presets"}{/if}
    </span>
    <button
      class="btn btn-quiet"
      type="button"
      aria-label="Refresh profile discovery"
      disabled={busy}
      onclick={onRefresh}><RefreshCw size={14} /> Refresh</button
    >
  </div>

  <section class="source-folders" aria-labelledby="source-folders-heading">
    <h3 id="source-folders-heading">Source folders</h3>
    <div class="source-folder-fields">
      <label class="popover-field">
        <span>Application</span>
        <select
          class="select"
          aria-label="Source application"
          bind:value={sourceApp}
        >
          <option value="orca_slicer">OrcaSlicer</option>
          <option value="bambu_studio">Bambu Studio</option>
        </select>
      </label>
      <label class="popover-field">
        <span>Profile kind</span>
        <select class="select" aria-label="Source kind" bind:value={sourceKind}>
          <option value="factory_system">System presets</option>
          <option value="user_custom">Your presets</option>
        </select>
      </label>
    </div>
    <button
      class="btn btn-secondary"
      type="button"
      disabled={busy}
      onclick={() => onAddSource(sourceApp, sourceKind)}
      ><FolderPlus size={14} /> Choose source folder</button
    >
    {#if discovery.manual_source_roots.length}
      <ul class="source-folder-list">
        {#each discovery.manual_source_roots as root (root.id)}
          <li class:invalid={!root.active}>
            <span>
              <strong
                >{root.source_app === "orca_slicer"
                  ? "OrcaSlicer"
                  : "Bambu Studio"} · {root.source_kind === "factory_system"
                  ? "System presets"
                  : "Your presets"}</strong
              >
              <small title={root.path}>{root.path}</small>
              {#if root.error}<small class="field-error">{root.error}</small
                >{/if}
            </span>
            <button
              class="btn btn-quiet"
              type="button"
              aria-label={`Remove manual source ${root.path}`}
              onclick={() => onRemoveSource(root.id)}
              ><Trash2 size={14} /></button
            >
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</Popover>
