<script lang="ts">
  import { RotateCcw, Save, Search, SlidersHorizontal, Trash2 } from '@lucide/svelte';
  import { toggleSetValue, type FilterFacet, type FilterPreset, type SourceFilters, type SourceFacets } from '../state';
  import type { MigrationStatus, SourceApp, SourceKind } from '../types';

  let {
    filters,
    facets,
    savedPresets = [],
    onFiltersChanged,
    onReset,
    onSavePreset = () => {},
    onLoadPreset = () => {},
    onDeletePreset = () => {},
  }: {
    filters: SourceFilters;
    facets: SourceFacets;
    savedPresets?: FilterPreset[];
    onFiltersChanged: (filters: Partial<SourceFilters>) => void;
    onReset: () => void;
    onSavePreset?: (name: string) => void;
    onLoadPreset?: (id: string) => void;
    onDeletePreset?: (id: string) => void;
  } = $props();

  let presetName = $state('');

  function savePreset() {
    const name = presetName.trim();
    if (!name) return;
    onSavePreset(name);
    presetName = '';
  }

  const sourceApps: Array<{ value: SourceApp; label: string }> = [
    { value: 'orca_slicer', label: 'OrcaSlicer' },
    { value: 'bambu_studio', label: 'Bambu Studio' },
  ];
  const sourceKinds: Array<{ value: SourceKind; label: string }> = [
    { value: 'factory_system', label: 'Factory / system' },
    { value: 'user_custom', label: 'User / custom' },
  ];
  const statuses: Array<{ value: MigrationStatus; label: string }> = [
    { value: 'new', label: 'New' },
    { value: 'already_migrated', label: 'Already migrated' },
    { value: 'incomplete', label: 'Incomplete' },
    { value: 'conflicting', label: 'Conflicting' },
    { value: 'unsupported', label: 'Unsupported' },
  ];

  function facetOptions(title: string, dimension: 'vendors' | 'materials' | 'families' | 'variants' | 'compatiblePrinters', values: FilterFacet[]) {
    return { title, dimension, values };
  }

  const groupedFacets = $derived([
    facetOptions('Manufacturer', 'vendors', facets.vendors),
    facetOptions('Material', 'materials', facets.materials),
    facetOptions('Family', 'families', facets.families),
    facetOptions('Variant', 'variants', facets.variants),
    facetOptions('Compatible printer', 'compatiblePrinters', facets.compatiblePrinters),
  ]);
</script>

<aside class="filter-panel" aria-labelledby="filters-heading">
  <header class="filter-heading">
    <div><SlidersHorizontal size={16} /><h2 id="filters-heading">Source filters</h2></div>
    <button class="icon-button subtle" type="button" aria-label="Reset filters" onclick={onReset}><RotateCcw size={14} /></button>
  </header>

  <label class="search-field">
    <span class="sr-only">Search source profiles</span>
    <Search size={15} />
    <input
      type="search"
      aria-label="Search source profiles"
      placeholder="Search profiles"
      value={filters.search}
      oninput={(event) => onFiltersChanged({ search: event.currentTarget.value })}
    />
  </label>

  <fieldset class="filter-group source-origin">
    <legend>Source app</legend>
    {#each sourceApps as app (app.value)}
      <label><input type="checkbox" aria-label={app.label} checked={filters.sourceApps.has(app.value)} onchange={() => onFiltersChanged({ sourceApps: toggleSetValue(filters.sourceApps, app.value) })} /><span>{app.label}</span></label>
    {/each}
  </fieldset>

  <fieldset class="filter-group">
    <legend>Source kind</legend>
    {#each sourceKinds as kind (kind.value)}
      <label><input type="checkbox" aria-label={kind.label} checked={filters.sourceKinds.has(kind.value)} onchange={() => onFiltersChanged({ sourceKinds: toggleSetValue(filters.sourceKinds, kind.value) })} /><span>{kind.label}</span></label>
    {/each}
  </fieldset>

  {#each groupedFacets as group (group.dimension)}
    <details class="facet-group" open={group.dimension === 'vendors' || group.dimension === 'materials'}>
      <summary>{group.title}<span>{filters[group.dimension].size || group.values.length}</span></summary>
      <div class="facet-options">
        {#each group.values as facet (facet.value)}
          <label>
            <input
              type="checkbox"
              checked={filters[group.dimension].has(facet.value)}
              onchange={() => onFiltersChanged({ [group.dimension]: toggleSetValue(filters[group.dimension], facet.value) })}
            />
            <span>{facet.value}</span><small>{facet.count}</small>
          </label>
        {:else}
          <p class="filter-empty">No values</p>
        {/each}
      </div>
    </details>
  {/each}

  <fieldset class="filter-group status-filter">
    <legend>Migration status</legend>
    {#each statuses as status (status.value)}
      <label><input type="checkbox" checked={filters.migrationStatuses.has(status.value)} onchange={() => onFiltersChanged({ migrationStatuses: toggleSetValue(filters.migrationStatuses, status.value) })} /><span>{status.label}</span></label>
    {/each}
  </fieldset>

  <label class="selected-only"><input type="checkbox" aria-label="Selected only" checked={filters.selectedOnly} onchange={(event) => onFiltersChanged({ selectedOnly: event.currentTarget.checked })} /><span>Selected only</span></label>

  <details class="filter-presets">
    <summary>Saved filter presets</summary>
    <div class="filter-preset-controls">
      <label><span>Preset name</span><input aria-label="Filter preset name" bind:value={presetName} placeholder="Panchroma" /></label>
      <button class="secondary-button" type="button" aria-label="Save filter preset" onclick={savePreset}><Save size={13} /> Save</button>
      {#if savedPresets.length}
        <label><span>Saved presets</span><select aria-label="Saved filter presets" onchange={(event) => event.currentTarget.value && onLoadPreset(event.currentTarget.value)}><option value="">Choose preset</option>{#each savedPresets as preset (preset.id)}<option value={preset.id}>{preset.name}</option>{/each}</select></label>
        <div class="filter-preset-list">{#each savedPresets as preset (preset.id)}<button type="button" aria-label={`Delete filter preset ${preset.name}`} onclick={() => onDeletePreset(preset.id)}><Trash2 size={12} /> {preset.name}</button>{/each}</div>
      {/if}
    </div>
  </details>
</aside>
