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

  it("lays out enabled printers as a card grid", () => {
    expect(css).toMatch(
      /\.printer-grid\s*\{[^}]*grid-template-columns:\s*repeat\(auto-fill,\s*minmax\(280px,\s*1fr\)\);/s,
    );
    expect(css).toContain(".printer-grid .printer:has(.nozzle-check:checked)");
  });

  it("wraps run step names instead of truncating them", () => {
    const label = css.match(/\.phase-name\s*\{([^}]*)\}/);
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

  it("keeps the step bench within the available desktop viewport", () => {
    expect(css).toMatch(
      /\.route-shell\s*\{[^}]*height:\s*clamp\(540px,\s*calc\(100dvh - 262px\),\s*940px\);[^}]*overflow:\s*hidden;/s,
    );
    expect(css).toMatch(
      /\.route-shell > \.step-rail\.is-open > \.rail-body\s*\{[^}]*overflow:\s*auto;/s,
    );
    expect(css).toMatch(/\.picker-stage\s*\{[^}]*overflow:\s*auto;/s);
    expect(css).toMatch(
      /\.route-shell > \.step-rail\.is-open > \.rail-head,\s*\.route-shell > \.step-rail\.is-open > \.rail-body\s*\{[^}]*min-width:\s*600px;/s,
    );
  });

  it("animates the accordion and drilldown with the specified timings", () => {
    expect(css).toMatch(
      /\.route-shell > \.step-rail\s*\{[^}]*flex:\s*0 0 84px;[^}]*transition:\s*flex-grow 0\.44s var\(--ease-standard\);/s,
    );
    expect(css).toMatch(
      /animation:\s*rail-reveal 0\.36s 0\.1s var\(--ease-standard\) both;/,
    );
    expect(css).toMatch(
      /animation:\s*tier-in 0\.26s var\(--ease-standard\) both;/,
    );
    expect(css).toMatch(
      /transition:\s*background-color 0\.32s var\(--ease-standard\);/,
    );
    expect(css).toMatch(
      /--ease-standard:\s*cubic-bezier\(0\.2,\s*0,\s*0,\s*1\);/,
    );
  });

  it("stacks the accordion into horizontal 68px bars below 900px", () => {
    const narrow = css.slice(css.lastIndexOf("@media (max-width: 900px)"));
    expect(narrow).toMatch(/\.route-shell\s*\{[^}]*flex-direction:\s*column;/s);
    expect(narrow).toMatch(
      /\.rail-spine\s*\{[^}]*min-height:\s*68px;[^}]*flex-direction:\s*row;/s,
    );
  });
});
