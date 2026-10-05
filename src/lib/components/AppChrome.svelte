<script lang="ts">
  import { ShieldCheck } from "@lucide/svelte";
  import type { Theme } from "../theme";
  import type { DiscoveryResponse } from "../types";
  import ThemeToggle from "./ThemeToggle.svelte";
  import WorkspaceTabs, { type WorkspaceView } from "./WorkspaceTabs.svelte";

  let {
    discovery,
    selectedAccountId,
    theme,
    activeView = "setup",
    planCount = 0,
    onAccountChanged,
    onThemeChanged,
    onViewChanged = () => {},
  }: {
    discovery: DiscoveryResponse;
    selectedAccountId: string | null;
    theme: Theme;
    activeView?: WorkspaceView;
    planCount?: number;
    onAccountChanged: (accountId: string) => void;
    onThemeChanged: (theme: Theme) => void;
    onViewChanged?: (view: WorkspaceView) => void;
  } = $props();
</script>

<header class="application-chrome">
  <div class="titlebar" data-tauri-drag-region>
    <div class="brand">
      <img
        class="brand-lockup"
        src="/spool-ledger-lockup-transparent.png"
        alt="Spool Ledger · Bambu Filament Migrator"
        width="680"
        height="155"
        draggable="false"
      />
    </div>
    <div class="environment" aria-label="Migration route">
      <span
        class="route-source"
        title={discovery.orca_slicer_detected
          ? "OrcaSlicer detected"
          : "OrcaSlicer not detected"}
        >OrcaSlicer<span class="sr-only"
          >{` ${discovery.orca_slicer_detected ? "detected" : "not detected"}`}</span
        ></span
      >
      <span aria-hidden="true">→</span>
      <span
        title={discovery.bambu_studio_detected
          ? "Bambu Studio detected"
          : "Bambu Studio not detected"}
        >Bambu Studio<span class="sr-only"
          >{` ${discovery.bambu_studio_detected ? "detected" : "not detected"}`}</span
        ></span
      >
    </div>
    <ThemeToggle {theme} onChange={onThemeChanged} />
  </div>

  <div class="toolbar">
    <WorkspaceTabs active={activeView} {planCount} onChange={onViewChanged} />

    <label class="account">
      <span>Destination</span>
      <select
        aria-label="Destination account"
        value={selectedAccountId ?? ""}
        onchange={(event) => onAccountChanged(event.currentTarget.value)}
      >
        {#if selectedAccountId === null}
          <option value="" disabled>
            {discovery.accounts.some(
              (account) => account.eligibility === "eligible",
            )
              ? "Select an account"
              : "No eligible account"}
          </option>
        {/if}
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

    <span class="safety"><ShieldCheck size={16} /> Local only</span>
  </div>
</header>
