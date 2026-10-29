import { describe, expect, it } from "vitest";
import { parseLog } from "./smoke-log.mts";
import { THEME_RUNS, earlyThemeFailures, parseEarlyTheme, parseTheme, themeFailures, type ThemeRun } from "./smoke-theme.mts";

const light = THEME_RUNS[0] as ThemeRun;
const defaultDark = THEME_RUNS[2] as ThemeRun;

const GOOD_LIGHT = `1 started
2 window-created
3 theme-early light colors=2
3 theme light colors=2
4 title LJ-SMOKE ready h264=mse
5 hibernated
6 theme-early light colors=2
6 window-created
7 theme light colors=2
8 suggestions 1 first=127.0.0.1:5000 typed=0
9 hibernated
10 settings-window appearance
11 settings-loaded
`;

describe("theme events", () => {
  it("parse the mode and the number of colors", () => {
    const [event] = parseLog("1 theme dark colors=3\n");
    expect(event && parseTheme(event)).toEqual({ mode: "dark", colors: 3 });
    for (const bad of ["1 theme sepia colors=1", "1 theme dark", "1 welcome"]) {
      const [e] = parseLog(`${bad}\n`);
      expect(e && parseTheme(e)).toBeUndefined();
    }
  });
});

describe("theme runs", () => {
  it("cover two colors with grain 0.4 in light and dark, and the default in dark", () => {
    expect(THEME_RUNS.map((run) => [run.theme.mode, run.theme.colors?.length ?? 0, run.theme.grain ?? null])).toEqual([
      ["light", 2, 0.4],
      ["dark", 2, 0.4],
      ["dark", 0, null],
    ]);
    expect(light.shots).toMatchObject({ page: "page.png", commandBar: "command-bar.png" });
    expect(THEME_RUNS[1]?.shots.commandBar).toBe("command-bar-dark.png");
  });

  it("pass when the bar got the theme, the page and the command bar showed, and Settings loaded", () => {
    expect(themeFailures(light, parseLog(GOOD_LIGHT))).toEqual([]);
    const defaults = "1 theme-early dark colors=1\n1 theme dark colors=1\n2 suggestions 1 first=127.0.0.1:5000 typed=0\n";
    expect(themeFailures(defaultDark, parseLog(defaults))).toEqual([]);
  });

  it("fail on the wrong theme, a missing page, command bar or Settings", () => {
    expect(themeFailures(light, parseLog(GOOD_LIGHT.replace("7 theme light colors=2", "7 theme dark colors=2")))).toEqual([
      "theme-light: the bar got a dark theme with 2 colors, not light with 2",
    ]);
    expect(themeFailures(light, parseLog("1 started\n"))).toEqual([
      "theme-light: the bar never asked for its theme",
      "theme-light: the smoke page never loaded",
      "theme-light: the command bar never offered the smoke page's site",
      "theme-light: Settings never opened on Appearance (or its page never loaded)",
    ]);
    expect(themeFailures(defaultDark, parseLog("1 theme-early dark colors=1\n1 theme dark colors=2\n2 suggestions 1 first=127.0.0.1:5000 typed=0\n"))).toEqual([
      "theme-default-dark: the bar got a dark theme with 2 colors, not dark with 1",
    ]);
  });
});

describe("the early theme (LJ38)", () => {
  it("passes when every window's bar painted the run's theme before start_info's", () => {
    expect(earlyThemeFailures(light, parseLog(GOOD_LIGHT))).toEqual([]);
    const [early] = parseLog("1 theme-early dark colors=3\n");
    expect(early && parseEarlyTheme(early)).toEqual({ mode: "dark", colors: 3 });
    expect(early && parseTheme(early)).toBeUndefined();
  });

  it("fails when a window got its theme only from start_info, or painted another one first", () => {
    const late = GOOD_LIGHT.replace("6 theme-early light colors=2\n", "");
    expect(earlyThemeFailures(light, parseLog(late))).toEqual([
      "theme-light: window #2's bar got its theme from start_info with no theme-early before it",
    ]);
    // An early report after start_info's doesn't count for that window.
    const after = GOOD_LIGHT.replace("6 theme-early light colors=2\n", "").replace("7 theme light colors=2\n", "7 theme light colors=2\n7 theme-early light colors=2\n");
    expect(earlyThemeFailures(light, parseLog(after))).toHaveLength(1);
    const wrong = GOOD_LIGHT.replace("3 theme-early light colors=2", "3 theme-early system colors=1");
    expect(themeFailures(light, parseLog(wrong))).toEqual([
      "theme-light: window #1's bar painted a system theme with 1 colors first, not light with 2",
    ]);
  });
});

describe("summons from the ready window (LJ19.4)", () => {
  it("hand the bar the run's theme again, and fail on another one", () => {
    const ready = `${GOOD_LIGHT}12 window-created ready\n12 theme-summon light colors=2\n`;
    expect(themeFailures(light, parseLog(ready))).toEqual([]);
    const wrong = `${GOOD_LIGHT}12 window-created ready\n12 theme-summon dark colors=1\n`;
    expect(themeFailures(light, parseLog(wrong))).toEqual([
      "theme-light: a summon from ready handed the bar a dark theme with 1 colors, not light with 2",
    ]);
  });
});
