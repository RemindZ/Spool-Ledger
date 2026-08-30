// @vitest-environment jsdom
import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import TemplateComposerHarness from "./fixtures/TemplateComposerHarness.svelte";

afterEach(cleanup);

function transfer() {
  const values = new Map<string, string>();
  return {
    effectAllowed: "all",
    dropEffect: "move",
    setData(type: string, value: string) {
      values.set(type, value);
    },
    getData(type: string) {
      return values.get(type) ?? "";
    },
  };
}

describe("TemplateComposer", () => {
  it("edits arbitrary text gaps without changing adjacent tokens", async () => {
    render(TemplateComposerHarness, {
      props: { initialValue: "Before {vendor} after" },
    });

    await fireEvent.input(screen.getByLabelText("Template text 1"), {
      target: { value: "Prefix " },
    });

    expect(screen.getByLabelText("Serialized template")).toHaveTextContent(
      "Prefix {vendor} after",
    );
  });

  it("reorders tokens with Left and Right while retaining token focus", async () => {
    render(TemplateComposerHarness, {
      props: { initialValue: "{vendor} {material}" },
    });
    const vendor = screen.getByRole("button", {
      name: "Template token Vendor",
    });
    vendor.focus();

    await fireEvent.keyDown(vendor, { key: "ArrowRight" });

    expect(screen.getByLabelText("Serialized template")).toHaveTextContent(
      "{material} {vendor}",
    );
    expect(
      screen.getByRole("button", { name: "Template token Vendor" }),
    ).toHaveFocus();
  });

  it("removes a token with Delete and moves focus to a surviving control", async () => {
    render(TemplateComposerHarness, {
      props: { initialValue: "A{vendor}B{material}C" },
    });
    const vendor = screen.getByRole("button", {
      name: "Template token Vendor",
    });
    vendor.focus();

    await fireEvent.keyDown(vendor, { key: "Delete" });

    expect(screen.getByLabelText("Serialized template")).toHaveTextContent(
      "AB{material}C",
    );
    expect(document.activeElement).not.toBe(document.body);
  });

  it("inserts a palette token dropped on the composer", async () => {
    render(TemplateComposerHarness, {
      props: { initialValue: "{vendor}" },
    });
    const dataTransfer = transfer();
    dataTransfer.setData("application/x-bfm-template-token", "{family}");

    await fireEvent.drop(
      screen.getByRole("group", { name: "Template composer" }),
      {
        dataTransfer,
      },
    );

    expect(screen.getByLabelText("Serialized template")).toHaveTextContent(
      "{vendor}{family}",
    );
  });
});
