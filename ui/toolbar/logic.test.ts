import { describe, expect, it } from "vitest";
import {
  MAX_DOMAIN,
  MAX_STATUS_NAME,
  STATUS_LINGER_MS,
  displayDomain,
  extensionLabel,
  initialStatus,
  isMac,
  middleEllipsis,
  nextStatusChange,
  pillFor,
  shownStatus,
  siteFor,
  siteModeInfo,
  statusMessage,
  statusView,
  toolbarView,
  withStatus,
  withToolbarVisible,
  type SiteModeInfo,
  type StatusMessage,
} from "./logic.ts";

describe("the domain shown", () => {
  it("strips www.", () => {
    expect(displayDomain("https://www.example.com/a?b=1#c")).toBe("example.com");
    expect(displayDomain("https://WWW.Example.com/")).toBe("example.com");
    expect(displayDomain("https://news.example.com/")).toBe("news.example.com");
    expect(displayDomain("https://wwwx.test/")).toBe("wwwx.test");
  });

  it("shows IP addresses as is, without the port", () => {
    expect(displayDomain("http://127.0.0.1:5000/smoke.html")).toBe("127.0.0.1");
    expect(displayDomain("http://[::1]:8080/")).toBe("[::1]");
  });

  it("shortens long hosts in the middle", () => {
    const long = "a-very-long-subdomain.of-an-even-longer-domain.example.com";
    const shown = displayDomain(`https://${long}/`);
    expect(Array.from(shown)).toHaveLength(MAX_DOMAIN);
    expect(shown.startsWith("a-very-long-subd")).toBe(true);
    expect(shown.endsWith("example.com")).toBe(true);
    expect(shown).toContain("…");
    expect(middleEllipsis("short.test")).toBe("short.test");
    expect(middleEllipsis("abcdefghij", 5)).toBe("ab…ij");
  });

  it("is empty for what isn't an address", () => {
    expect(displayDomain("")).toBe("");
    expect(displayDomain("not a url")).toBe("");
  });

  it("names the site for its icon with the port, www. stripped", () => {
    expect(siteFor("http://127.0.0.1:5000/smoke.html")).toBe("127.0.0.1:5000");
    expect(siteFor("https://www.example.com/")).toBe("example.com");
    expect(siteFor("nope")).toBe("");
  });
});

describe("the toolbar", () => {
  const page = "http://127.0.0.1:5000/smoke.html";

  it("has no domain and no working buttons without a page", () => {
    expect(toolbarView("", true)).toEqual({ domain: "", site: "", icon: null, badge: "", back: false, openInBrowser: false, pill: null });
  });

  it("enables back only once the page navigated within this window", () => {
    // A new window: Rust's counter says there is nowhere to go back to yet.
    expect(toolbarView(page, false)).toEqual({ domain: "127.0.0.1", site: "127.0.0.1:5000", icon: null, badge: "", back: false, openInBrowser: true, pill: null });
    // After a second page in the same window.
    expect(toolbarView("https://example.com/next", true).back).toBe(true);
  });

  it("never offers web actions for the app's own pages", () => {
    expect(toolbarView("tauri://localhost/index.html", true)).toEqual({ domain: "", site: "", icon: null, badge: "", back: false, openInBrowser: false, pill: null });
  });
});

describe("an extension's panel", () => {
  const icon = "data:image/svg+xml;base64,PHN2Zy8+";

  it("shows the extension's name and icon, and never the default browser", () => {
    const label = extensionLabel({ name: "Hello", icon });
    expect(toolbarView("", false, label)).toEqual({ domain: "Hello", site: "", icon, badge: "Extension", back: false, openInBrowser: false, pill: null });
    // A web page never carries the badge, so a name next to it reads as an extension's.
    expect(toolbarView("https://paypal.com/", false).badge).toBe("");
    expect(toolbarView("", true, label).back).toBe(true);
  });

  it("takes only a checked label from Rust", () => {
    expect(extensionLabel(null)).toBeNull();
    expect(extensionLabel({ name: "" })).toBeNull();
    expect(extensionLabel({ name: "x".repeat(41) })).toBeNull();
    expect(extensionLabel({ name: "  Notes  ", icon: null })).toEqual({ name: "Notes", icon: null });
    // Markup in the name stays text (toolbar.ts draws it with textContent).
    expect(extensionLabel({ name: "<img src=x onerror=alert(1)>" })?.name).toBe("<img src=x onerror=alert(1)>");
    for (const bad of [
      "https://example.com/icon.svg",
      "ljext://hello.localhost/icon.svg",
      "data:text/html;base64,PHNjcmlwdD4=",
      "data:image/svg+xml,<svg onload=alert(1)>",
      "javascript:alert(1)",
      `${icon}"`,
    ]) {
      expect(extensionLabel({ name: "Hello", icon: bad })?.icon).toBeNull();
    }
    expect(extensionLabel({ name: "Hello", icon: "data:image/png;base64,iVBORw0KGgo=" })?.icon).toBe("data:image/png;base64,iVBORw0KGgo=");
  });
});

describe("the platform", () => {
  it("knows macOS from WKWebView's user agent", () => {
    expect(isMac("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)")).toBe(true);
    expect(isMac("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Edg/141.0")).toBe(false);
    expect(isMac("Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15")).toBe(false);
  });
});

describe("the phone / desktop pill (LJ40)", () => {
  const page = "https://www.youtube.com/watch?v=x";
  const info = (mode: SiteModeInfo["mode"], agent: SiteModeInfo["agent"], builtIn = false): SiteModeInfo => ({
    host: "youtube.com",
    mode,
    builtIn,
    agent,
  });

  it("shows while the page has the phone version, and a click asks for desktop", () => {
    expect(pillFor(info("auto", "phone"), page, null)).toMatchObject({ label: "Phone", next: "desktop" });
    expect(pillFor(info("phone", "phone"), page, null)).toMatchObject({ label: "Phone", next: "desktop" });
  });

  it("shows for a site with a mode of its own (a built-in one too), and a click asks for phone", () => {
    expect(pillFor(info("desktop", "desktop"), page, null)).toMatchObject({ label: "Desktop", next: "phone" });
    expect(pillFor(info("desktop", "desktop", true), page, null)).toMatchObject({ label: "Desktop", next: "phone" });
  });

  it("is gone for a desktop page that follows the width — most of the time", () => {
    expect(pillFor(info("auto", "desktop"), page, null)).toBeNull();
  });

  it("is never on the start page, a panel, or without Rust's info", () => {
    expect(pillFor(info("auto", "phone"), "", null)).toBeNull();
    expect(pillFor(info("auto", "phone"), "tauri://localhost/index.html", null)).toBeNull();
    expect(pillFor(info("auto", "phone"), page, { name: "Snake", icon: null })).toBeNull();
    expect(pillFor(null, page, null)).toBeNull();
    expect(toolbarView(page, false, { name: "Snake", icon: null }, info("auto", "phone")).pill).toBeNull();
    expect(toolbarView(page, false, null, info("auto", "phone")).pill?.label).toBe("Phone");
  });

  it("names the site in its tooltip, shortened", () => {
    expect(pillFor(info("auto", "phone"), page, null)?.title).toContain("youtube.com");
    const long = { ...info("auto", "phone"), host: `${"a".repeat(60)}.example.com` };
    expect(pillFor(long, page, null)?.title).toContain("…");
  });

  it("takes only a well-formed info from Rust", () => {
    expect(siteModeInfo({ host: "youtube.com", mode: "desktop", builtIn: true, agent: "desktop" })).toEqual(info("desktop", "desktop", true));
    for (const bad of [
      null,
      "youtube.com",
      { host: "", mode: "auto", builtIn: false, agent: "phone" },
      { host: "a.b", mode: "tablet", builtIn: false, agent: "phone" },
      { host: "a.b", mode: "auto", builtIn: "no", agent: "phone" },
      { host: "a.b", mode: "auto", builtIn: false, agent: "watch" },
      { host: "x".repeat(254), mode: "auto", builtIn: false, agent: "phone" },
    ]) {
      expect(siteModeInfo(bad), JSON.stringify(bad)).toBeNull();
    }
  });
});

const download = (state: "downloading" | "saved" | "failed" | "blocked", name = "lj-smoke.bin"): StatusMessage => ({
  kind: "download",
  state,
  name,
});

describe("the status Rust sends", () => {
  it("is checked", () => {
    expect(statusMessage({ kind: "download", state: "saved", name: "a.pdf" })).toEqual(download("saved", "a.pdf"));
    expect(statusMessage({ kind: "download", state: "failed", name: "" })).toEqual(download("failed", ""));
    expect(statusMessage({ kind: "download", state: "blocked", name: "" })).toEqual(download("blocked", ""));
    for (const bad of [
      null,
      "saved",
      {},
      { kind: "download", state: "exploded", name: "a" },
      { kind: "download", state: "saved" },
      { kind: "download", state: "saved", name: "" },
      { kind: "download", state: "downloading", name: 3 },
      { kind: "download", state: "saved", name: "x".repeat(1_001) },
      { kind: "other", state: "saved", name: "a" },
    ]) {
      expect(statusMessage(bad), JSON.stringify(bad)).toBeNull();
    }
  });

  it("reads as the plan says, with Show in folder once saved", () => {
    expect(statusView(download("downloading"))).toEqual({ text: "Downloading lj-smoke.bin…", title: "Downloading lj-smoke.bin…", action: null });
    expect(statusView(download("saved"))).toEqual({
      text: "lj-smoke.bin saved",
      title: "lj-smoke.bin saved in Downloads",
      action: { label: "Show in folder", command: "reveal_download" },
    });
    expect(statusView(download("failed")).text).toBe("Download failed");
    expect(statusView(download("failed")).title).toBe("Download failed: lj-smoke.bin");
    expect(statusView(download("failed", "")).title).toBe("Download failed");
    expect(statusView(download("blocked", "")).text).toBe("Download blocked");
  });

  it("shortens long names in the middle, keeping the extension in sight", () => {
    const long = `${"quarterly-report-".repeat(5)}final.pdf`;
    const view = statusView(download("saved", long));
    expect(view.text.endsWith("final.pdf saved")).toBe(true);
    expect(Array.from(view.text.replace(/ saved$/, ""))).toHaveLength(MAX_STATUS_NAME);
    expect(view.title).toBe(`${long} saved in Downloads`);
  });

  it("keeps markup as text (it is drawn with textContent)", () => {
    const view = statusView(download("saved", "<img src=x onerror=alert(1)>.html"));
    expect(view.text).toBe("<img src=x onerror=alert(1)>.html saved");
  });
});

describe("the status area over time", () => {
  it("goes downloading → saved → cleared after 8 s", () => {
    let state = initialStatus(true, 0);
    expect(shownStatus(state, 0)).toBeNull();
    state = withStatus(state, download("downloading"), 100);
    expect(shownStatus(state, 60_000)?.text).toBe("Downloading lj-smoke.bin…");
    expect(nextStatusChange(state, 60_000)).toBeNull();
    state = withStatus(state, download("saved"), 60_000);
    expect(shownStatus(state, 60_000)?.text).toBe("lj-smoke.bin saved");
    expect(shownStatus(state, 60_000 + STATUS_LINGER_MS - 1)?.action?.label).toBe("Show in folder");
    expect(nextStatusChange(state, 61_000)).toBe(STATUS_LINGER_MS - 1_000);
    expect(shownStatus(state, 60_000 + STATUS_LINGER_MS)).toBeNull();
    expect(nextStatusChange(state, 60_000 + STATUS_LINGER_MS)).toBeNull();
  });

  it("shows a failure for 8 s too", () => {
    let state = withStatus(initialStatus(true, 0), download("downloading"), 0);
    state = withStatus(state, download("failed"), 500);
    expect(shownStatus(state, 500)?.text).toBe("Download failed");
    expect(shownStatus(state, 500 + STATUS_LINGER_MS - 1)?.text).toBe("Download failed");
    expect(shownStatus(state, 500 + STATUS_LINGER_MS)).toBeNull();
  });

  it("never loses a status while the command bar is open", () => {
    // Saved while the toolbar shows; the command bar opens after 3 s and stays a minute.
    let state = withStatus(initialStatus(true, 0), download("saved"), 0);
    state = withToolbarVisible(state, false, 3_000);
    expect(nextStatusChange(state, 3_000)).toBeNull();
    expect(shownStatus(state, 63_000)?.text).toBe("lj-smoke.bin saved");
    // Back to the toolbar: the 5 s it had left.
    state = withToolbarVisible(state, true, 63_000);
    expect(shownStatus(state, 67_999)?.text).toBe("lj-smoke.bin saved");
    expect(nextStatusChange(state, 63_000)).toBe(5_000);
    expect(shownStatus(state, 68_000)).toBeNull();
  });

  it("waits for the toolbar when it arrives during the command bar", () => {
    // A new window opens with the command bar: a download finishing meanwhile waits.
    let state = initialStatus(false, 0);
    state = withStatus(state, download("saved"), 1_000);
    expect(shownStatus(state, 30_000)?.text).toBe("lj-smoke.bin saved");
    state = withToolbarVisible(state, true, 30_000);
    expect(shownStatus(state, 30_000 + STATUS_LINGER_MS - 1)?.text).toBe("lj-smoke.bin saved");
    expect(shownStatus(state, 30_000 + STATUS_LINGER_MS)).toBeNull();
  });

  it("shows the latest download, one at a time", () => {
    let state = withStatus(initialStatus(true, 0), download("saved", "a.zip"), 0);
    state = withStatus(state, download("downloading", "b.zip"), 1_000);
    expect(shownStatus(state, 1_000)?.text).toBe("Downloading b.zip…");
    expect(shownStatus(state, 20_000)?.text).toBe("Downloading b.zip…");
    state = withStatus(state, download("saved", "b.zip"), 20_000);
    expect(shownStatus(state, 20_000)?.text).toBe("b.zip saved");
  });
});
