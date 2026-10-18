import { describe, expect, it } from "vitest";
import { CITIES } from "./cities.ts";
import { answer, cityName, instantAt, offsetMinutes, parseClock, utcOffset, zoneFor } from "./time.ts";

// 2026-10-18 12:00:00 UTC (a Sunday): London is on BST (UTC+1) until Oct 25, New York on EDT.
const OCT_18 = Date.UTC(2026, 9, 18, 12, 0, 0);
// 2026-12-01 12:00:00 UTC: both back on standard time.
const DEC_1 = Date.UTC(2026, 11, 1, 12, 0, 0);

describe("time zones", () => {
  it("say the time in a city", () => {
    expect(answer("time in Tokyo", OCT_18, "en-US")).toEqual({
      title: "9:00 PM in Tokyo",
      subtitle: "Sun, Oct 18 · UTC+9",
      copy: "9:00 PM",
    });
    expect(answer("what time is it in new york", OCT_18, "en-US")?.title).toBe("8:00 AM in New York");
    expect(answer("london time", OCT_18, "en-US")?.subtitle).toBe("Sun, Oct 18 · UTC+1");
    expect(answer("time in mumbai", OCT_18, "en-US")?.subtitle).toBe("Sun, Oct 18 · UTC+5:30");
    expect(answer("time in nyc", OCT_18, "en-US")?.title).toBe("8:00 AM in NYC");
  });

  it("convert a time between cities, across daylight saving time", () => {
    expect(answer("3pm london to new york", OCT_18, "en-US")).toEqual({
      title: "10:00 AM in New York",
      subtitle: "3:00 PM in London · Sun, Oct 18 · UTC−4",
      copy: "10:00 AM",
    });
    // In December London is UTC+0 and New York UTC−5: still five hours apart.
    expect(answer("3pm london to new york", DEC_1, "en-US")?.title).toBe("10:00 AM in New York");
    // Between Oct 25 and Nov 1 2026 Europe has left summer time and the US hasn't: four hours.
    expect(answer("3pm london to new york", Date.UTC(2026, 9, 28, 12), "en-US")?.title).toBe("11:00 AM in New York");
    expect(answer("15:30 tokyo in paris", OCT_18, "en-US")?.title).toBe("8:30 AM in Paris");
    expect(answer("noon sydney to los angeles", OCT_18, "en-US")?.subtitle).toMatch(/^12:00 PM in Sydney · Sat, Oct 17/);
  });

  it("read clock times strictly", () => {
    expect(parseClock("3pm")).toEqual([15, 0]);
    expect(parseClock("3 PM")).toEqual([15, 0]);
    expect(parseClock("12am")).toEqual([0, 0]);
    expect(parseClock("12pm")).toEqual([12, 0]);
    expect(parseClock("9:05am")).toEqual([9, 5]);
    expect(parseClock("23:59")).toEqual([23, 59]);
    expect(parseClock("midnight")).toEqual([0, 0]);
    for (const bad of ["3", "13pm", "24:00", "9:60", "0am", "soon"]) {
      expect(parseClock(bad), bad).toBeNull();
    }
  });

  it("compute offsets and instants with Intl", () => {
    expect(offsetMinutes("Europe/London", OCT_18)).toBe(60);
    expect(offsetMinutes("Europe/London", DEC_1)).toBe(0);
    expect(offsetMinutes("America/St_Johns", DEC_1)).toBe(-210);
    expect(instantAt("America/New_York", OCT_18, 8, 0)).toBe(OCT_18);
    expect(utcOffset(0)).toBe("UTC");
    expect(utcOffset(-210)).toBe("UTC−3:30");
    expect(utcOffset(345)).toBe("UTC+5:45");
  });

  it("know about 150 cities, every one a real zone", () => {
    expect(Object.keys(CITIES).length).toBeGreaterThanOrEqual(150);
    for (const zone of new Set(Object.values(CITIES))) {
      expect(() => new Intl.DateTimeFormat("en-US", { timeZone: zone }), zone).not.toThrow();
    }
    expect(zoneFor(" New  York ")).toBe("America/New_York");
    expect(zoneFor("st. johns")).toBe("America/St_Johns");
    expect(zoneFor("constructor")).toBeUndefined();
    expect(cityName("new york")).toBe("New York");
    expect(cityName("sf")).toBe("SF");
  });

  it("have no answer for other text", () => {
    for (const bad of ["time in atlantis", "time", "3pm atlantis to london", "hello time", "10 km in mi", "= 12*7"]) {
      expect(answer(bad, OCT_18, "en-US"), bad).toBeNull();
    }
  });
});
