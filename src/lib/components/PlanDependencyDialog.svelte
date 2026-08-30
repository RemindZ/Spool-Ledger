<script lang="ts">
  import { tick } from "svelte";
  import { AlertTriangle } from "@lucide/svelte";
  import type { TargetTemplateDecision, TargetTemplateIssue } from "../types";

  let {
    open,
    issues,
    busy = false,
    onCancel,
    onResolve,
    onRemove,
  }: {
    open: boolean;
    issues: TargetTemplateIssue[];
    busy?: boolean;
    onCancel: () => void;
    onResolve: (decisions: TargetTemplateDecision[]) => void;
    onRemove: (sourceIds: string[]) => void;
  } = $props();

  const componentId = $props.id();
  const titleId = `${componentId}-title`;
  let dialog = $state<HTMLDialogElement>();
  let cancelButton = $state<HTMLButtonElement>();
  let choices = $state<Record<string, string>>({});
  let returnFocus: HTMLElement | null = null;

  const affectedSourceIds = $derived([
    ...new Set(
      issues.flatMap((issue) =>
        issue.affected_sources.map((source) => source.id),
      ),
    ),
  ]);
  const decisions = $derived(
    issues.flatMap((issue): TargetTemplateDecision[] => {
      const choice = choices[issue.id];
      if (choice?.startsWith("installed:")) {
        return [
          {
            issue_id: issue.id,
            action: "use_installed",
            profile_name: choice.slice("installed:".length),
            source_id: null,
          },
        ];
      }
      if (choice?.startsWith("source:")) {
        return [
          {
            issue_id: issue.id,
            action: "use_source",
            profile_name: null,
            source_id: choice.slice("source:".length),
          },
        ];
      }
      return [];
    }),
  );

  $effect(() => {
    if (!open) return;
    choices = {};
    returnFocus = document.activeElement as HTMLElement | null;
    void tick().then(() => {
      if (!dialog) return;
      if (!dialog.open) dialog.showModal();
      cancelButton?.focus();
    });
  });

  async function restoreFocus() {
    await tick();
    returnFocus?.focus();
    returnFocus = null;
  }

  function close() {
    if (dialog?.open) dialog.close();
    onCancel();
    void restoreFocus();
  }

  function resolve() {
    if (decisions.length !== issues.length) return;
    onResolve(decisions);
  }

  function removeAffected() {
    onRemove(affectedSourceIds);
  }

  function handleCancel(event: Event) {
    event.preventDefault();
    close();
  }

  function handleBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget) close();
  }
</script>

{#if open}
  <dialog
    bind:this={dialog}
    class="modal-dialog dependency-dialog"
    aria-labelledby={titleId}
    oncancel={handleCancel}
    onclick={handleBackdrop}
  >
    <div class="modal-dialog-surface">
      <header>
        <span class="dependency-alert" aria-hidden="true">
          <AlertTriangle size={19} />
        </span>
        <div>
          <p class="section-kicker">Plan dependency</p>
          <h2 id={titleId}>Target profile required</h2>
        </div>
      </header>

      <div class="modal-dialog-content dependency-content">
        <p>
          The selected filaments need target profiles that are not available
          under their expected Generic names. Choose a verified compatible
          profile or remove every affected filament.
        </p>

        {#each issues as issue (issue.id)}
          <fieldset class="dependency-issue">
            <legend>{issue.expected_name}</legend>
            <p class="dependency-context">
              <strong>{issue.material}</strong> for {issue.printer_name}, {issue.nozzle}
              mm
            </p>
            <p>{issue.diagnostic}</p>
            <div class="affected-list">
              <strong>Affected filaments</strong>
              <ul>
                {#each issue.affected_sources as source (source.id)}
                  <li>{source.name}</li>
                {/each}
              </ul>
            </div>

            {#if issue.installed_candidates.length || issue.source_candidates.length}
              <div class="dependency-options">
                {#each issue.installed_candidates as candidate (candidate.profile_name)}
                  <label>
                    <input
                      type="radio"
                      name={`${componentId}-${issue.id}`}
                      value={`installed:${candidate.profile_name}`}
                      bind:group={choices[issue.id]}
                    />
                    <span>
                      Use {candidate.profile_name} for {issue.printer_name}
                      {#if candidate.recommended}<small
                          >Recommended installed profile</small
                        >{/if}
                    </span>
                  </label>
                {/each}
                {#each issue.source_candidates as candidate (candidate.source_id)}
                  <label>
                    <input
                      type="radio"
                      name={`${componentId}-${issue.id}`}
                      value={`source:${candidate.source_id}`}
                      bind:group={choices[issue.id]}
                    />
                    <span>
                      Include and migrate {candidate.name}
                      <small>Validated Bambu user profile</small>
                    </span>
                  </label>
                {/each}
              </div>
            {:else}
              <p class="dependency-unavailable">
                No compatible installed or selected Bambu user profile is
                available.
              </p>
            {/if}
          </fieldset>
        {/each}

        <p class="dependency-safety">
          The app will not fabricate a target profile from an unverified source.
        </p>
      </div>

      <footer>
        <button
          bind:this={cancelButton}
          type="button"
          class="secondary-button"
          disabled={busy}
          onclick={close}>Cancel</button
        >
        <button
          type="button"
          class="secondary-button"
          disabled={busy || affectedSourceIds.length === 0}
          onclick={removeAffected}
        >
          Remove {affectedSourceIds.length} affected {affectedSourceIds.length ===
          1
            ? "filament"
            : "filaments"}
        </button>
        <button
          type="button"
          class="primary-button"
          disabled={busy || decisions.length !== issues.length}
          onclick={resolve}
        >
          {busy ? "Resolving…" : "Resolve dependencies"}
        </button>
      </footer>
    </div>
  </dialog>
{/if}

<style>
  .dependency-dialog {
    width: min(46rem, calc(100vw - 2rem));
  }

  .dependency-dialog .modal-dialog-surface {
    width: 100%;
    max-height: min(44rem, calc(100vh - 2rem));
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  header :global(.section-kicker),
  header h2 {
    margin: 0;
  }

  .dependency-alert {
    display: grid;
    width: 2.25rem;
    height: 2.25rem;
    place-items: center;
    border-radius: 50%;
    color: var(--warning-strong);
    background: var(--warning-soft);
  }

  .dependency-content {
    display: grid;
    gap: 1rem;
    overflow-y: auto;
    overscroll-behavior: contain;
  }

  .dependency-content > p,
  .dependency-issue p {
    margin: 0;
  }

  .dependency-issue {
    display: grid;
    gap: 0.75rem;
    min-width: 0;
    margin: 0;
    padding: 1rem;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-md);
  }

  .dependency-issue legend {
    padding: 0 0.35rem;
    font-weight: 760;
    overflow-wrap: anywhere;
  }

  .dependency-context,
  .dependency-issue > p,
  .affected-list,
  .dependency-safety {
    color: var(--text-muted);
  }

  .affected-list {
    display: grid;
    gap: 0.3rem;
  }

  .affected-list ul {
    display: grid;
    gap: 0.2rem;
    margin: 0;
    padding-left: 1.2rem;
  }

  .dependency-options {
    display: grid;
    gap: 0.5rem;
  }

  .dependency-options label {
    display: flex;
    align-items: flex-start;
    gap: 0.65rem;
    padding: 0.7rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .dependency-options label:has(input:checked) {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .dependency-options input {
    margin-top: 0.15rem;
  }

  .dependency-options span {
    display: grid;
    gap: 0.15rem;
  }

  .dependency-options small {
    color: var(--text-muted);
  }

  .dependency-unavailable {
    padding: 0.65rem;
    border-radius: var(--radius-sm);
    color: var(--warning-strong) !important;
    background: var(--warning-soft);
  }

  footer {
    flex-wrap: wrap;
  }

  @media (max-width: 34rem) {
    .dependency-dialog {
      width: calc(100vw - 1rem);
    }

    .dependency-dialog .modal-dialog-surface {
      max-height: calc(100vh - 1rem);
    }

    footer > button {
      flex: 1 1 100%;
    }
  }
</style>
