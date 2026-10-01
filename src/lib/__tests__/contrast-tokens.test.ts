import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const css = readFileSync(`${root}/src/app.css`, "utf8");

type Rgb = [number, number, number];

function block(selector: string): Map<string, string> {
  const start = css.indexOf(`${selector} {`);
  if (start < 0) throw new Error(`missing theme block: ${selector}`);
  const body = css.slice(start, css.indexOf("}", start));
  return new Map(
    [...body.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)].map((match) => [
      match[1],
      match[2].trim(),
    ]),
  );
}

const light = block(":root");
const themes = {
  light,
  "dark (system)": new Map([
    ...light,
    ...block(':root:not([data-theme="light"])'),
  ]),
  "dark (explicit)": new Map([...light, ...block(':root[data-theme="dark"]')]),
};

function resolve(tokens: Map<string, string>, name: string): string {
  const value = tokens.get(name);
  if (!value) throw new Error(`undefined token ${name}`);
  const reference = value.match(/^var\((--[a-z0-9-]+)\)$/);
  return reference ? resolve(tokens, reference[1]) : value;
}

function oklchToRgb(l: number, c: number, h: number): Rgb {
  const radians = (h * Math.PI) / 180;
  const a = c * Math.cos(radians);
  const b = c * Math.sin(radians);
  const lp = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const mp = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const sp = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * lp - 3.3077115913 * mp + 0.2309699292 * sp,
    -1.2684380046 * lp + 2.6097574011 * mp - 0.3413193965 * sp,
    -0.0041960863 * lp - 0.7034186147 * mp + 1.707614701 * sp,
  ].map((channel) => {
    const clamped = Math.min(1, Math.max(0, channel));
    const encoded =
      clamped <= 0.0031308
        ? 12.92 * clamped
        : 1.055 * clamped ** (1 / 2.4) - 0.055;
    return Math.round(encoded * 255);
  }) as Rgb;
}

function rgb(color: string): Rgb {
  if (color.startsWith("#")) {
    return [1, 3, 5].map((index) =>
      parseInt(color.slice(index, index + 2), 16),
    ) as Rgb;
  }
  const oklch = color.match(/^oklch\(([\d.]+)%\s+([\d.]+)\s+([\d.]+)\)$/);
  if (!oklch) throw new Error(`unsupported color ${color}`);
  return oklchToRgb(Number(oklch[1]) / 100, Number(oklch[2]), Number(oklch[3]));
}

function luminance(color: Rgb): number {
  const [r, g, b] = color.map((channel) => {
    const value = channel / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(foreground: string, background: string): number {
  const [high, low] = [luminance(rgb(foreground)), luminance(rgb(background))]
    .sort((a, b) => b - a)
    .map((value) => value + 0.05);
  return high / low;
}

const panelSurfaces = [
  "--surface",
  "--surface-raised",
  "--surface-muted",
  "--surface-strong",
  "--bambu-soft",
  "--warning-soft",
  "--danger-soft",
  "--orca-soft",
];

const requirements: Array<[string, string[], number]> = [
  ["--text-soft", panelSurfaces, 4.5],
  ["--text-faint", panelSurfaces, 4.5],
  ["--orca-ink", panelSurfaces, 4.5],
  ["--on-action", ["--action", "--action-hover"], 4.5],
  ["--on-danger", ["--danger"], 4.5],
  ["--control-border", ["--surface", "--surface-raised", "--surface-muted"], 3],
  [
    "--focus-ring",
    ["--surface", "--surface-raised", "--surface-muted", "--bambu-soft"],
    3,
  ],
];

describe("theme contrast", () => {
  for (const [theme, tokens] of Object.entries(themes)) {
    it(`meets WCAG AA for text, controls, and focus in the ${theme} theme`, () => {
      const failures = requirements.flatMap(([foreground, backgrounds, min]) =>
        backgrounds
          .map((background) => ({
            pair: `${foreground} on ${background}`,
            ratio: contrast(
              resolve(tokens, foreground),
              resolve(tokens, background),
            ),
          }))
          .filter(({ ratio }) => ratio < min)
          .map(({ pair, ratio }) => `${pair}: ${ratio.toFixed(2)} < ${min}`),
      );
      expect(failures).toEqual([]);
    });
  }

  it("routes action, danger, orange text, controls, and focus through the contrast tokens", () => {
    expect(css).toMatch(
      /\.primary-button\s*\{[^}]*background:\s*var\(--action\);[^}]*color:\s*var\(--on-action\);/s,
    );
    expect(css).toMatch(
      /\.danger-button\s*\{[^}]*color:\s*var\(--on-danger\);/s,
    );
    for (const selector of [
      String.raw`\.provenance\[data-app="orca_slicer"\] span:first-child`,
      String.raw`\.support-button:hover`,
    ]) {
      expect(css).toMatch(
        new RegExp(
          `${selector}\\s*\\{[^}]*color:\\s*var\\(--orca-ink\\);`,
          "s",
        ),
      );
    }
    expect(css).toMatch(
      /button:focus-visible,\s*input:focus-visible,\s*select:focus-visible,\s*summary:focus-visible\s*\{[^}]*outline:\s*2px solid var\(--focus-ring\);/s,
    );
    expect(css).toMatch(
      /\.source-profile-check\s*\{[^}]*border:\s*1px solid var\(--control-border\);/s,
    );
    expect(css).toMatch(
      /select,\s*\.template-fields input,\s*\.search-field\s*\{[^}]*border:\s*1px solid var\(--control-border\);/s,
    );
  });
});
