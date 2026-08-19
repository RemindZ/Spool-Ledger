<script lang="ts">
  import {
    ArrowDown,
    ArrowUp,
    Braces,
    ChevronRight,
    Plus,
    Save,
    Trash2,
    WandSparkles,
  } from "@lucide/svelte";
  import type {
    NamePreview,
    NamingPreset,
    NamingRule,
    RuleConditionField,
    RulePatternKind,
  } from "../types";

  let {
    presetTemplate,
    amsTemplate,
    presetRules = [],
    amsRules = [],
    savedPresets = [],
    preview,
    error = null,
    onTemplatesChanged,
    onRulesChanged = () => {},
    onSavePreset = () => {},
    onLoadPreset = () => {},
    onDeletePreset = () => {},
  }: {
    presetTemplate: string;
    amsTemplate: string;
    presetRules?: NamingRule[];
    amsRules?: NamingRule[];
    savedPresets?: NamingPreset[];
    preview: NamePreview | null;
    error?: string | null;
    onTemplatesChanged: (preset: string, ams: string) => void;
    onRulesChanged?: (preset: NamingRule[], ams: NamingRule[]) => void;
    onSavePreset?: (name: string) => void;
    onLoadPreset?: (id: string) => void;
    onDeletePreset?: (id: string) => void;
  } = $props();

  let output = $state<"preset" | "ams">("ams");
  let kind = $state<RulePatternKind>("wildcard");
  let pattern = $state("");
  let replacement = $state("");
  let caseSensitive = $state(false);
  let conditionField = $state<RuleConditionField | "">("");
  let conditionValue = $state("");
  let presetName = $state("");
  let draftError = $state<string | null>(null);
  let ruleSequence = 0;

  const conditionFields: Array<{ value: RuleConditionField; label: string }> = [
    { value: "source_app", label: "Source app" },
    { value: "source_kind", label: "Source kind" },
    { value: "vendor", label: "Manufacturer" },
    { value: "material", label: "Material" },
    { value: "family", label: "Family" },
  ];

  function updatePreset(event: Event) {
    onTemplatesChanged(
      (event.currentTarget as HTMLInputElement).value,
      amsTemplate,
    );
  }

  function updateAms(event: Event) {
    onTemplatesChanged(
      presetTemplate,
      (event.currentTarget as HTMLInputElement).value,
    );
  }

  function addRule() {
    if (!pattern.trim()) {
      draftError = "Rule pattern is required.";
      return;
    }
    if (conditionField && !conditionValue.trim()) {
      draftError = "A condition value is required when a field is selected.";
      return;
    }
    draftError = null;
    const rule: NamingRule = {
      id: `rule-${++ruleSequence}`,
      kind,
      pattern,
      replacement,
      case_sensitive: caseSensitive,
      condition: conditionField
        ? { field: conditionField, value: conditionValue.trim() }
        : null,
    };
    if (output === "preset") onRulesChanged([...presetRules, rule], amsRules);
    else onRulesChanged(presetRules, [...amsRules, rule]);
    pattern = "";
    replacement = "";
  }

  function removeRule(ruleOutput: "preset" | "ams", id: string) {
    onRulesChanged(
      ruleOutput === "preset"
        ? presetRules.filter((rule) => rule.id !== id)
        : presetRules,
      ruleOutput === "ams"
        ? amsRules.filter((rule) => rule.id !== id)
        : amsRules,
    );
  }

  function moveRule(ruleOutput: "preset" | "ams", index: number, direction: -1 | 1) {
    const rules = [...(ruleOutput === "preset" ? presetRules : amsRules)];
    const destination = index + direction;
    if (destination < 0 || destination >= rules.length) return;
    [rules[index], rules[destination]] = [rules[destination], rules[index]];
    onRulesChanged(
      ruleOutput === "preset" ? rules : presetRules,
      ruleOutput === "ams" ? rules : amsRules,
    );
  }

  function savePreset() {
    const name = presetName.trim();
    if (!name) {
      draftError = "Naming preset name is required.";
      return;
    }
    draftError = null;
    onSavePreset(name);
    presetName = "";
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
      <input
        aria-label="Bambu slicing preset template"
        value={presetTemplate}
        oninput={updatePreset}
        spellcheck="false"
      />
    </label>
    <label>
      <span>AMS identity template</span>
      <input
        aria-label="AMS identity template"
        value={amsTemplate}
        oninput={updateAms}
        spellcheck="false"
      />
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
        {#if preview && preview.preset_before !== preview.preset_name}
          <small class="preview-before">{preview.preset_before}</small>
        {/if}
        <dd>{preview?.preset_name ?? "Select a source and target to preview"}</dd>
      </div>
      <div>
        <dt>AMS identity</dt>
        {#if preview && preview.ams_before !== preview.ams_name}
          <small class="preview-before">{preview.ams_before}</small>
        {/if}
        <dd>{preview?.ams_name ?? "Select a source and target to preview"}</dd>
      </div>
    </dl>
  </div>

  {#if error}<p class="field-error" role="alert">{error}</p>{/if}

  <details class="advanced-rules">
    <summary>Advanced naming rules</summary>
    <div class="rule-editor">
      <div class="rule-grid">
        <label><span>Apply to</span><select aria-label="Rule output" bind:value={output}><option value="ams">AMS identity</option><option value="preset">Slicing preset</option></select></label>
        <label><span>Rule type</span><select aria-label="Rule type" bind:value={kind}><option value="wildcard">Wildcard</option><option value="regex">Regular expression</option></select></label>
        <label class="rule-wide"><span>Rule pattern</span><input aria-label="Rule pattern" bind:value={pattern} placeholder={kind === "wildcard" ? "Polymaker PLA *" : "(?i)^Polymaker PLA (.*)$"} spellcheck="false" /></label>
        <label class="rule-wide"><span>Replacement</span><input aria-label="Replacement" bind:value={replacement} placeholder="$1" spellcheck="false" /></label>
        <label><span>Condition</span><select aria-label="Condition field" bind:value={conditionField}><option value="">Apply to all</option>{#each conditionFields as field (field.value)}<option value={field.value}>{field.label}</option>{/each}</select></label>
        <label><span>Matches</span><input aria-label="Condition value" bind:value={conditionValue} disabled={!conditionField} placeholder="Panchroma" /></label>
      </div>
      <label class="case-toggle"><input type="checkbox" bind:checked={caseSensitive} /><span>Case-sensitive matching</span></label>
      {#if draftError}<p class="field-error" role="alert">{draftError}</p>{/if}
      <button class="secondary-button" type="button" onclick={addRule}><Plus size={14} /> Add rule</button>

      {#each [["preset", "Slicing preset rules", presetRules], ["ams", "AMS identity rules", amsRules]] as group (group[0])}
        {@const ruleOutput = group[0] as "preset" | "ams"}
        {@const rules = group[2] as NamingRule[]}
        {#if rules.length}
          <section class="rule-list" aria-label={group[1] as string}>
            <h3>{group[1]} <span>{rules.length}</span></h3>
            <ol>
              {#each rules as rule, index (rule.id)}
                <li>
                  <span class="rule-index">{index + 1}</span>
                  <span class="rule-description"><strong>{rule.kind === "wildcard" ? "Wildcard" : "Regex"}: {rule.pattern}</strong><small>Replace with {rule.replacement || "nothing"}{rule.condition ? ` when ${rule.condition.field} is ${rule.condition.value}` : " on all matching names"}</small></span>
                  <span class="rule-actions"><button type="button" aria-label={`Move rule ${index + 1} up`} disabled={index === 0} onclick={() => moveRule(ruleOutput, index, -1)}><ArrowUp size={13} /></button><button type="button" aria-label={`Move rule ${index + 1} down`} disabled={index === rules.length - 1} onclick={() => moveRule(ruleOutput, index, 1)}><ArrowDown size={13} /></button><button type="button" aria-label={`Delete rule ${index + 1}`} onclick={() => removeRule(ruleOutput, rule.id)}><Trash2 size={13} /></button></span>
                </li>
              {/each}
            </ol>
          </section>
        {/if}
      {/each}

      <div class="preset-controls">
        <label><span>Naming preset name</span><input aria-label="Naming preset name" bind:value={presetName} placeholder="Panchroma clean" /></label>
        <button class="secondary-button" type="button" aria-label="Save naming preset" onclick={savePreset}><Save size={14} /> Save</button>
        {#if savedPresets.length}
          <label class="saved-preset"><span>Saved presets</span><select aria-label="Saved naming presets" onchange={(event) => event.currentTarget.value && onLoadPreset(event.currentTarget.value)}><option value="">Choose preset</option>{#each savedPresets as saved (saved.id)}<option value={saved.id}>{saved.name}</option>{/each}</select></label>
          <div class="saved-preset-list">{#each savedPresets as saved (saved.id)}<button type="button" aria-label={`Delete naming preset ${saved.name}`} onclick={() => onDeletePreset(saved.id)}><Trash2 size={12} /> {saved.name}</button>{/each}</div>
        {/if}
      </div>
    </div>
  </details>
</section>
