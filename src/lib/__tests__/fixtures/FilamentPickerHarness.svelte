<script lang="ts">
  import FilamentPicker from "../../components/FilamentPicker.svelte";
  import { withValue } from "../../picker";
  import {
    defaultFilters,
    type FilterPreset,
    type SourceFilters,
  } from "../../state";
  import type { CatalogSource, DiscoveryResponse } from "../../types";

  let {
    sources,
    discovery,
    historyActive = true,
    initialSelected = [],
    savedPresets = [],
    onSelectionChange = () => {},
    onSavePreset = () => {},
    onLoadPreset = () => {},
    onDeletePreset = () => {},
    onRefresh = () => {},
    onAddSource = () => {},
    onRemoveSource = () => {},
  }: {
    sources: CatalogSource[];
    discovery: DiscoveryResponse;
    historyActive?: boolean;
    initialSelected?: string[];
    savedPresets?: FilterPreset[];
    onSelectionChange?: (ids: string[]) => void;
    onSavePreset?: (name: string) => void;
    onLoadPreset?: (id: string) => void;
    onDeletePreset?: (id: string) => void;
    onRefresh?: () => void;
    onAddSource?: () => void;
    onRemoveSource?: (id: string) => void;
  } = $props();

  let filters = $state<SourceFilters>(defaultFilters());
  // svelte-ignore state_referenced_locally
  let selected = $state(new Set<string>(initialSelected));

  function update(next: Set<string>) {
    selected = next;
    onSelectionChange([...next].sort());
  }
</script>

<FilamentPicker
  {sources}
  {filters}
  selectedIds={selected}
  {historyActive}
  {discovery}
  {savedPresets}
  onFiltersChanged={(changes) => (filters = { ...filters, ...changes })}
  onToggle={(id) => update(withValue(selected, id, !selected.has(id)))}
  onSelect={(ids, value) =>
    update(
      new Set(
        value
          ? [...selected, ...ids]
          : [...selected].filter((id) => !ids.includes(id)),
      ),
    )}
  onClearSelection={() => update(new Set())}
  {onSavePreset}
  {onLoadPreset}
  {onDeletePreset}
  {onRefresh}
  {onAddSource}
  {onRemoveSource}
/>
