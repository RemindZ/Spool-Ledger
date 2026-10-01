<script lang="ts">
  import { RotateCcw, Search } from "@lucide/svelte";
  import FacetMenu from "./FacetMenu.svelte";
  import {
    toggleSetValue,
    type SourceFacets,
    type SourceFilters,
  } from "../state";

  let {
    filters,
    facets,
    onFiltersChanged,
    onReset,
  }: {
    filters: SourceFilters;
    facets: SourceFacets;
    onFiltersChanged: (filters: Partial<SourceFilters>) => void;
    onReset: () => void;
  } = $props();

  type SetDimension =
    | "sourceApps"
    | "sourceKinds"
    | "vendors"
    | "materials"
    | "families"
    | "variants"
    | "compatiblePrinters"
    | "migrationStatuses";
  type Option = { value: string; label: string; count?: number };

  const sourceApps: Option[] = [
    { value: "orca_slicer", label: "OrcaSlicer" },
    { value: "bambu_studio", label: "Bambu Studio" },
  ];
  const sourceKinds: Option[] = [
    { value: "factory_system", label: "Factory / system" },
    { value: "user_custom", label: "User / custom" },
  ];
  const statuses: Option[] = [
    { value: "new", label: "New" },
    { value: "already_migrated", label: "Already migrated" },
    { value: "incomplete", label: "Incomplete" },
    { value: "conflicting", label: "Conflicting" },
    { value: "unsupported", label: "Unsupported" },
  ];

  const facetOptions = (values: SourceFacets[keyof SourceFacets]): Option[] =>
    values.map((facet) => ({
      value: facet.value,
      label: facet.value,
      count: facet.count,
    }));

  const moreGroups = $derived([
    { title: "Source kind", dimension: "sourceKinds", options: sourceKinds },
    {
      title: "Family",
      dimension: "families",
      options: facetOptions(facets.families),
    },
    {
      title: "Variant",
      dimension: "variants",
      options: facetOptions(facets.variants),
    },
    {
      title: "Compatible printer",
      dimension: "compatiblePrinters",
      options: facetOptions(facets.compatiblePrinters),
    },
  ] satisfies Array<{
    title: string;
    dimension: SetDimension;
    options: Option[];
  }>);
  const moreCount = $derived(
    moreGroups.reduce((sum, group) => sum + filters[group.dimension].size, 0),
  );

  function toggle(dimension: SetDimension, value: string) {
    onFiltersChanged({
      [dimension]: toggleSetValue(filters[dimension] as Set<string>, value),
    } as Partial<SourceFilters>);
  }
</script>

{#snippet optionList(dimension: SetDimension, options: Option[])}
  {#each options as option (option.value)}
    <label class="facet-option">
      <input
        type="checkbox"
        aria-label={option.label}
        checked={(filters[dimension] as Set<string>).has(option.value)}
        onchange={() => toggle(dimension, option.value)}
      />
      <span>{option.label}</span>
      {#if option.count !== undefined}<small>{option.count}</small>{/if}
    </label>
  {:else}
    <p class="filter-empty">No values</p>
  {/each}
{/snippet}

<div class="filter-bar" role="search" aria-label="Source filters">
  <label class="search-field">
    <span class="sr-only">Search source profiles</span>
    <Search size={15} />
    <input
      type="search"
      aria-label="Search source profiles"
      placeholder="Search profiles"
      value={filters.search}
      oninput={(event) =>
        onFiltersChanged({ search: event.currentTarget.value })}
    />
  </label>
  <FacetMenu label="Source app" activeCount={filters.sourceApps.size}>
    {@render optionList("sourceApps", sourceApps)}
  </FacetMenu>
  <FacetMenu label="Manufacturer" activeCount={filters.vendors.size}>
    {@render optionList("vendors", facetOptions(facets.vendors))}
  </FacetMenu>
  <FacetMenu label="Material" activeCount={filters.materials.size}>
    {@render optionList("materials", facetOptions(facets.materials))}
  </FacetMenu>
  <FacetMenu label="Status" activeCount={filters.migrationStatuses.size}>
    {@render optionList("migrationStatuses", statuses)}
  </FacetMenu>
  <FacetMenu label="More filters" activeCount={moreCount} more>
    {#each moreGroups as group (group.dimension)}
      <div class="facet-group-block">
        <p class="facet-group-title">{group.title}</p>
        {@render optionList(group.dimension, group.options)}
      </div>
    {/each}
  </FacetMenu>
  <button
    class="icon-button subtle"
    type="button"
    aria-label="Reset filters"
    title="Reset filters"
    onclick={onReset}><RotateCcw size={14} /></button
  >
</div>
