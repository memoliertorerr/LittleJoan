// Pure logic of the bar's address field, kept apart from the DOM so it can be unit-tested. What
// keys do in each view lives in command/view-state.ts.

/**
 * Whether a new address from the app may replace the field's text: always while the user isn't
 * in the field, and while they are, only if they haven't changed what it showed.
 */
export function mayReplace(value: string, shown: string, editing: boolean): boolean {
  return !editing || value === shown;
}

/** The field's text for an address from the app (`null` = the start page). */
export function textFor(url: string | null): string {
  return url ?? "";
}
