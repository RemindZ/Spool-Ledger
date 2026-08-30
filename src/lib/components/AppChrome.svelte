<script lang="ts">
  import { FolderPlus, RefreshCw, Trash2 } from "@lucide/svelte";
  import type { Theme } from "../state";
  import type { DiscoveryResponse, SourceApp, SourceKind } from "../types";
  import WorkspaceTabs, { type WorkspaceView } from "./WorkspaceTabs.svelte";

  let {
    discovery,
    selectedAccountId,
    theme,
    busy,
    activeView = "setup",
    planCount = 0,
    onRefresh,
    onAddSource = () => {},
    onRemoveSource = () => {},
    onAccountChanged,
    onThemeChanged,
    onViewChanged = () => {},
  }: {
    discovery: DiscoveryResponse;
    selectedAccountId: string | null;
    theme: Theme;
    busy: boolean;
    activeView?: WorkspaceView;
    planCount?: number;
    onRefresh: () => void;
    onAddSource?: (sourceApp: SourceApp, sourceKind: SourceKind) => void;
    onRemoveSource?: (id: string) => void;
    onAccountChanged: (accountId: string) => void;
    onThemeChanged: (theme: Theme) => void;
    onViewChanged?: (view: WorkspaceView) => void;
  } = $props();

  let sourceApp = $state<SourceApp>("orca_slicer");
  let sourceKind = $state<SourceKind>("factory_system");

  const platformLabel = $derived(
    `${discovery.platform === "windows" ? "Windows" : discovery.platform === "macos" ? "macOS" : discovery.platform === "linux" ? "Linux" : discovery.platform} build`,
  );
</script>

<header class="application-chrome">
  <div class="titlebar" data-tauri-drag-region>
    <div class="brand-lockup">
      <img
        src="/spool-ledger-lockup-transparent.png"
        alt="Spool Ledger · Bambu Filament Migrator"
        draggable="false"
      />
    </div>
    <div class="environment" aria-label="Detected applications">
      {#if discovery.orca_slicer_detected}<span><i></i>OrcaSlicer detected</span
        >{/if}
      {#if discovery.bambu_studio_detected}<span
          ><i></i>Bambu Studio detected</span
        >{/if}
    </div>
    <span class="platform-label">{platformLabel}</span>
  </div>

  <div class="application-toolbar">
    <label class="account-control">
      <span>Destination account</span>
      <select
        aria-label="Destination account"
        value={selectedAccountId ?? ""}
        onchange={(event) => onAccountChanged(event.currentTarget.value)}
      >
        {#each discovery.accounts as account (account.id)}
          <option
            value={account.id}
            disabled={account.eligibility !== "eligible"}
          >
            {account.id} · {account.filament_profile_count} profiles{account.eligibility ===
            "eligible"
              ? ""
              : ` · ${account.eligibility}`}
          </option>
        {/each}
      </select>
    </label>

    <p class="local-safety">
      <strong>Local only</strong><span
        >Nothing is written until you review and commit the plan.</span
      >
    </p>

    <details class="manual-source-control">
      <summary><FolderPlus size={14} /> Source folders</summary>
      <div class="manual-source-popover">
        <div class="manual-source-fields">
          <label>
            <span>Application</span>
            <select aria-label="Source application" bind:value={sourceApp}>
              <option value="orca_slicer">OrcaSlicer</option>
              <option value="bambu_studio">Bambu Studio</option>
            </select>
          </label>
          <label>
            <span>Profile kind</span>
            <select aria-label="Source kind" bind:value={sourceKind}>
              <option value="factory_system">Factory / system</option>
              <option value="user_custom">User / custom</option>
            </select>
          </label>
        </div>
        <button
          class="secondary-button"
          type="button"
          disabled={busy}
          onclick={() => onAddSource(sourceApp, sourceKind)}
          ><FolderPlus size={14} /> Choose source folder</button
        >
        {#if discovery.manual_source_roots.length}
          <ul class="manual-source-list">
            {#each discovery.manual_source_roots as root (root.id)}
              <li class:invalid={!root.active}>
                <span
                  ><strong
                    >{root.source_app === "orca_slicer"
                      ? "OrcaSlicer"
                      : "Bambu Studio"} · {root.source_kind === "factory_system"
                      ? "Factory"
                      : "User"}</strong
                  ><small title={root.path}>{root.path}</small
                  >{#if root.error}<small class="field-error"
                      >{root.error}</small
                    >{/if}</span
                >
                <button
                  type="button"
                  aria-label={`Remove manual source ${root.path}`}
                  onclick={() => onRemoveSource(root.id)}
                  ><Trash2 size={13} /></button
                >
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </details>

    <div class="theme-switch" role="group" aria-label="Theme">
      {#each ["system", "light", "dark"] as choice (choice)}
        <button
          type="button"
          aria-pressed={theme === choice}
          onclick={() => onThemeChanged(choice as Theme)}
          >{choice[0].toUpperCase() + choice.slice(1)}</button
        >
      {/each}
    </div>

    <button
      class="icon-button"
      type="button"
      aria-label="Refresh profile discovery"
      disabled={busy}
      onclick={onRefresh}
    >
      <RefreshCw size={16} class={busy ? "spin" : ""} />
    </button>
  </div>

  <WorkspaceTabs active={activeView} {planCount} onChange={onViewChanged} />
</header>
