import { describe, expect, it } from "vitest";
import { FORMATS, escapeHtml, escapeMarkdown, link, markdownUrl, move, trimSiteName } from "./logic.js";

describe("copy as Markdown link", () => {
  it("makes a Markdown link", () => {
    expect(link("markdown", "Example Domain", "https://example.com/")).toBe("[Example Domain](https://example.com/)");
  });

  it("escapes what would break the link text", () => {
    expect(escapeMarkdown("[draft] *bold* _it_ `code` a\\b <tag>")).toBe("\\[draft\\] \\*bold\\* \\_it\\_ \\`code\\` a\\\\b \\<tag\\>");
    expect(link("markdown", "Arrays [] and (parens)", "https://example.com/")).toBe("[Arrays \\[\\] and (parens)](https://example.com/)");
    expect(escapeMarkdown("two\nlines\t here")).toBe("two lines here");
  });

  it("keeps the address in one piece", () => {
    expect(markdownUrl("https://en.wikipedia.org/wiki/Bracket_(disambiguation)")).toBe("https://en.wikipedia.org/wiki/Bracket_%28disambiguation%29");
    expect(markdownUrl("https://example.com/a b<c>")).toBe("https://example.com/a%20b%3Cc%3E");
    expect(link("markdown", "Wiki", "https://en.wikipedia.org/wiki/A_(b)")).toBe("[Wiki](https://en.wikipedia.org/wiki/A_%28b%29)");
  });

  it("uses the address when there is no title", () => {
    expect(link("markdown", "   ", "https://example.com/")).toBe("[https://example.com/](https://example.com/)");
    expect(link("plain", "", "https://example.com/")).toBe("https://example.com/");
  });

  it("writes HTML, plain text and Org mode too", () => {
    expect(link("html", 'Tom & "Jerry" <3', "https://example.com/?a=1&b=2")).toBe('<a href="https://example.com/?a=1&amp;b=2">Tom &amp; &quot;Jerry&quot; &lt;3</a>');
    expect(escapeHtml("it's")).toBe("it&#39;s");
    expect(link("plain", "Example", "https://example.com/")).toBe("Example — https://example.com/");
    expect(link("org", "A [b] c", "https://example.com/[x]")).toBe("[[https://example.com/%5Bx%5D][A (b) c]]");
  });

  it("can drop a site name from a title", () => {
    expect(trimSiteName("How to bake bread - YouTube")).toBe("How to bake bread");
    expect(trimSiteName("Rust Book | The Rust Programming Language")).toBe("Rust Book");
    expect(trimSiteName("Short - X")).toBe("Short - X");
    expect(trimSiteName("No separator here")).toBe("No separator here");
  });

  it("moves between formats", () => {
    expect(FORMATS.map((f) => f.id)).toEqual(["markdown", "html", "plain", "org"]);
    expect(move("markdown", 1)).toBe("html");
    expect(move("org", 1)).toBe("org");
    expect(move("markdown", -1)).toBe("markdown");
  });
});
