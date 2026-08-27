<script lang="ts">
  import TemplateComposer from "../../components/TemplateComposer.svelte";
  import type { TemplateScope } from "../../template-composer";

  let {
    initialValue = $bindable(),
    scope = "slicing",
  }: { initialValue: string; scope?: TemplateScope } = $props();
  let active = $state(false);
</script>

<TemplateComposer
  label="Template"
  value={initialValue}
  {scope}
  {active}
  onActivate={() => (active = true)}
  onChange={(next) => (initialValue = next)}
/>
<output aria-label="Serialized template">{initialValue}</output>
