export type TemplateScope = "slicing" | "ams";

export interface TemplateTokenDefinition {
  value: string;
  label: string;
  scope: "all" | "slicing";
}

export interface ParsedTemplate {
  tokens: string[];
  texts: string[];
}

export const TEMPLATE_TOKENS: readonly TemplateTokenDefinition[] = [
  { value: "{source_name}", label: "Source name", scope: "all" },
  { value: "{clean_name}", label: "Clean name", scope: "all" },
  { value: "{vendor}", label: "Vendor", scope: "all" },
  { value: "{material}", label: "Material", scope: "all" },
  { value: "{family}", label: "Family", scope: "all" },
  { value: "{variant}", label: "Variant", scope: "all" },
  { value: "{source_app}", label: "Source application", scope: "all" },
  { value: "{source_kind}", label: "Source kind", scope: "all" },
  { value: "{printer}", label: "Printer", scope: "slicing" },
  { value: "{printer_code}", label: "Printer code", scope: "slicing" },
  { value: "{nozzle}", label: "Nozzle", scope: "slicing" },
  { value: "@", label: "At separator", scope: "slicing" },
];

const TOKEN_VALUES = TEMPLATE_TOKENS.map((token) => token.value);
const TOKEN_PATTERN = new RegExp(
  TOKEN_VALUES.map((value) =>
    value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"),
  ).join("|"),
  "g",
);

export function parseTemplate(value: string): ParsedTemplate {
  const tokens: string[] = [];
  const texts: string[] = [];
  let cursor = 0;

  for (const match of value.matchAll(TOKEN_PATTERN)) {
    texts.push(value.slice(cursor, match.index));
    tokens.push(match[0]);
    cursor = (match.index ?? cursor) + match[0].length;
  }
  texts.push(value.slice(cursor));

  return { tokens, texts };
}

export function serializeTemplate(template: ParsedTemplate): string {
  return template.tokens.reduce(
    (value, token, index) => value + token + template.texts[index + 1],
    template.texts[0] ?? "",
  );
}

export function insertToken(
  template: ParsedTemplate,
  token: string,
  index: number,
): ParsedTemplate {
  const tokens = [...template.tokens];
  const texts = [...template.texts];
  const position = Math.max(0, Math.min(index, tokens.length));
  tokens.splice(position, 0, token);
  texts.splice(position + 1, 0, "");
  return { tokens, texts };
}

export function moveToken(
  template: ParsedTemplate,
  from: number,
  to: number,
): ParsedTemplate {
  if (
    from < 0 ||
    from >= template.tokens.length ||
    to < 0 ||
    to >= template.tokens.length ||
    from === to
  ) {
    return { tokens: [...template.tokens], texts: [...template.texts] };
  }

  const tokens = [...template.tokens];
  const [token] = tokens.splice(from, 1);
  tokens.splice(to, 0, token);
  return { tokens, texts: [...template.texts] };
}

export function removeToken(
  template: ParsedTemplate,
  index: number,
): ParsedTemplate {
  if (index < 0 || index >= template.tokens.length) {
    return { tokens: [...template.tokens], texts: [...template.texts] };
  }

  const tokens = [...template.tokens];
  const texts = [...template.texts];
  tokens.splice(index, 1);
  texts.splice(index, 2, `${texts[index] ?? ""}${texts[index + 1] ?? ""}`);
  return { tokens, texts };
}

export function scopeViolations(value: string, scope: TemplateScope): string[] {
  if (scope === "slicing") return [];
  const parsed = parseTemplate(value);
  const slicingOnly = new Set(
    TEMPLATE_TOKENS.filter((token) => token.scope === "slicing").map(
      (token) => token.value,
    ),
  );
  return [...new Set(parsed.tokens.filter((token) => slicingOnly.has(token)))];
}
