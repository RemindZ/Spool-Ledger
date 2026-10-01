import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const css = readFileSync(`${root}/src/app.css`, "utf8");

describe("desktop layout containment", () => {
  it("allows long migration plan IDs to wrap inside modal dialogs", () => {
    expect(css).toMatch(
      /\.modal-dialog-surface\s*>\s*\*\s*\{[^}]*min-width:\s*0;/s,
    );
    expect(css).toMatch(
      /\.modal-dialog-surface\s+h2\s*\{[^}]*overflow-wrap:\s*anywhere;/s,
    );
  });

  it("sizes the printer summary artwork column to the artwork tile", () => {
    expect(css).toMatch(
      /\.printer-card summary\s*\{[^}]*grid-template-columns:\s*auto\s+minmax\(0,\s*1fr\)\s+auto;/s,
    );
  });

  it("wraps execution phase names instead of truncating them", () => {
    const label = css.match(
      /\.execution-phases li > span:nth-child\(2\)\s*\{([^}]*)\}/,
    );
    expect(label?.[1]).toBeDefined();
    expect(label?.[1]).not.toMatch(/white-space:\s*nowrap|text-overflow/);
  });

  it("sizes the restore description like the completed-run description", () => {
    expect(css).toMatch(
      /\.restore-panel \.panel-heading p:last-child\s*\{[^}]*color:\s*var\(--text-soft\);[^}]*font-size:\s*0\.66rem;/s,
    );
  });

  it("scrolls the migration plan table inside its panel on narrow windows", () => {
    expect(css).toMatch(/\.plan-panel\s*\{[^}]*min-width:\s*0;/s);
    expect(css).toMatch(/\.plan-panel table\s*\{[^}]*min-width:\s*720px;/s);
  });

  it("keeps template tokens together and lets only the trailing gap grow", () => {
    expect(css).toMatch(
      /\.template-composer \.template-text-gap\s*\{[^}]*flex:\s*0 1 auto;/s,
    );
    expect(css).toMatch(
      /\.template-composer \.template-text-gap:last-of-type\s*\{[^}]*flex:\s*1 1 3ch;/s,
    );
  });

  it("keeps the setup workspace within the available desktop viewport", () => {
    expect(css).toMatch(
      /\.workspace-shell:has\(\.setup-view\)\s*\{[^}]*height:\s*calc\(100dvh\s*-\s*96px\);[^}]*overflow:\s*hidden;/s,
    );
    expect(css).toMatch(
      /\.setup-view\s*\{[^}]*grid-template-rows:\s*minmax\(0,\s*1fr\)\s+auto;/s,
    );
    expect(css).toMatch(
      /\.setup-view\s+\.(?:naming|destination)-rail[\s\S]*overflow-y:\s*auto;/,
    );
  });
});
