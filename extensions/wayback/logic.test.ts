import { describe, expect, it } from "vitest";
import { ARCHIVE, archivable, archiveUrls, move } from "./logic.js";

describe("wayback", () => {
  it("opens the calendar and the newest snapshot of a page", () => {
    expect(archiveUrls("https://example.com/post?id=7")).toEqual({
      calendar: "https://web.archive.org/web/*/https://example.com/post?id=7",
      newest: "https://web.archive.org/web/https://example.com/post?id=7",
    });
    expect(archiveUrls("http://127.0.0.1:8080/smoke.html")?.calendar).toBe(`${ARCHIVE}*/http://127.0.0.1:8080/smoke.html`);
  });

  it("leaves out what never reaches a server", () => {
    expect(archivable("https://me:secret@example.com/a#section")).toBe("https://example.com/a");
    expect(archivable("  https://example.com  ")).toBe("https://example.com/");
  });

  it("only takes web addresses", () => {
    expect(archiveUrls("ljext://wayback.localhost/index.html")).toBeNull();
    expect(archiveUrls("javascript:alert(1)")).toBeNull();
    expect(archiveUrls("file:///etc/passwd")).toBeNull();
    expect(archiveUrls("not a url")).toBeNull();
  });

  it("moves between the two choices", () => {
    expect(move("calendar", 1)).toBe("newest");
    expect(move("newest", 1)).toBe("newest");
    expect(move("newest", -1)).toBe("calendar");
    expect(move("calendar", -1)).toBe("calendar");
  });
});
