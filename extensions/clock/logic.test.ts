import { describe, expect, it } from "vitest";
import { CITIES } from "./cities.js";
import {
  BY_ID,
  MAX_CITIES,
  addCity,
  cityId,
  clockText,
  dayLabel,
  daypart,
  difference,
  differenceText,
  fold,
  moveCity,
  offsetMinutes,
  parseList,
  removeCity,
  rowKey,
  search,
  utcText,
  wallTime,
  zoneName,
} from "./logic.js";

/** Noon UTC on a day of October / November 2026. */
const noon = (month: number, day: number) => Date.UTC(2026, month - 1, day, 12, 0);

describe("world clock", () => {
  it("knows every city's zone, and every city by one id", () => {
    for (const city of CITIES) {
      expect(() => new Intl.DateTimeFormat("en-US", { timeZone: city.zone }), city.zone).not.toThrow();
    }
    expect(BY_ID.size).toBe(CITIES.length);
    expect(cityId({ name: "New York", country: "", zone: "" })).toBe("new-york");
    expect(cityId({ name: "St. John's", country: "", zone: "" })).toBe("st-john-s");
    expect(cityId({ name: "São Paulo", country: "", zone: "" })).toBe("sao-paulo");
    expect(cityId({ name: "İzmir", country: "", zone: "" })).toBe("izmir");
    expect(fold("Reykjavík")).toBe("reykjavik");
  });

  it("finds cities by name or country, names first", () => {
    expect(search("tok").map((m) => m.id)).toEqual(["tokyo"]);
    expect(search("sao").map((m) => m.id)).toEqual(["sao-paulo"]);
    expect(search("new").map((m) => m.id).slice(0, 3)).toEqual(["new-delhi", "new-orleans", "new-york"]);
    expect(search("york").map((m) => m.id)).toEqual(["new-york"]);
    expect(search("japan").map((m) => m.id)).toEqual(["kyoto", "osaka", "tokyo"]);
    expect(search("san fr").map((m) => m.id)).toEqual(["san-francisco"]);
    expect(search("tokyo", ["tokyo"])).toEqual([]);
    expect(search("   ")).toEqual([]);
    expect(search("united", [], 3)).toHaveLength(3);
  });

  it("is right on both sides of a daylight-saving change", () => {
    // 2026: Europe leaves summer time on 25 October, the US on 1 November.
    expect(offsetMinutes("Europe/London", noon(10, 20))).toBe(60);
    expect(offsetMinutes("Europe/London", noon(10, 28))).toBe(0);
    expect(offsetMinutes("America/New_York", noon(10, 28))).toBe(-240);
    expect(offsetMinutes("America/New_York", noon(11, 2))).toBe(-300);
    expect(difference("America/New_York", "Europe/London", noon(10, 20))).toBe(-300);
    expect(difference("America/New_York", "Europe/London", noon(10, 28))).toBe(-240);
    expect(difference("America/New_York", "Europe/London", noon(11, 2))).toBe(-300);
    // The southern hemisphere goes the other way: Sydney is on summer time from 4 October.
    expect(offsetMinutes("Australia/Sydney", noon(9, 30))).toBe(600);
    expect(offsetMinutes("Australia/Sydney", noon(10, 18))).toBe(660);
    // Half and quarter hours.
    expect(difference("Asia/Kolkata", "UTC", noon(10, 18))).toBe(330);
    expect(difference("Asia/Kathmandu", "Asia/Kolkata", noon(10, 18))).toBe(15);
    // The wall clock itself, at the change: 01:59:59 BST, then 01:00:00 GMT again.
    expect(wallTime("Europe/London", Date.UTC(2026, 9, 25, 0, 59, 59))).toMatchObject({ hour: 1, minute: 59 });
    expect(wallTime("Europe/London", Date.UTC(2026, 9, 25, 1, 0, 0))).toMatchObject({ hour: 1, minute: 0 });
  });

  it("writes differences and offsets plainly", () => {
    expect(differenceText(0)).toBe("Same time");
    expect(differenceText(540)).toBe("+9 h");
    expect(differenceText(-240)).toBe("−4 h");
    expect(differenceText(330)).toBe("+5 h 30 min");
    expect(differenceText(-30)).toBe("−30 min");
    expect(utcText(0)).toBe("UTC");
    expect(utcText(540)).toBe("UTC+9");
    expect(utcText(-210)).toBe("UTC−3:30");
  });

  it("says which day it is there", () => {
    const lateInLondon = Date.UTC(2026, 9, 28, 22, 30);
    expect(dayLabel("Asia/Tokyo", "Europe/London", lateInLondon)).toBe("Tomorrow");
    expect(dayLabel("America/Los_Angeles", "Europe/London", lateInLondon)).toBe("Today");
    expect(dayLabel("Pacific/Honolulu", "Asia/Tokyo", Date.UTC(2026, 9, 28, 1, 0))).toBe("Yesterday");
    // Across a month's end.
    expect(dayLabel("Pacific/Auckland", "America/New_York", Date.UTC(2026, 9, 31, 20, 0))).toBe("Tomorrow");
  });

  it("knows night from day, and writes the time as the locale does", () => {
    expect([0, 5, 6, 11, 12, 17, 18, 20, 21, 23].map(daypart)).toEqual(["night", "night", "morning", "morning", "day", "day", "evening", "evening", "night", "night"]);
    const at = Date.UTC(2026, 9, 18, 13, 5);
    expect(clockText("UTC", at, "en-GB")).toBe("13:05");
    expect(clockText("UTC", at, "en-US").replace(/\s/g, " ")).toBe("1:05 PM");
    expect(clockText("Asia/Tokyo", at, "en-GB")).toBe("22:05");
  });

  it("names your own zone", () => {
    expect(zoneName("Asia/Tokyo")).toBe("Tokyo");
    expect(zoneName("America/Argentina/Buenos_Aires")).toBe("Buenos Aires");
    expect(zoneName("America/Indiana/Knox")).toBe("Knox");
  });

  it("adds, removes and moves cities in its list", () => {
    let list = addCity([], "tokyo");
    list = addCity(list, "london");
    expect(addCity(list, "tokyo")).toEqual(["tokyo", "london"]);
    expect(addCity(list, "atlantis")).toEqual(["tokyo", "london"]);
    expect(moveCity(list, "london", -1)).toEqual(["london", "tokyo"]);
    expect(moveCity(list, "tokyo", -1)).toEqual(["tokyo", "london"]);
    expect(moveCity(list, "london", 1)).toEqual(["tokyo", "london"]);
    expect(removeCity(list, "tokyo")).toEqual(["london"]);
    const full = [...BY_ID.keys()].slice(0, MAX_CITIES);
    expect(addCity(full, [...BY_ID.keys()][MAX_CITIES] ?? "")).toHaveLength(MAX_CITIES);
  });

  it("reads its list from scratch, starting with a few cities the first time", () => {
    expect(parseList(["tokyo", "tokyo", "atlantis", 3, "london"], "UTC", noon(10, 18))).toEqual(["tokyo", "london"]);
    expect(parseList("tokyo", "UTC", noon(10, 18))).toEqual([]);
    expect(parseList([], "UTC", noon(10, 18))).toEqual([]);
    expect(parseList(undefined, "Asia/Istanbul", noon(10, 18))).toEqual(["london", "new-york", "tokyo"]);
    // Not the zone you're in (London in winter is UTC).
    expect(parseList(undefined, "Europe/London", noon(10, 28))).toEqual(["new-york", "tokyo", "sydney"]);
    expect(parseList(undefined, "UTC", noon(10, 28))).toEqual(["new-york", "tokyo", "sydney"]);
  });

  it("knows the rows' keys", () => {
    expect(rowKey({ key: "ArrowDown" })).toBe("down");
    expect(rowKey({ key: "ArrowUp", altKey: true })).toBe("move-up");
    expect(rowKey({ key: "ArrowDown", altKey: true })).toBe("move-down");
    expect(rowKey({ key: "Delete" })).toBe("remove");
    expect(rowKey({ key: "a", code: "KeyA" })).toBe("add");
    expect(rowKey({ key: "/" })).toBe("add");
    expect(rowKey({ key: "Escape" })).toBe("escape");
    expect(rowKey({ key: "a", code: "KeyA", metaKey: true })).toBeNull();
    expect(rowKey({ key: "x", code: "KeyX" })).toBeNull();
  });
});
