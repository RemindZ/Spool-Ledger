<script lang="ts">
  import { CheckCircle2, CircleDashed, Cloud, HardDrive, ShieldCheck } from '@lucide/svelte';
  import type { MigrationPlan } from '../types';
  import type { OperationProgress } from '../state';

  let {
    plan,
    progress,
    running,
    cancellable = false,
    onCancel = () => {},
    onVerify = () => {},
  }: {
    plan: MigrationPlan;
    progress: Record<string, OperationProgress>;
    running: boolean;
    cancellable?: boolean;
    onCancel?: () => void;
    onVerify?: (operationIds: string[]) => void;
  } = $props();

  const evidenceLabels = {
    created_local: 'Created locally',
    loaded_by_bambu: 'Loaded by Bambu',
    cloud_id_assigned: 'Cloud ID assigned',
    ams_verified: 'AMS verified',
  } as const;

  const verifiedIds = $derived(
    plan.operations
      .filter((operation) => progress[operation.id]?.evidence === 'cloud_id_assigned')
      .map((operation) => operation.id),
  );
</script>

<section class="run-progress panel" aria-labelledby="progress-heading">
  <header class="panel-heading">
    <div>
      <p class="section-kicker">Evidence ledger</p>
      <h2 id="progress-heading">Migration progress</h2>
    </div>
    {#if running}<span class="live-indicator"><i></i> Running</span>{/if}
  </header>

  <ol class="evidence-steps">
    <li class:active={plan.operations.some((item) => progress[item.id])}>
      <HardDrive size={17} /><span><strong>Local</strong><small>Files committed and parsed</small></span>
    </li>
    <li class:active={plan.operations.some((item) => ['loaded_by_bambu', 'cloud_id_assigned', 'ams_verified'].includes(progress[item.id]?.evidence))}>
      <CheckCircle2 size={17} /><span><strong>Bambu</strong><small>Profile acknowledged locally</small></span>
    </li>
    <li class:active={plan.operations.some((item) => ['cloud_id_assigned', 'ams_verified'].includes(progress[item.id]?.evidence))}>
      <Cloud size={17} /><span><strong>Cloud</strong><small>Unique PFUS ID observed</small></span>
    </li>
    <li class:active={plan.operations.some((item) => progress[item.id]?.evidence === 'ams_verified')}>
      <ShieldCheck size={17} /><span><strong>AMS</strong><small>Operator-confirmed only</small></span>
    </li>
  </ol>

  <div class="operation-progress-list">
    {#each plan.operations as operation (operation.id)}
      {@const item = progress[operation.id]}
      <div class="operation-progress-row">
        {#if item}<CheckCircle2 size={15} />{:else}<CircleDashed size={15} />{/if}
        <span><strong>{operation.ams_name}</strong><small>{operation.printer_preset_name}</small></span>
        <span class="evidence-label">{item ? evidenceLabels[item.evidence] : 'Waiting'}</span>
        {#if item?.evidence === 'cloud_id_assigned'}
          <span class="pending-label">AMS check pending</span>
        {/if}
      </div>
    {/each}
  </div>

  <footer class="panel-actions">
    {#if running && cancellable}
      <button class="secondary-button" type="button" onclick={onCancel}>Cancel monitoring</button>
    {:else if !running && verifiedIds.length}
      <button class="secondary-button" type="button" onclick={() => onVerify(verifiedIds)}
        >Record AMS verification</button
      >
    {/if}
  </footer>
</section>
