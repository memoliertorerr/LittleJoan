import { describe, expect, it } from "vitest";
import { answer, convert, findUnit } from "./units.ts";

function unit(name: string) {
  const found = findUnit(name);
  if (!found) {
    throw new Error(`no unit ${name}`);
  }
  return found;
}

describe("unit conversion", () => {
  it("converts length, mass, volume, speed and data sizes", () => {
    expect(answer("10 km in mi")).toEqual({ title: "6.213711922 mi", subtitle: "10 km in mi", copy: "6.213711922" });
    expect(answer("1 mile to km")?.title).toBe("1.609344 km");
    expect(answer("6 ft in cm")?.title).toBe("182.88 cm");
    expect(answer("12 inches to feet")?.title).toBe("1 ft");
    expect(answer("5 kg in lb")?.title).toBe("11.02311311 lb");
    expect(answer("16 oz to g")?.title).toBe("453.59237 g");
    expect(answer("1 gallon in liters")?.title).toBe("3.785411784 l");
    expect(answer("250 ml to cups")?.title).toBe("1.056688209 cup");
    expect(answer("100 km/h in mph")?.title).toBe("62.13711922 mph");
    expect(answer("10 knots to km/h")?.title).toBe("18.52 km/h");
    expect(answer("1.5 GiB in MB")?.title).toBe("1610.612736 MB");
    expect(answer("1 TB to GiB")?.title).toBe("931.3225746 GiB");
    expect(answer("2048 kb as mb")?.title).toBe("2.048 MB");
  });

  it("converts temperatures", () => {
    expect(answer("72f to c")).toEqual({ title: "22.22222222 °C", subtitle: "72 °F in °C", copy: "22.22222222" });
    expect(answer("100 c in f")?.title).toBe("212 °F");
    expect(answer("-40 celsius to fahrenheit")?.title).toBe("-40 °F");
    expect(answer("0 k in c")?.title).toBe("-273.15 °C");
    expect(convert(300, unit("kelvin"), unit("f"))).toBeCloseTo(80.33, 2);
  });

  it("refuses different kinds, unknown units and other text", () => {
    for (const bad of ["10 km in kg", "5 parsecs in m", "10 km", "km in mi", "hello world", "10 km in", "= 12*7", "time in tokyo"]) {
      expect(answer(bad), bad).toBeNull();
    }
  });

  it("knows units by symbol and name, in any case", () => {
    expect(findUnit("KM")?.symbol).toBe("km");
    expect(findUnit("Kilometres")?.symbol).toBe("km");
    expect(findUnit("fl  oz")?.symbol).toBe("fl oz");
    expect(findUnit("°F")?.symbol).toBe("°F");
    expect(findUnit("parsec")).toBeUndefined();
  });
});
