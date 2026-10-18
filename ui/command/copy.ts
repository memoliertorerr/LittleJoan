// Copying a quick answer from the trusted bar (DOM glue): the async clipboard API on the Enter
// keypress (or click) that asked for it, falling back to a hidden textarea and execCommand, which
// some engines still need without a fresh user gesture. The field gets the keyboard back after.

export async function copyText(text: string, refocus: HTMLElement): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    // Fall back below.
  }
  const area = document.createElement("textarea");
  area.value = text;
  area.setAttribute("readonly", "");
  area.setAttribute("aria-hidden", "true");
  area.style.setProperty("position", "fixed");
  area.style.setProperty("opacity", "0");
  area.style.setProperty("pointer-events", "none");
  document.body.append(area);
  area.select();
  let copied: boolean;
  try {
    // The textarea fallback: deprecated, but the one way left when the async API is refused.
    copied = document.execCommand("copy");
  } catch {
    copied = false;
  }
  area.remove();
  refocus.focus();
  return copied;
}
