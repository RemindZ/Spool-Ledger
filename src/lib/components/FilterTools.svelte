<script lang="ts">
  import { ChevronDown, Save, Trash2 } from "@lucide/svelte";
  import type { FilterPreset, SourceFilters } from "../state";

  let {
    filters,
    savedPresets = [],
    onFiltersChanged,
    onSavePreset = () => {},
    onLoadPreset = () => {},
    onDeletePreset = () => {},
  }: {
    filters: SourceFilters;
    savedPresets?: FilterPreset[];
    onFiltersChanged: (filters: Partial<SourceFilters>) => void;
    onSavePreset?: (name: string) => void;
    onLoadPreset?: (id: string) => void;
    onDeletePreset?: (id: string) => void;
  } = $props();

  let presetName = $state("");

  function savePreset() {
    const name = presetName.trim();
    if (!name) return;
    onSavePreset(name);
    presetName = "";
  }
</script>

<div class="filter-tools">
  <label class="switch-toggle">
    <input
      type="checkbox"
      role="switch"
      aria-label="Selected only"
      checked={filters.selectedOnly}
      onchange={(event) =>
        onFiltersChanged({ selectedOnly: event.currentTarget.checked })}
    />
    <span>Selected only</span>
  </label>

  <details class="filter-presets">
    <summary>Saved filters <ChevronDown size={12} aria-hidden="true" /></summary
    >
    <div class="filter-preset-controls">
      <label
        ><span>Preset name</span><input
          aria-label="Filter preset name"
          bind:value={presetName}
          placeholder="Panchroma"
        /></label
      >
      <button
        class="secondary-button"
        type="button"
        aria-label="Save filter preset"
        onclick={savePreset}><Save size={13} /> Save</button
      >
      {#if savedPresets.length}
        <label
          ><span>Saved presets</span><select
            aria-label="Saved filter presets"
            onchange={(event) =>
              event.currentTarget.value &&
              onLoadPreset(event.currentTarget.value)}
            ><option value="">Choose preset</option
            >{#each savedPresets as preset (preset.id)}<option value={preset.id}
                >{preset.name}</option
              >{/each}</select
          ></label
        >
        <div class="filter-preset-list">
          {#each savedPresets as preset (preset.id)}<button
              type="button"
              aria-label={`Delete filter preset ${preset.name}`}
              onclick={() => onDeletePreset(preset.id)}
              ><Trash2 size={12} /> {preset.name}</button
            >{/each}
        </div>
      {/if}
    </div>
  </details>
</div>
