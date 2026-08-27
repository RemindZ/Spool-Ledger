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
  import TemplateComposer from "./TemplateComposer.svelte";
  import ModalDialog from "./ModalDialog.svelte";
  import {
    TEMPLATE_TOKENS,
    insertToken,
    parseTemplate,
    serializeTemplate,
  } from "../template-composer";
  import type {
    NamePreview,
    NamingPreset,
    NamingRule,
    OutputSelection,
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
    outputs = { slicing_presets: true, custom_filaments: true },
    onTemplatesChanged,
    onRulesChanged = () => {},
    onOutputsChanged = () => {},
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
    outputs?: OutputSelection;
    onTemplatesChanged: (preset: string, ams: string) => void;
    onRulesChanged?: (preset: NamingRule[], ams: NamingRule[]) => void;
    onOutputsChanged?: (outputs: OutputSelection) => void;
    onSavePreset?: (name: string) => void;
    onLoadPreset?: (id: string) => void;
    onDeletePreset?: (id: string) => void;
  } = $props();

  let activeTemplate = $state<"preset" | "ams">("preset");
  let helpOpen = $state(false);
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

  function updateTemplate(template: "preset" | "ams", value: string) {
    onTemplatesChanged(
      template === "preset" ? value : presetTemplate,
      template === "ams" ? value : amsTemplate,
    );
  }

  function insertVariable(token: string) {
    const value = activeTemplate === "preset" ? presetTemplate : amsTemplate;
    const parsed = parseTemplate(value);
    updateTemplate(
      activeTemplate,
      serializeTemplate(insertToken(parsed, token, parsed.tokens.length)),
    );
  }

  function beginVariableDrag(event: DragEvent, token: string) {
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "copy";
    event.dataTransfer.setData("application/x-bfm-template-token", token);
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

  function moveRule(
    ruleOutput: "preset" | "ams",
    index: number,
    direction: -1 | 1,
  ) {
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
    <div class="panel-heading-actions">
      <button
        type="button"
        class="icon-button subtle help-button"
        aria-label="Naming help"
        onclick={() => (helpOpen = true)}
      >
        <span class="help-glyph" aria-hidden="true">?</span>
      </button>
      <Braces size={18} aria-hidden="true" />
    </div>
  </header>

  <div class="template-fields">
    <TemplateComposer
      label="Bambu slicing preset template"
      value={presetTemplate}
      scope="slicing"
      active={activeTemplate === "preset"}
      onActivate={() => (activeTemplate = "preset")}
      onChange={(value) => updateTemplate("preset", value)}
    />
    <TemplateComposer
      label="AMS custom filament template"
      value={amsTemplate}
      scope="ams"
      active={activeTemplate === "ams"}
      onActivate={() => (activeTemplate = "ams")}
      onChange={(value) => updateTemplate("ams", value)}
    />
  </div>

  <div
    class="template-variable-picker"
    role="group"
    aria-label="Template variables"
  >
    <span class="field-label">Variables</span>
    <div class="template-variable-list">
      {#each TEMPLATE_TOKENS as token (token.value)}
        <button
          type="button"
          class="template-variable"
          draggable={!(activeTemplate === "ams" && token.scope === "slicing")}
          disabled={activeTemplate === "ams" && token.scope === "slicing"}
          aria-label={`Insert ${token.label} into active template`}
          title={activeTemplate === "ams" && token.scope === "slicing"
            ? `${token.label} is available only for slicing preset names.`
            : `Insert ${token.value}`}
          onclick={() => insertVariable(token.value)}
          ondragstart={(event) => beginVariableDrag(event, token.value)}
        >
          {token.value}
        </button>
      {/each}
    </div>
    <small>
      Insert into the active template. Drag variables onto a composer or use its
      token controls to reorder.
    </small>
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
        <dd>
          {preview?.preset_name ?? "Select a source and target to preview"}
        </dd>
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
        <label
          ><span>Apply to</span><select
            aria-label="Rule output"
            bind:value={output}
            ><option value="ams">AMS identity</option><option value="preset"
              >Slicing preset</option
            ></select
          ></label
        >
        <label
          ><span>Rule type</span><select
            aria-label="Rule type"
            bind:value={kind}
            ><option value="wildcard">Wildcard</option><option value="regex"
              >Regular expression</option
            ></select
          ></label
        >
        <label class="rule-wide"
          ><span>Rule pattern</span><input
            aria-label="Rule pattern"
            bind:value={pattern}
            placeholder={kind === "wildcard"
              ? "Polymaker PLA *"
              : "(?i)^Polymaker PLA (.*)$"}
            spellcheck="false"
          /></label
        >
        <label class="rule-wide"
          ><span>Replacement</span><input
            aria-label="Replacement"
            bind:value={replacement}
            placeholder="$1"
            spellcheck="false"
          /></label
        >
        <label
          ><span>Condition</span><select
            aria-label="Condition field"
            bind:value={conditionField}
            ><option value="">Apply to all</option
            >{#each conditionFields as field (field.value)}<option
                value={field.value}>{field.label}</option
              >{/each}</select
          ></label
        >
        <label
          ><span>Matches</span><input
            aria-label="Condition value"
            bind:value={conditionValue}
            disabled={!conditionField}
            placeholder="Panchroma"
          /></label
        >
      </div>
      <label class="case-toggle"
        ><input type="checkbox" bind:checked={caseSensitive} /><span
          >Case-sensitive matching</span
        ></label
      >
      {#if draftError}<p class="field-error" role="alert">{draftError}</p>{/if}
      <button class="secondary-button" type="button" onclick={addRule}
        ><Plus size={14} /> Add rule</button
      >

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
                  <span class="rule-description"
                    ><strong
                      >{rule.kind === "wildcard" ? "Wildcard" : "Regex"}: {rule.pattern}</strong
                    ><small
                      >Replace with {rule.replacement ||
                        "nothing"}{rule.condition
                        ? ` when ${rule.condition.field} is ${rule.condition.value}`
                        : " on all matching names"}</small
                    ></span
                  >
                  <span class="rule-actions"
                    ><button
                      type="button"
                      aria-label={`Move rule ${index + 1} up`}
                      disabled={index === 0}
                      onclick={() => moveRule(ruleOutput, index, -1)}
                      ><ArrowUp size={13} /></button
                    ><button
                      type="button"
                      aria-label={`Move rule ${index + 1} down`}
                      disabled={index === rules.length - 1}
                      onclick={() => moveRule(ruleOutput, index, 1)}
                      ><ArrowDown size={13} /></button
                    ><button
                      type="button"
                      aria-label={`Delete rule ${index + 1}`}
                      onclick={() => removeRule(ruleOutput, rule.id)}
                      ><Trash2 size={13} /></button
                    ></span
                  >
                </li>
              {/each}
            </ol>
          </section>
        {/if}
      {/each}

      <div class="preset-controls">
        <label
          ><span>Naming preset name</span><input
            aria-label="Naming preset name"
            bind:value={presetName}
            placeholder="Panchroma clean"
          /></label
        >
        <button
          class="secondary-button"
          type="button"
          aria-label="Save naming preset"
          onclick={savePreset}><Save size={14} /> Save</button
        >
        {#if savedPresets.length}
          <label class="saved-preset"
            ><span>Saved presets</span><select
              aria-label="Saved naming presets"
              onchange={(event) =>
                event.currentTarget.value &&
                onLoadPreset(event.currentTarget.value)}
              ><option value="">Choose preset</option
              >{#each savedPresets as saved (saved.id)}<option value={saved.id}
                  >{saved.name}</option
                >{/each}</select
            ></label
          >
          <div class="saved-preset-list">
            {#each savedPresets as saved (saved.id)}<button
                type="button"
                aria-label={`Delete naming preset ${saved.name}`}
                onclick={() => onDeletePreset(saved.id)}
                ><Trash2 size={12} /> {saved.name}</button
              >{/each}
          </div>
        {/if}
      </div>
    </div>
  </details>

  <details class="advanced-outputs">
    <summary>Advanced outputs</summary>
    <div class="output-controls">
      <label>
        <input
          aria-label="Create Bambu slicing presets"
          type="checkbox"
          checked={outputs.slicing_presets}
          disabled={outputs.slicing_presets && !outputs.custom_filaments}
          onchange={(event) =>
            onOutputsChanged({
              ...outputs,
              slicing_presets: event.currentTarget.checked,
            })}
        />
        <span
          ><strong>Create Bambu slicing presets</strong><small
            >Normal user presets for the selected printer family</small
          ></span
        >
      </label>
      <label>
        <input
          aria-label="Create AMS custom filaments"
          type="checkbox"
          checked={outputs.custom_filaments}
          disabled={outputs.custom_filaments && !outputs.slicing_presets}
          onchange={(event) =>
            onOutputsChanged({
              ...outputs,
              custom_filaments: event.currentTarget.checked,
            })}
        />
        <span
          ><strong>Create AMS custom filaments</strong><small
            >Flattened profile JSON and paired synchronization sidecar</small
          ></span
        >
      </label>
    </div>
  </details>

  <ModalDialog
    open={helpOpen}
    title="Naming templates and rules"
    cancelLabel="Close"
    onClose={() => (helpOpen = false)}
  >
    <p>
      Templates combine literal text with the listed profile variables. Use the
      separate slicing and AMS composers because printer and nozzle variables
      apply only to slicing preset names.
    </p>
    <p>
      Advanced rules run in order after template expansion. Wildcards and
      regular expressions can be limited by source application, profile kind,
      manufacturer, material, or family.
    </p>
    <p>
      Preview names always come from the migration engine. The interface does
      not invent a local fallback preview.
    </p>
  </ModalDialog>
</section>
