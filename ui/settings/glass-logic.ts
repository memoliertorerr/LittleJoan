// Settings → Appearance's Liquid Glass (LJ19.9; pure, unit-tested): Off / Clear / Tinted, like macOS
// 26.1's own setting, for the toolbar and the command bar on top of the theme. It shows only where
// Rust says this Mac has glass (macOS 26+; never on Windows, Linux or older Macs), with a note
// saying what the choice does — and why the solid theme shows instead while Reduce Transparency or
// Increase Contrast is on (macOS hides its own Liquid Glass setting then too).

/** The setting (settings.json's `glass`). */
export type GlassSetting = "off" | "clear" | "tinted";

/** The choices, in order (drawn as text). */
export const GLASS_CHOICES: readonly { id: GlassSetting; name: string }[] = [
  { id: "off", name: "Off" },
  { id: "clear", name: "Clear" },
  { id: "tinted", name: "Tinted" },
];

/** What get_settings says of glass on this system. */
export interface GlassSupport {
  /** macOS 26 or later: the setting shows. */
  available: boolean;
  /** What shows the solid theme instead now. */
  blockedBy: "reduceTransparency" | "increaseContrast" | null;
}

/** No glass here (and what an older Rust, or a malformed answer, means). */
export const NO_GLASS: GlassSupport = { available: false, blockedBy: null };

/** The setting from anything: one of the three, else Off. */
export function glassSettingFrom(value: unknown): GlassSetting {
  return value === "clear" || value === "tinted" ? value : "off";
}

/** get_settings' `glassSupport`, each field checked; anything else is no glass. */
export function glassSupportFrom(value: unknown): GlassSupport {
  if (typeof value !== "object" || value === null) {
    return NO_GLASS;
  }
  const raw = value as Record<string, unknown>;
  const blockedBy = raw.blockedBy === "reduceTransparency" || raw.blockedBy === "increaseContrast" ? raw.blockedBy : null;
  return { available: raw.available === true, blockedBy };
}

const BLOCKED: Record<NonNullable<GlassSupport["blockedBy"]>, string> = {
  reduceTransparency: "Reduce Transparency is on in Accessibility settings, so the theme shows solid.",
  increaseContrast: "Increase Contrast is on in Accessibility settings, so the theme shows solid.",
};

/**
 * The note under the choices, one short line (the tab fits its window without scrolling): what
 * `choice` does — or, while the system asks for it, why the solid theme shows instead.
 */
export function glassNote(choice: GlassSetting, support: GlassSupport): string {
  if (support.blockedBy) {
    return BLOCKED[support.blockedBy];
  }
  return choice === "clear"
    ? "More transparent. Tinted by your theme; no grain on glass."
    : choice === "tinted"
      ? "More opaque, with more contrast. Tinted by your theme."
      : "Glass for the toolbar and the command bar, tinted by your theme.";
}
