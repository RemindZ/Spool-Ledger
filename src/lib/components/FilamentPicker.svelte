<script lang="ts">
  import { AlertTriangle, ArrowLeft, Search } from "@lucide/svelte";
  import { tick } from "svelte";
  import ChecklistMenu, { type ChecklistGroup } from "./ChecklistMenu.svelte";
  import CatalogMenu from "./CatalogMenu.svelte";
  import SavedFilters from "./SavedFilters.svelte";
  import {
    SOURCE_OPTIONS,
    checklistTriggerLabel,
    emptyHistory,
    emptyStateKind,
    groupPresets,
    hiddenSelectedCount,
    isTiered,
    materialCards,
    materialLabel,
    normalizePath,
    recordNavigation,
    scopeSelection,
    searchGroups,
    setShownValues,
    sourceTriggerLabel,
    travel,
    typeCards,
    typesOf,
    vendorTriggerLabel,
    vendorsOf,
    viewSources,
    withValue,
    type DrillHistory,
    type PickerPath,
    type PickerView,
  } from "../picker";
  import {
    MIGRATION_STATUSES,
    SOURCE_KEYS,
    sourceKey,
    type FilterPreset,
    type SourceFilters,
    type SourceKey,
  } from "../state";
  import type {
    CatalogSource,
    DiscoveryResponse,
    MigrationStatus,
    SourceApp,
    SourceKind,
  } from "../types";

  let {
    sources,
    filters,
    selectedIds,
    historyActive = false,
    initialPath = [],
    discovery,
    busy = false,
    savedPresets = [],
    onFiltersChanged,
    onToggle,
    onSelect,
    onClearSelection,
    onSavePreset,
    onLoadPreset,
    onDeletePreset,
    onRefresh,
    onAddSource,
    onRemoveSource,
  }: {
    sources: CatalogSource[];
    filters: SourceFilters;
    selectedIds: Set<string>;
    /** Mouse Back/Forward and Alt+arrows act only while the step is in front. */
    historyActive?: boolean;
    /** Opening level, for example a restored or documented drilldown path. */
    initialPath?: PickerPath;
    discovery: DiscoveryResponse;
    busy?: boolean;
    savedPresets?: FilterPreset[];
    onFiltersChanged: (filters: Partial<SourceFilters>) => void;
    onToggle: (sourceId: string) => void;
    onSelect: (sourceIds: string[], selected: boolean) => void;
    onClearSelection: () => void;
    onSavePreset: (name: string) => void;
    onLoadPreset: (id: string) => void;
    onDeletePreset: (id: string) => void;
    onRefresh: () => void;
    onAddSource: (sourceApp: SourceApp, sourceKind: SourceKind) => void;
    onRemoveSource: (id: string) => void;
  } = $props();

  const STATUS_LABELS: Record<MigrationStatus, string> = {
    new: "New",
    already_migrated: "Already migrated",
    incomplete: "Incomplete",
    conflicting: "Conflicting",
    unsupported: "Unsupported",
  };

  // svelte-ignore state_referenced_locally
  let path = $state<PickerPath>([...initialPath]);
  let history = $state<DrillHistory>(emptyHistory());
  let motion = $state<"forward" | "back" | null>(null);
  let announcement = $state("");
  let vendorQuery = $state("");
  let printerQuery = $state("");
  let stage = $state<HTMLDivElement>();
  let crumbs = $state<HTMLElement>();
  let motionTimer: ReturnType<typeof setTimeout> | undefined;

  const view = $derived<PickerView>({
    sources,
    filters,
    selected: selectedIds,
    path,
    query: filters.search,
  });
  const query = $derived(filters.search.trim());
  const level = $derived(
    query ? "search" : !path[0] ? "materials" : !path[1] ? "types" : "presets",
  );
  const shown = $derived(viewSources(view));
  const scope = $derived(scopeSelection(view, path));
  const empty = $derived(emptyStateKind(view));
  const pathTiered = $derived(
    Boolean(path[0] && path[1] && isTiered(view, path[0])),
  );
  const multiType = $derived(
    Boolean(path[0]) && typesOf(viewSources(view, [path[0]])).length > 1,
  );
  const vendors = $derived(vendorsOf(sources));
  const printers = $derived(
    [...new Set(sources.flatMap((source) => source.compatible_printers))].sort(
      (left, right) => left.localeCompare(right),
    ),
  );
  const hidden = $derived(hiddenSelectedCount(view));

  const plural = (count: number, one: string, many = `${one}s`) =>
    `${count} ${count === 1 ? one : many}`;

  function statusText(): string {
    if (empty) return "No presets shown";
    if (level === "search")
      return `${plural(shown.length, "match", "matches")} for ${query}`;
    if (level === "materials") return "All materials";
    if (level === "types")
      return `${materialLabel(path[0])}: ${plural(typeCards(view).length, "type")}`;
    return `${materialLabel(path[0])} ${path[1]}: ${plural(shown.length, "preset")}`;
  }

  async function go(
    next: PickerPath,
    direction: "forward" | "back",
    { record = true, exact = false } = {},
  ) {
    const target = normalizePath(view, next, direction, exact);
    if (record) history = recordNavigation(history, path, target);
    path = target;
    motion = direction;
    clearTimeout(motionTimer);
    motionTimer = setTimeout(() => (motion = null), 320);
    if (stage) stage.scrollTop = 0;
    announcement = statusText();
    await tick();
    crumbs
      ?.querySelector<HTMLElement>("[aria-current]")
      ?.focus({ preventScroll: true });
  }

  function retrace(direction: -1 | 1) {
    const step = travel(history, path, direction);
    if (!step) return;
    history = step.history;
    void go(step.path, direction < 0 ? "back" : "forward", {
      record: false,
      exact: true,
    });
  }

  $effect(() => {
    if (!historyActive) return;
    // Mouse buttons 3 and 4 are Back and Forward. Both presses are cancelled
    // so the WebView never navigates the page itself.
    const onMouse = (event: MouseEvent) => {
      if (event.button !== 3 && event.button !== 4) return;
      event.preventDefault();
      if (event.type === "mouseup") retrace(event.button === 3 ? -1 : 1);
    };
    const onKey = (event: KeyboardEvent) => {
      if (
        !event.altKey ||
        (event.key !== "ArrowLeft" && event.key !== "ArrowRight")
      )
        return;
      event.preventDefault();
      retrace(event.key === "ArrowLeft" ? -1 : 1);
    };
    document.addEventListener("mousedown", onMouse);
    document.addEventListener("mouseup", onMouse);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onMouse);
      document.removeEventListener("mouseup", onMouse);
      document.removeEventListener("keydown", onKey);
    };
  });

  function toggleIds(ids: string[]) {
    onSelect(ids, !ids.every((id) => selectedIds.has(id)));
  }

  function presetName(source: CatalogSource, withVendor: boolean): string {
    if (withVendor) return source.name;
    const prefix = `${source.vendor} `;
    return source.name.startsWith(prefix)
      ? source.name.slice(prefix.length)
      : source.name;
  }

  const sourceGroups = $derived<ChecklistGroup[]>(
    (["OrcaSlicer", "Bambu Studio"] as const).map((group) => ({
      title: group,
      options: SOURCE_OPTIONS.filter((option) => option.group === group).map(
        (option) => ({
          value: option.key,
          label: option.label,
          count: sources.filter(
            (source) =>
              sourceKey(source) === option.key &&
              !filters.hiddenVendors.has(source.vendor),
          ).length,
          checked: filters.sources.has(option.key),
        }),
      ),
    })),
  );
  const vendorGroups = $derived<ChecklistGroup[]>([
    {
      options: vendors.map((vendor) => ({
        value: vendor,
        label: vendor,
        count: sources.filter(
          (source) =>
            source.vendor === vendor && filters.sources.has(sourceKey(source)),
        ).length,
        checked: !filters.hiddenVendors.has(vendor),
      })),
    },
  ]);
  const printerGroups = $derived<ChecklistGroup[]>([
    {
      options: printers.map((printer) => ({
        value: printer,
        label: printer,
        count: sources.filter(
          (source) =>
            source.compatible_printers.includes(printer) &&
            filters.sources.has(sourceKey(source)) &&
            !filters.hiddenVendors.has(source.vendor),
        ).length,
        checked: !filters.hiddenPrinters.has(printer),
      })),
    },
  ]);
  const statusGroups = $derived<ChecklistGroup[]>([
    {
      options: MIGRATION_STATUSES.map((status) => ({
        value: status,
        label: STATUS_LABELS[status],
        count: sources.filter(
          (source) =>
            source.migration_status === status &&
            filters.sources.has(sourceKey(source)) &&
            !filters.hiddenVendors.has(source.vendor),
        ).length,
        checked: !filters.hiddenStatuses.has(status),
      })),
    },
  ]);
</script>

{#snippet scopeCheck(scopePath: PickerPath, label: string)}
  {@const selection = scopeSelection(view, scopePath)}
  <input
    type="checkbox"
    class="source-check scope-check"
    checked={selection.state === "all"}
    indeterminate={selection.state === "mixed"}
    aria-label={`Select all ${selection.ids.length} shown ${label} presets`}
    onchange={(event) => onSelect(selection.ids, event.currentTarget.checked)}
  />
{/snippet}

{#snippet meter(selected: number, count: number)}
  <span class="mat-meter" aria-hidden="true"
    ><span style:width={`${count ? Math.round((selected / count) * 100) : 0}%`}
    ></span></span
  >
{/snippet}

{#snippet presetGroup(
  title: string,
  items: CatalogSource[],
  withVendor: boolean,
)}
  {@const ids = items.map((item) => item.id)}
  <section class="vendor-group">
    <div class="vendor-head">
      <h3>{title}</h3>
      <span class="sub-count">{ids.length}</span>
      <button type="button" class="btn btn-quiet" onclick={() => toggleIds(ids)}
        >{ids.every((id) => selectedIds.has(id))
          ? "Clear"
          : "Select all"}</button
      >
    </div>
    <div class="preset-grid">
      {#each items as source (source.id)}
        <label class="source-row">
          <input
            type="checkbox"
            class="source-check"
            aria-label={`Select ${source.name}`}
            checked={selectedIds.has(source.id)}
            onchange={() => onToggle(source.id)}
          />
          <span class="source-copy">
            <span class="source-name">{presetName(source, withVendor)}</span>
            <span class="source-meta"
              >{source.source_app === "orca_slicer"
                ? "OrcaSlicer"
                : "Bambu Studio"} · {source.source_kind === "factory_system"
                ? "System preset"
                : "Your preset"}{withVendor
                ? ` · ${materialLabel(source.material_archetype)} · ${source.material_type}`
                : ""}{source.migration_status === "new"
                ? ""
                : ` · ${STATUS_LABELS[source.migration_status]}`}</span
            >
            {#each source.warnings as warning (`${source.id}:${warning}`)}<span
                class="source-warning"
                ><AlertTriangle size={12} aria-hidden="true" /> {warning}</span
              >{/each}
          </span>
          {#if source.source_app === "orca_slicer"}<span
              class="source-tag orca"
              title="OrcaSlicer source">Orca</span
            >{:else}<span class="source-tag">Bambu</span>{/if}
        </label>
      {/each}
    </div>
  </section>
{/snippet}

<div class="picker-filters">
  <ChecklistMenu
    id="source-menu"
    label="Source"
    menuLabel="Preset sources"
    value={sourceTriggerLabel(filters.sources)}
    filtered={filters.sources.size !== SOURCE_KEYS.length}
    groups={sourceGroups}
    onToggle={(value, checked) =>
      onFiltersChanged({
        sources: withValue(filters.sources, value as SourceKey, checked),
      })}
    onSetAll={(checked) =>
      onFiltersChanged({ sources: new Set(checked ? SOURCE_KEYS : []) })}
  />
  <ChecklistMenu
    id="vendor-menu"
    label="Manufacturer"
    menuLabel="Manufacturers"
    value={vendorTriggerLabel(vendors, filters.hiddenVendors)}
    filtered={vendors.some((vendor) => filters.hiddenVendors.has(vendor))}
    groups={vendorGroups}
    searchLabel="Filter manufacturers"
    emptyText="No manufacturer matches"
    bind:query={vendorQuery}
    onToggle={(value, checked) =>
      onFiltersChanged({
        hiddenVendors: withValue(filters.hiddenVendors, value, !checked),
      })}
    onSetAll={(checked, search) =>
      onFiltersChanged({
        hiddenVendors: setShownValues(
          vendors,
          filters.hiddenVendors,
          search,
          checked,
        ),
      })}
  />
  {#if printers.length}
    <ChecklistMenu
      id="printer-menu"
      label="Compatible printer"
      menuLabel="Compatible printers"
      value={checklistTriggerLabel(
        printers,
        filters.hiddenPrinters,
        "All printers",
      )}
      filtered={printers.some((printer) => filters.hiddenPrinters.has(printer))}
      groups={printerGroups}
      searchLabel="Filter printers"
      emptyText="No printer matches"
      bind:query={printerQuery}
      onToggle={(value, checked) =>
        onFiltersChanged({
          hiddenPrinters: withValue(filters.hiddenPrinters, value, !checked),
        })}
      onSetAll={(checked, search) =>
        onFiltersChanged({
          hiddenPrinters: setShownValues(
            printers,
            filters.hiddenPrinters,
            search,
            checked,
          ),
        })}
    />
  {/if}
  <ChecklistMenu
    id="status-menu"
    label="Migration status"
    menuLabel="Migration statuses"
    value={checklistTriggerLabel(
      [...MIGRATION_STATUSES],
      filters.hiddenStatuses,
      "All statuses",
      (status) => STATUS_LABELS[status as MigrationStatus],
    )}
    filtered={filters.hiddenStatuses.size > 0}
    groups={statusGroups}
    onToggle={(value, checked) =>
      onFiltersChanged({
        hiddenStatuses: withValue(
          filters.hiddenStatuses,
          value as MigrationStatus,
          !checked,
        ),
      })}
    onSetAll={(checked) =>
      onFiltersChanged({
        hiddenStatuses: new Set(checked ? [] : MIGRATION_STATUSES),
      })}
  />
  <div class="catalog-search search-wrap">
    <Search size={16} aria-hidden="true" />
    <input
      type="search"
      class="input search"
      placeholder="Search presets…"
      aria-label="Search presets in this view"
      autocomplete="off"
      value={filters.search}
      oninput={(event) =>
        onFiltersChanged({ search: event.currentTarget.value })}
    />
  </div>
</div>

<div class="picker-path">
  <nav class="crumbs" aria-label="Material path" bind:this={crumbs}>
    {#if !path[0]}
      <span aria-current="page" tabindex="-1">All materials</span>
    {:else}
      <button
        type="button"
        class="crumb-back"
        aria-label="Up one level"
        title="Up one level · mouse Back or Alt+← retraces your steps"
        onclick={() => go(pathTiered ? [path[0]] : [], "back")}
        ><ArrowLeft size={16} aria-hidden="true" /></button
      >
      <button type="button" onclick={() => go([], "back")}>All materials</button
      >
      <span class="crumb-sep" aria-hidden="true">/</span>
      {#if pathTiered}
        <button type="button" onclick={() => go([path[0]], "back")}
          >{materialLabel(path[0])}</button
        >
        <span class="crumb-sep" aria-hidden="true">/</span>
        <span aria-current="page" tabindex="-1">{path[1]}</span>
      {:else}
        <span aria-current="page" tabindex="-1"
          >{materialLabel(path[0])}{path[1] && multiType
            ? ` · ${path[1]}`
            : ""}</span
        >
      {/if}
    {/if}
  </nav>
  {#if shown.length}
    <div class="path-actions">
      <span
        ><strong>{shown.length}</strong> shown ·
        <strong>{scope.selected}</strong> selected</span
      >
      {#if level === "presets" || level === "search"}
        <button
          type="button"
          class="btn btn-quiet"
          onclick={() => onSelect(scope.ids, scope.state !== "all")}
          >{scope.state === "all"
            ? "Clear these"
            : `Select all ${scope.ids.length}`}</button
        >
      {/if}
    </div>
  {/if}
</div>
<p class="sr-only" aria-live="polite">{announcement}</p>

<div class="picker-stage" bind:this={stage} data-motion={motion}>
  {#key `${path.join("/")}|${level}`}
    {#if empty === "no-manufacturers"}
      <div class="source-empty">
        <strong>No manufacturers selected</strong>
        <span>Choose at least one manufacturer to browse presets.</span>
        <button
          type="button"
          class="btn btn-secondary"
          onclick={() => onFiltersChanged({ hiddenVendors: new Set() })}
          >Select all manufacturers</button
        >
      </div>
    {:else if empty === "no-sources"}
      <div class="source-empty">
        <strong>No sources selected</strong>
        <span>Choose OrcaSlicer, Bambu Studio, or both.</span>
        <button
          type="button"
          class="btn btn-secondary"
          onclick={() => onFiltersChanged({ sources: new Set(SOURCE_KEYS) })}
          >Select all sources</button
        >
      </div>
    {:else if empty === "no-matches"}
      <div class="source-empty">
        <strong>No presets match “{query}”</strong>
        <span>Try another product name, or search from All materials.</span>
        <button
          type="button"
          class="btn btn-secondary"
          onclick={() => onFiltersChanged({ search: "" })}>Clear search</button
        >
      </div>
    {:else if empty === "nothing-selected"}
      <div class="source-empty">
        <strong>Nothing selected here yet</strong>
        <span>Turn off “Show selected only” to browse every preset.</span>
      </div>
    {:else if empty === "no-presets"}
      <div class="source-empty">
        <strong>No presets for these filters</strong>
        <span>Widen the source or manufacturer filters.</span>
      </div>
    {:else if level === "search"}
      <div class="preset-groups">
        {#each searchGroups(view) as group (group.key)}
          {@render presetGroup(group.title, group.items, true)}
        {/each}
      </div>
    {:else if level === "materials"}
      <div class="mat-grid">
        {#each materialCards(view) as card (card.id)}
          <article class="mat-card" class:has-selection={card.selected > 0}>
            <div class="mat-head">
              <div class="mat-title">
                <h3 class="mat-name">{card.label}</h3>
                {#if card.selected}<span class="sel-pill"
                    >{card.selected} selected</span
                  >{/if}
              </div>
              {@render scopeCheck([card.id], card.label)}
            </div>
            <p class="mat-meta">
              <strong>{card.count}</strong>
              {card.count === 1 ? "preset" : "presets"} · {plural(
                card.manufacturers,
                "manufacturer",
              )}
            </p>
            {@render meter(card.selected, card.count)}
            {#if card.layout === "tiered"}
              <p class="mat-types">{card.types.join(" · ")}</p>
              <button
                type="button"
                class="mat-open"
                onclick={() => go([card.id], "forward")}
                ><span>Browse {card.types.length} types</span><span
                  class="mat-arrow"
                  aria-hidden="true">→</span
                ></button
              >
            {:else if card.layout === "single"}
              <p class="mat-types">{card.manufacturerNames.join(", ")}</p>
              <button
                type="button"
                class="mat-open"
                onclick={() => go([card.id, card.types[0]], "forward")}
                ><span>Browse {plural(card.count, "preset")}</span><span
                  class="mat-arrow"
                  aria-hidden="true">→</span
                ></button
              >
            {:else}
              <ul class="mat-subrows" aria-label={`${card.label} types`}>
                {#each card.typeRows as row (row.type)}
                  <li>
                    <button
                      type="button"
                      onclick={() => go([card.id, row.type], "forward")}
                      ><span class="sub-name">{row.type}</span
                      >{#if row.selected}<span class="sel-pill"
                          >{row.selected}</span
                        >{/if}<span class="sub-count">{row.count}</span><span
                        class="mat-arrow"
                        aria-hidden="true">→</span
                      ></button
                    >
                  </li>
                {/each}
              </ul>
            {/if}
          </article>
        {/each}
      </div>
    {:else if level === "types"}
      <div class="mat-grid">
        {#each typeCards(view) as card (card.type)}
          {@const names = card.manufacturerNames}
          <article
            class="mat-card is-type"
            class:has-selection={card.selected > 0}
          >
            <div class="mat-head">
              <div class="mat-title">
                <h3 class="mat-name">{card.type}</h3>
                {#if card.selected}<span class="sel-pill"
                    >{card.selected} selected</span
                  >{/if}
              </div>
              {@render scopeCheck(
                [path[0], card.type],
                `${materialLabel(path[0])} ${card.type}`,
              )}
            </div>
            <p class="mat-meta">
              <strong>{card.count}</strong>
              {card.count === 1 ? "preset" : "presets"} · {plural(
                names.length,
                "manufacturer",
              )}
            </p>
            {@render meter(card.selected, card.count)}
            <p class="mat-types">
              {names.slice(0, 3).join(", ")}{names.length > 3
                ? ` +${names.length - 3}`
                : ""}
            </p>
            <button
              type="button"
              class="mat-open"
              onclick={() => go([path[0], card.type], "forward")}
              ><span>Browse presets</span><span
                class="mat-arrow"
                aria-hidden="true">→</span
              ></button
            >
          </article>
        {/each}
      </div>
    {:else}
      <div class="preset-groups">
        {#each groupPresets(view) as group (group.key)}
          {@render presetGroup(group.title, group.items, false)}
        {/each}
      </div>
    {/if}
  {/key}
</div>

<div class="picker-foot">
  <p class="sel-count">
    <strong>{selectedIds.size}</strong>
    {selectedIds.size === 1 ? "preset" : "presets"} selected{#if hidden}<span
        class="selection-detail">{` · ${hidden} hidden by filters`}</span
      >{/if}
  </p>
  <label class="switch-row"
    ><input
      type="checkbox"
      class="switch"
      checked={filters.selectedOnly}
      onchange={(event) =>
        onFiltersChanged({ selectedOnly: event.currentTarget.checked })}
    />Show selected only</label
  >
  <button
    type="button"
    class="btn btn-quiet"
    disabled={selectedIds.size === 0}
    onclick={async () => {
      onClearSelection();
      await tick();
      crumbs?.querySelector<HTMLElement>("[aria-current]")?.focus();
    }}>Clear selection</button
  >
  <span class="foot-spacer"></span>
  <SavedFilters {savedPresets} {onSavePreset} {onLoadPreset} {onDeletePreset} />
  <CatalogMenu
    {discovery}
    presetCount={sources.length}
    {busy}
    {onRefresh}
    {onAddSource}
    {onRemoveSource}
  />
</div>
