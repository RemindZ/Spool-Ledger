<script lang="ts">
  import { tick } from "svelte";
  import { GripVertical, X } from "@lucide/svelte";
  import {
    TEMPLATE_TOKENS,
    insertToken,
    moveToken,
    parseTemplate,
    removeToken,
    scopeViolations,
    serializeTemplate,
    type ParsedTemplate,
    type TemplateScope,
  } from "../template-composer";

  let {
    label,
    value,
    scope,
    active = false,
    onActivate,
    onChange,
  }: {
    label: string;
    value: string;
    scope: TemplateScope;
    active?: boolean;
    onActivate: () => void;
    onChange: (value: string) => void;
  } = $props();

  const parsed = $derived(parseTemplate(value));
  const violations = $derived(scopeViolations(value, scope));
  let composer: HTMLDivElement;

  function emit(next: ParsedTemplate) {
    onActivate();
    onChange(serializeTemplate(next));
  }

  function changeText(index: number, nextText: string) {
    const texts = [...parsed.texts];
    texts[index] = nextText;
    emit({ tokens: [...parsed.tokens], texts });
  }

  async function focusToken(index: number) {
    await tick();
    composer
      ?.querySelector<HTMLButtonElement>(`[data-token-index="${index}"]`)
      ?.focus();
  }

  async function reorder(index: number, offset: -1 | 1) {
    const target = index + offset;
    if (target < 0 || target >= parsed.tokens.length) return;
    emit(moveToken(parsed, index, target));
    await focusToken(target);
  }

  async function remove(index: number) {
    const remaining = parsed.tokens.length - 1;
    emit(removeToken(parsed, index));
    await tick();
    if (remaining > 0) {
      const nextIndex = Math.min(index, remaining - 1);
      composer
        ?.querySelector<HTMLButtonElement>(`[data-token-index="${nextIndex}"]`)
        ?.focus();
    } else {
      composer
        ?.querySelector<HTMLInputElement>(`[data-text-index="${index}"]`)
        ?.focus();
    }
  }

  function handleTokenKeydown(event: KeyboardEvent, index: number) {
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      void reorder(index, event.key === "ArrowLeft" ? -1 : 1);
    } else if (event.key === "Delete" || event.key === "Backspace") {
      event.preventDefault();
      void remove(index);
    }
  }

  function beginDrag(event: DragEvent, index: number) {
    if (!event.dataTransfer) return;
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData(
      "application/x-bfm-template-index",
      String(index),
    );
  }

  function allowDrop(event: DragEvent) {
    event.preventDefault();
  }

  function droppedToken(event: DragEvent, index = parsed.tokens.length) {
    event.preventDefault();
    const transfer = event.dataTransfer;
    if (!transfer) return;

    const sourceIndex = Number.parseInt(
      transfer.getData("application/x-bfm-template-index"),
      10,
    );
    if (Number.isInteger(sourceIndex)) {
      const target = Math.max(0, Math.min(index, parsed.tokens.length - 1));
      emit(moveToken(parsed, sourceIndex, target));
      void focusToken(target);
      return;
    }

    const token = transfer.getData("application/x-bfm-template-token");
    const definition = TEMPLATE_TOKENS.find((item) => item.value === token);
    if (!definition || (scope === "ams" && definition.scope === "slicing")) {
      return;
    }
    emit(insertToken(parsed, token, index));
    void focusToken(index);
  }
</script>

<div class="template-composer-field">
  <span class="field-label">{label}</span>
  <div
    bind:this={composer}
    class:active
    class:invalid={violations.length > 0}
    class="template-composer"
    role="group"
    aria-label={`${label} composer`}
    ondragover={allowDrop}
    ondrop={droppedToken}
    onfocusin={onActivate}
  >
    {#each parsed.tokens as token, index (`${index}:${token}`)}
      <input
        class="template-text-gap"
        data-text-index={index}
        aria-label={`${label} text ${index + 1}`}
        value={parsed.texts[index]}
        size={Math.max(1, parsed.texts[index].length)}
        oninput={(event) => changeText(index, event.currentTarget.value)}
      />
      <span
        class:scope-invalid={violations.includes(token)}
        class="template-token-wrap"
        role="group"
        aria-label={`${token} token controls`}
        ondragover={allowDrop}
        ondrop={(event) => droppedToken(event, index)}
      >
        <button
          type="button"
          class="template-token"
          data-token-index={index}
          draggable="true"
          aria-label={`${label} token ${TEMPLATE_TOKENS.find((item) => item.value === token)?.label ?? token}`}
          title="Drag or use Left and Right to reorder. Delete removes this token."
          ondragstart={(event) => beginDrag(event, index)}
          onkeydown={(event) => handleTokenKeydown(event, index)}
          onclick={onActivate}
        >
          <GripVertical size={12} aria-hidden="true" />
          <span>{token}</span>
        </button>
        <button
          type="button"
          class="remove-template-token"
          aria-label={`Remove ${token} from ${label}`}
          onclick={() => remove(index)}
        >
          <X size={11} aria-hidden="true" />
        </button>
      </span>
    {/each}
    <input
      class="template-text-gap"
      data-text-index={parsed.tokens.length}
      aria-label={`${label} text ${parsed.tokens.length + 1}`}
      value={parsed.texts[parsed.tokens.length]}
      size={Math.max(1, parsed.texts[parsed.tokens.length].length)}
      oninput={(event) =>
        changeText(parsed.tokens.length, event.currentTarget.value)}
    />
  </div>
  {#if violations.length > 0}
    <p class="field-error" role="alert">
      Remove slicing-only {violations.join(", ")} from this AMS template.
    </p>
  {/if}
</div>
