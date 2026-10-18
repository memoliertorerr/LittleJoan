// Quick answers: Little Joan's own code in the bar (third-party providers are Later — they would
// run on every keystroke). Each built-in can be switched off in Settings → Extensions → Built-in
// (`disabledExtensions`, ids `builtin:<name>`). The first that answers gives the command bar's
// first row; Enter copies its text.
import type { Row } from "../command/rows.ts";
import type { BuiltIn } from "../settings/extensions-logic.ts";
import * as calc from "./calc.ts";
import * as random from "./random.ts";
import * as time from "./time.ts";
import * as timestamp from "./timestamp.ts";
import type { Answer } from "./types.ts";
import * as units from "./units.ts";

/** The built-ins, in the order they are asked, as Settings lists them. */
export const BUILT_INS: readonly BuiltIn[] = [
  { id: "builtin:calc", name: "Calculator", description: "= 12*7, sqrt(2), 50% * 30, (1+2)^3, pi" },
  { id: "builtin:units", name: "Unit conversion", description: "10 km in mi, 72f to c, 1.5 GiB in MB" },
  { id: "builtin:time", name: "Time zones", description: "time in Tokyo, 3pm London to New York" },
  { id: "builtin:random", name: "Random", description: "uuid, password 20, roll 2d6, flip a coin, random 1-100" },
  { id: "builtin:timestamp", name: "Unix timestamps", description: "1700000000, now unix" },
];

export interface Context {
  /** Unix ms. */
  now: number;
  /** The built-ins switched off (`builtin:<name>`). */
  disabled: readonly string[];
  random?: random.RandomInt;
  /** How times are written; the system's when undefined. */
  locale?: string;
}

/** The answer for `query`, if a switched-on built-in has one. */
export function findAnswer(query: string, context: Context): Answer | null {
  const on = (id: string): boolean => !context.disabled.includes(id);
  const tries: [string, () => Answer | null][] = [
    ["builtin:calc", () => calc.answer(query)],
    ["builtin:units", () => units.answer(query)],
    ["builtin:time", () => time.answer(query, context.now, context.locale)],
    ["builtin:random", () => random.answer(query, context.random)],
    ["builtin:timestamp", () => timestamp.answer(query, context.now)],
  ];
  for (const [id, ask] of tries) {
    if (on(id)) {
      const found = ask();
      if (found) {
        return found;
      }
    }
  }
  return null;
}

/** The command bar's answer row: the answer as its title, what was asked muted; Enter copies. */
export function quickAnswer(query: string, context: Context): Row | null {
  const found = findAnswer(query, context);
  return found
    ? { kind: "answer", title: found.title, domain: found.subtitle, host: "", icon: "answer", action: { kind: "copy", text: found.copy } }
    : null;
}
