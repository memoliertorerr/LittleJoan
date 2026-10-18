import { describe, expect, it } from "vitest";
import { MAX_DOMAIN, displayDomain, extensionLabel, isMac, middleEllipsis, siteFor, toolbarView } from "./logic.ts";

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
    expect(toolbarView("", true)).toEqual({ domain: "", site: "", icon: null, back: false, openInBrowser: false });
  });

  it("enables back only once the page navigated within this window", () => {
    // A new window: Rust's counter says there is nowhere to go back to yet.
    expect(toolbarView(page, false)).toEqual({ domain: "127.0.0.1", site: "127.0.0.1:5000", icon: null, back: false, openInBrowser: true });
    // After a second page in the same window.
    expect(toolbarView("https://example.com/next", true).back).toBe(true);
  });

  it("never offers web actions for the app's own pages", () => {
    expect(toolbarView("tauri://localhost/index.html", true)).toEqual({ domain: "", site: "", icon: null, back: false, openInBrowser: false });
  });
});

describe("an extension's panel", () => {
  const icon = "data:image/svg+xml;base64,PHN2Zy8+";

  it("shows the extension's name and icon, and never the default browser", () => {
    const label = extensionLabel({ name: "Hello", icon });
    expect(toolbarView("", false, label)).toEqual({ domain: "Hello", site: "", icon, back: false, openInBrowser: false });
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
