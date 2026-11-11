<script lang="ts" module>
  export interface ChecklistOption {
    value: string;
    label: string;
    count: number;
    checked: boolean;
  }

  export interface ChecklistGroup {
    title?: string;
    options: ChecklistOption[];
  }
</script>

<script lang="ts">
  import { Search } from "@lucide/svelte";
  import Popover from "./Popover.svelte";

  let {
    id,
    label,
    value,
    menuLabel,
    groups,
    filtered = false,
    searchLabel = "",
    emptyText = "No option matches",
    query = $bindable(""),
    onToggle,
    onSetAll,
  }: {
    id: string;
    label: string;
    value: string;
    menuLabel: string;
    groups: ChecklistGroup[];
    filtered?: boolean;
    /** Shows the filter box when set, for example "Filter manufacturers". */
    searchLabel?: string;
    emptyText?: string;
    query?: string;
    onToggle: (value: string, checked: boolean) => void;
    /** Select all or Clear all, limited to the options the search shows. */
    onSetAll: (checked: boolean, query: string) => void;
  } = $props();

  let search = $state<HTMLInputElement>();
  const needle = $derived(query.trim().toLocaleLowerCase());
  const shownGroups = $derived(
    groups
      .map((group) => ({
        ...group,
        options: group.options.filter((option) =>
          option.label.toLocaleLowerCase().includes(needle),
        ),
      }))
      .filter((group) => group.options.length > 0),
  );
</script>

<Popover
  class={`filter-menu ${id}`}
  bodyClass="checklist-menu"
  bodyLabel={menuLabel}
  {filtered}
  onOpen={() => setTimeout(() => search?.focus(), 0)}
>
  {#snippet summary()}
    <span class="filter-label">{label}</span>
    <span class="filter-value">{value}</span>
  {/snippet}
  {#if searchLabel}
    <div class="menu-search search-wrap">
      <Search size={16} aria-hidden="true" />
      <input
        bind:this={search}
        bind:value={query}
        type="search"
        class="input search"
        placeholder={`${searchLabel}…`}
        aria-label={searchLabel}
        autocomplete="off"
      />
    </div>
  {/if}
  <div class="menu-actions">
    <button
      type="button"
      class="btn btn-quiet"
      onclick={() => onSetAll(true, query)}>Select all</button
    >
    <button
      type="button"
      class="btn btn-quiet"
      onclick={() => onSetAll(false, query)}>Clear all</button
    >
  </div>
  <div class="menu-options">
    {#each shownGroups as group, index (group.title ?? index)}
      {#if group.title}<div class="menu-group">{group.title}</div>{/if}
      {#each group.options as option (option.value)}
        <label class="menu-option">
          <input
            type="checkbox"
            class="menu-check"
            checked={option.checked}
            onchange={(event) =>
              onToggle(option.value, event.currentTarget.checked)}
          />
          <span>{option.label}</span>
          <span class="count">{option.count}</span>
        </label>
      {/each}
    {:else}
      <p class="menu-empty">{emptyText} “{query.trim()}”</p>
    {/each}
  </div>
</Popover>
