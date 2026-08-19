<script lang="ts">
  import { Database, RefreshCw, SlidersHorizontal } from '@lucide/svelte';
  import type { Theme } from '../state';
  import type { DiscoveryResponse } from '../types';

  let {
    discovery,
    selectedAccountId,
    theme,
    busy,
    onRefresh,
    onAccountChanged,
    onThemeChanged,
  }: {
    discovery: DiscoveryResponse;
    selectedAccountId: string | null;
    theme: Theme;
    busy: boolean;
    onRefresh: () => void;
    onAccountChanged: (accountId: string) => void;
    onThemeChanged: (theme: Theme) => void;
  } = $props();
</script>

<header class="discovery-bar">
  <div class="brand-lockup">
    <span class="brand-spool" aria-hidden="true"><i></i></span>
    <div>
      <p class="brand-kicker">Bambu × Orca</p>
      <h1>Bambu Filament Migrator</h1>
    </div>
  </div>

  <div class="discovery-stats" aria-label="Discovered profile locations">
    <span><Database size={14} /><strong>{discovery.source_root_ids.length} source roots</strong></span>
    <span><SlidersHorizontal size={14} /><strong>{discovery.target_catalog_ids.length} target catalogs</strong></span>
  </div>

  <div class="discovery-controls">
    <label>
      <span>Bambu account</span>
      <select
        aria-label="Bambu account"
        value={selectedAccountId ?? ''}
        onchange={(event) => onAccountChanged(event.currentTarget.value)}
      >
        {#each discovery.accounts as account (account.id)}
          <option value={account.id} disabled={account.eligibility !== 'eligible'}>
            {account.id} · {account.filament_profile_count} profiles{account.eligibility === 'eligible'
              ? ''
              : ` · ${account.eligibility}`}
          </option>
        {/each}
      </select>
    </label>
    <label>
      <span>Theme</span>
      <select
        aria-label="Theme"
        value={theme}
        onchange={(event) => onThemeChanged(event.currentTarget.value as Theme)}
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>
    <button class="icon-button" type="button" aria-label="Refresh profile discovery" disabled={busy} onclick={onRefresh}>
      <RefreshCw size={16} class={busy ? 'spin' : ''} />
    </button>
  </div>
</header>
