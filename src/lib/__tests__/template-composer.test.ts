import { describe, expect, it } from "vitest";
import {
  TEMPLATE_TOKENS,
  insertToken,
  moveToken,
  parseTemplate,
  removeToken,
  scopeViolations,
  serializeTemplate,
} from "../template-composer";

describe("template composer model", () => {
  it("round-trips every backend placeholder and arbitrary text", () => {
    const value =
      "Prefix {source_name} {clean_name} {vendor} {material} {family} {variant} {source_app} {source_kind} @{printer} {printer_code} {nozzle} suffix";

    expect(serializeTemplate(parseTemplate(value))).toBe(value);
    expect(
      TEMPLATE_TOKENS.filter((token) => token.value !== "@").map(
        (token) => token.value,
      ),
    ).toEqual(
      expect.arrayContaining([
        "{source_name}",
        "{clean_name}",
        "{vendor}",
        "{material}",
        "{family}",
        "{variant}",
        "{source_app}",
        "{source_kind}",
        "{printer}",
        "{printer_code}",
        "{nozzle}",
      ]),
    );
  });

  it("preserves unknown braces so the backend can report the exact error", () => {
    const value = "{vendor} {unknown_token} tail{";
    expect(serializeTemplate(parseTemplate(value))).toBe(value);
  });

  it("inserts, reorders, and removes tokens without losing text gaps", () => {
    const parsed = parseTemplate("A{vendor}B{material}C");
    expect(serializeTemplate(insertToken(parsed, "{family}", 1))).toBe(
      "A{vendor}B{family}{material}C",
    );
    expect(serializeTemplate(moveToken(parsed, 0, 1))).toBe(
      "A{material}B{vendor}C",
    );
    expect(serializeTemplate(removeToken(parsed, 0))).toBe("AB{material}C");
  });

  it("reports slicing-only tokens in AMS templates without rewriting them", () => {
    const value = "{vendor} @{printer_code} {nozzle}";
    expect(scopeViolations(value, "ams")).toEqual([
      "@",
      "{printer_code}",
      "{nozzle}",
    ]);
    expect(scopeViolations(value, "slicing")).toEqual([]);
    expect(serializeTemplate(parseTemplate(value))).toBe(value);
  });
});
