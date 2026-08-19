<script lang="ts">
  import { AlertTriangle, CheckCircle2, CircleDotDashed, Layers3 } from '@lucide/svelte';
  import type { CatalogSource, MigrationStatus } from '../types';

  let {
    sources,
    selectedIds,
    totalCount,
    onToggle,
    onSelectVisible,
    onClearSelection,
  }: {
    sources: CatalogSource[];
    selectedIds: Set<string>;
    totalCount: number;
    onToggle: (sourceId: string) => void;
    onSelectVisible: (selected: boolean) => void;
    onClearSelection: () => void;
  } = $props();

  const statusLabels: Record<MigrationStatus, string> = {
    new: 'New',
    already_migrated: 'Migrated',
    incomplete: 'Incomplete',
    conflicting: 'Conflict',
    unsupported: 'Unsupported',
  };

  const selectedVisible = $derived(sources.filter((source) => selectedIds.has(source.id)).length);
</script>

<section class="source-panel" aria-labelledby="sources-heading">
  <header class="source-heading">
    <div>
      <p class="section-kicker">Source inventory</p>
      <h2 id="sources-heading">Filament profiles</h2>
    </div>
    <div class="source-counts">
      <strong>{sources.length} of {totalCount} profiles</strong>
      <span>{selectedIds.size} selected</span>
    </div>
    <div class="source-actions">
      <button class="text-button" type="button" onclick={() => onSelectVisible(true)}>Select visible</button>
      {#if selectedVisible > 0}<button class="text-button muted" type="button" onclick={() => onSelectVisible(false)}>Unselect visible</button>{/if}
      {#if selectedIds.size > 0}<button class="text-button muted" type="button" onclick={onClearSelection}>Clear all</button>{/if}
    </div>
  </header>

  <div class="source-table-wrap">
    <table class="source-table">
      <thead>
        <tr><th class="check-column"><span class="sr-only">Selected</span></th><th>Profile</th><th>Material route</th><th>Compatibility</th><th>Status</th></tr>
      </thead>
      <tbody>
        {#each sources as source (source.id)}
          <tr class:selected={selectedIds.has(source.id)}>
            <td class="check-column"><input type="checkbox" aria-label={`Select ${source.name}`} checked={selectedIds.has(source.id)} onchange={() => onToggle(source.id)} /></td>
            <td>
              <strong>{source.name}</strong>
              <span class="provenance" data-app={source.source_app}><span>{source.source_app === 'orca_slicer' ? 'OrcaSlicer' : 'Bambu Studio'}</span><i aria-hidden="true">·</i><span>{source.source_kind === 'factory_system' ? 'Factory / system' : 'User / custom'}</span></span>
              {#each source.warnings as warning (`${source.id}:${warning}`)}<small class="source-warning"><AlertTriangle size={12} /> {warning}</small>{/each}
            </td>
            <td><strong>{source.vendor}</strong><span>{source.material} · {source.family} · {source.variant}</span></td>
            <td>{#each source.compatible_printers.slice(0, 2) as printer (printer)}<span class="printer-tag">{printer}</span>{/each}{#if source.compatible_printers.length > 2}<small>+{source.compatible_printers.length - 2} more</small>{/if}</td>
            <td><span class="status-pill" data-status={source.migration_status}>{#if source.migration_status === 'already_migrated'}<CheckCircle2 size={12} />{:else if source.migration_status === 'conflicting' || source.migration_status === 'unsupported'}<AlertTriangle size={12} />{:else}<CircleDotDashed size={12} />{/if}{statusLabels[source.migration_status]}</span></td>
          </tr>
        {:else}
          <tr><td colspan="5"><div class="panel-empty"><Layers3 size={26} /><strong>No profiles match these filters</strong><p>Reset filters or keep selections and search a different source.</p></div></td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
