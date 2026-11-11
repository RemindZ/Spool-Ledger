<script lang="ts">
  import { Save, Trash2 } from "@lucide/svelte";
  import type { FilterPreset } from "../state";
  import Popover from "./Popover.svelte";

  let {
    savedPresets = [],
    onSavePreset,
    onLoadPreset,
    onDeletePreset,
  }: {
    savedPresets?: FilterPreset[];
    onSavePreset: (name: string) => void;
    onLoadPreset: (id: string) => void;
    onDeletePreset: (id: string) => void;
  } = $props();

  let presetName = $state("");

  function savePreset() {
    const name = presetName.trim();
    if (!name) return;
    onSavePreset(name);
    presetName = "";
  }
</script>

<Popover class="saved-filters" bodyClass="saved-filters-body">
  {#snippet summary()}Saved filters{/snippet}
  <label class="popover-field">
    <span>Preset name</span>
    <input
      class="input"
      aria-label="Filter preset name"
      bind:value={presetName}
      placeholder="Everyday materials"
      onkeydown={(event) => {
        if (event.key === "Enter") savePreset();
      }}
    />
  </label>
  <div class="row-actions">
    <button
      class="btn btn-secondary"
      type="button"
      aria-label="Save filter preset"
      disabled={!presetName.trim()}
      onclick={savePreset}><Save size={14} /> Save</button
    >
  </div>
  {#if savedPresets.length}
    <ul class="saved-filter-list" aria-label="Saved filter presets">
      {#each savedPresets as preset (preset.id)}
        <li>
          <span>{preset.name}</span>
          <button
            class="btn btn-quiet"
            type="button"
            aria-label={`Load filter preset ${preset.name}`}
            onclick={() => onLoadPreset(preset.id)}>Load</button
          >
          <button
            class="btn btn-quiet"
            type="button"
            aria-label={`Delete filter preset ${preset.name}`}
            onclick={() => onDeletePreset(preset.id)}
            ><Trash2 size={14} /></button
          >
        </li>
      {/each}
    </ul>
  {/if}
</Popover>
