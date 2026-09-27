import { describe, expect, it } from "vitest";
import { MAX_PANEL_HEIGHT, fittedHeight, panelShortcut } from "./panel-keys";

describe("panelShortcut", () => {
  it("maps Esc, ⌘Q and ⌘R", () => {
    expect(panelShortcut({ key: "Escape", metaKey: false })).toBe("close");
    expect(panelShortcut({ key: "q", metaKey: true })).toBe("quit");
    expect(panelShortcut({ key: "R", metaKey: true })).toBe("refresh");
  });

  it("ignores plain letters and other shortcuts", () => {
    expect(panelShortcut({ key: "q", metaKey: false })).toBeNull();
    expect(panelShortcut({ key: "w", metaKey: true })).toBeNull();
  });
});

describe("fittedHeight", () => {
  it("grows to the full content height", () => {
    expect(fittedHeight({ frameHeight: 300, scrollerVisible: 200, scrollerContent: 350, screenHeight: 1000 })).toBe(450);
  });

  it("caps at the maximum and at the screen", () => {
    const tall = { frameHeight: 300, scrollerVisible: 200, scrollerContent: 2000 };
    expect(fittedHeight({ ...tall, screenHeight: 1400 })).toBe(MAX_PANEL_HEIGHT);
    expect(fittedHeight({ ...tall, screenHeight: 600 })).toBe(552);
  });
});
