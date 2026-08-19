<script lang="ts">
  import { Braces, ChevronRight, WandSparkles } from '@lucide/svelte';

  let {
    presetTemplate,
    amsTemplate,
    preview,
    error = null,
    onTemplatesChanged,
  }: {
    presetTemplate: string;
    amsTemplate: string;
    preview: { preset_name: string; ams_name: string } | null;
    error?: string | null;
    onTemplatesChanged: (preset: string, ams: string) => void;
  } = $props();

  function updatePreset(event: Event) {
    onTemplatesChanged((event.currentTarget as HTMLInputElement).value, amsTemplate);
  }

  function updateAms(event: Event) {
    onTemplatesChanged(presetTemplate, (event.currentTarget as HTMLInputElement).value);
  }
</script>

<section class="panel naming-panel" aria-labelledby="naming-heading">
  <header class="panel-heading">
    <div>
      <p class="section-kicker">Output identity</p>
      <h2 id="naming-heading">Naming</h2>
    </div>
    <Braces size={18} />
  </header>

  <div class="template-fields">
    <label>
      <span>Bambu slicing preset template</span>
      <input value={presetTemplate} oninput={updatePreset} spellcheck="false" />
    </label>
    <label>
      <span>AMS identity template</span>
      <input value={amsTemplate} oninput={updateAms} spellcheck="false" />
    </label>
  </div>

  <div class="name-preview" aria-live="polite">
    <div class="preview-route">
      <span class="route-node source"><WandSparkles size={14} /> Source</span>
      <ChevronRight size={14} />
      <span class="route-node output">Output</span>
    </div>
    <dl>
      <div>
        <dt>Slicing preset</dt>
        <dd>{preview?.preset_name ?? 'Select a source and target to preview'}</dd>
      </div>
      <div>
        <dt>AMS identity</dt>
        <dd>{preview?.ams_name ?? 'Select a source and target to preview'}</dd>
      </div>
    </dl>
  </div>

  {#if error}<p class="field-error" role="alert">{error}</p>{/if}

  <details class="advanced-rules">
    <summary>Advanced naming rules</summary>
    <div class="advanced-copy">
      <p>Ordered wildcard and regular-expression rules can use capture groups and source conditions.</p>
      <p class="coming-note">Rules are validated by the backend before a plan can be built.</p>
    </div>
  </details>
</section>
