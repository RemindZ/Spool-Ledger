<script lang="ts">
  import { AlertOctagon, Check, CirclePlus, MinusCircle } from '@lucide/svelte';
  import type { MigrationPlan, PlanAction } from '../types';

  let { plan }: { plan: MigrationPlan } = $props();

  const actionLabels: Record<PlanAction, string> = {
    create: 'Create',
    add_target: 'Add target',
    update: 'Update',
    rename: 'Rename',
    replace: 'Replace',
    skip: 'Skip',
    block: 'Blocked',
  };

  function iconFor(action: PlanAction) {
    if (action === 'block') return AlertOctagon;
    if (action === 'skip') return MinusCircle;
    if (action === 'create' || action === 'add_target') return CirclePlus;
    return Check;
  }
</script>

<section class="plan-panel" aria-labelledby="plan-heading">
  <header class="plan-header">
    <div>
      <p class="section-kicker">Frozen preview</p>
      <h2 id="plan-heading">Migration plan</h2>
    </div>
    <div class="plan-metrics">
      <span><strong>{plan.operations.length}</strong> operations</span>
      <span class:danger={plan.operations.some((item) => item.action === 'block')}
        ><strong>{plan.operations.filter((item) => item.action === 'block').length}</strong> blocked</span
      >
    </div>
  </header>
  <div class="table-scroll">
    <table>
      <thead>
        <tr>
          <th>Source</th>
          <th>Slicing preset</th>
          <th>AMS identity</th>
          <th>Target</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {#each plan.operations as operation (operation.id)}
          {@const ActionIcon = iconFor(operation.action)}
          <tr class:blocked={operation.action === 'block'}>
            <td><strong>{operation.source_name}</strong></td>
            <td>{operation.preset_name}</td>
            <td>
              <span>{operation.ams_name}</span>
              <code>{operation.filament_id}</code>
            </td>
            <td>{operation.printer_preset_name}</td>
            <td>
              <span class="action-pill" data-action={operation.action}
                ><ActionIcon size={13} /> {actionLabels[operation.action]}</span
              >
              {#if operation.conflict}<small class="conflict-copy">{operation.conflict.message}</small>{/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
