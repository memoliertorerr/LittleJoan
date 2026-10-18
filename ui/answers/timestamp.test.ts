import { describe, expect, it } from "vitest";
import { answer, utcText } from "./timestamp.ts";

const NOW = Date.UTC(2026, 9, 18, 12, 0, 0);

describe("unix timestamps", () => {
  it("read seconds and milliseconds", () => {
    expect(answer("1700000000", NOW)).toEqual({
      title: "2023-11-14 22:13:20 UTC",
      subtitle: "Unix time in seconds · 2023-11-14T22:13:20.000Z",
      copy: "2023-11-14T22:13:20.000Z",
    });
    expect(answer("1700000000123", NOW)?.subtitle).toBe("Unix time in milliseconds · 2023-11-14T22:13:20.123Z");
  });

  it("say the current one", () => {
    expect(answer("now unix", NOW)).toEqual({ title: "1792324800", subtitle: "Unix time now · 2026-10-18 12:00:00 UTC", copy: "1792324800" });
    expect(answer("unix now", NOW)?.title).toBe("1792324800");
    expect(answer("Timestamp", NOW)?.title).toBe("1792324800");
    expect(utcText(0)).toBe("1970-01-01 00:00:00 UTC");
  });

  it("have no answer for other numbers or text", () => {
    for (const bad of ["170000000", "17000000000", "2024", "now", "unix", "12*7"]) {
      expect(answer(bad, NOW), bad).toBeNull();
    }
  });
});
