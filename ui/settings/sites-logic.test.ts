import { describe, expect, it } from "vitest";
import { BUILT_IN_WHERE, afterAction, builtInsFrom, rowText, sameSites, siteModesFrom, siteRows, type BuiltInSite } from "./sites-logic.ts";

const YOUTUBE: BuiltInSite[] = [{ host: "youtube.com", mode: "desktop" }];

describe("the list of sites", () => {
  it("shows the built-in ones where they apply", () => {
    const rows = siteRows({}, YOUTUBE);
    expect(rows).toEqual([{ host: "youtube.com", mode: "Desktop", note: BUILT_IN_WHERE, action: "remove" }]);
    expect(rows.map(rowText)).toEqual(["youtube.com — Desktop, built in on macOS and Linux"]);
    expect(siteRows({}, [])).toEqual([]);
  });

  it("shows the user's entries sorted, each with Remove", () => {
    const rows = siteRows({ "example.com": "phone", "127.0.0.1": "desktop" }, []);
    expect(rows.map(rowText)).toEqual(["127.0.0.1 — Desktop", "example.com — Phone"]);
    expect(rows.every((row) => row.action === "remove")).toBe(true);
  });

  it("shows an overridden built-in with the user's mode and Reset", () => {
    const rows = siteRows({ "youtube.com": "auto" }, YOUTUBE);
    expect(rows).toEqual([{ host: "youtube.com", mode: "Automatic", note: "built in: Desktop", action: "reset" }]);
    expect(rows.map(rowText)).toEqual(["youtube.com — Automatic, built in: Desktop"]);
  });
});

describe("the buttons", () => {
  it("Remove takes an entry away", () => {
    expect(afterAction({ "example.com": "phone", "a.test": "desktop" }, [], "example.com")).toEqual({ "a.test": "desktop" });
  });

  it("Reset gives a built-in back", () => {
    expect(afterAction({ "youtube.com": "phone" }, YOUTUBE, "youtube.com")).toEqual({});
  });

  it("Remove of a built-in keeps auto for it, so the width decides there", () => {
    const next = afterAction({}, YOUTUBE, "youtube.com");
    expect(next).toEqual({ "youtube.com": "auto" });
    expect(siteRows(next, YOUTUBE)[0]?.action).toBe("reset");
  });

  it("does nothing for a site it doesn't know", () => {
    expect(afterAction({ "a.test": "phone" }, YOUTUBE, "b.test")).toEqual({ "a.test": "phone" });
  });
});

describe("what Rust sent", () => {
  it("keeps valid entries only", () => {
    expect(siteModesFrom({ "a.test": "phone", "b.test": "tablet", "": "auto", "c.test": 3 })).toEqual({ "a.test": "phone" });
    expect(siteModesFrom(null)).toEqual({});
    expect(siteModesFrom(["a"])).toEqual({});
    expect(builtInsFrom([{ host: "youtube.com", mode: "desktop" }, { host: "", mode: "auto" }, { host: "x.test", mode: "big" }, 7])).toEqual(YOUTUBE);
    expect(builtInsFrom(undefined)).toEqual([]);
  });

  it("compares tables by content", () => {
    expect(sameSites({ a: "phone", b: "auto" }, { b: "auto", a: "phone" })).toBe(true);
    expect(sameSites({ a: "phone" }, { a: "desktop" })).toBe(false);
    expect(sameSites({ a: "phone" }, {})).toBe(false);
  });
});
