import { describe, expect, it } from "vitest";
import { CASES, LINES, TextToolError, apply, base64Decode, base64Encode, counts, splitWords, urlDecode, urlEncode } from "./logic.js";

describe("text tools", () => {
  it("counts characters, words, lines, bytes, sentences and reading time", () => {
    expect(counts("")).toEqual({ characters: 0, charactersNoSpaces: 0, words: 0, lines: 0, bytes: 0, sentences: 0, readingMinutes: 0 });
    expect(counts("Hello there. How are you?\nI'm fine!")).toEqual({
      characters: 35,
      charactersNoSpaces: 29,
      words: 7,
      lines: 2,
      bytes: 35,
      sentences: 3,
      readingMinutes: 1,
    });
    // Emoji and accents count once each as characters, by their bytes in UTF-8.
    expect(counts("café 👋")).toMatchObject({ characters: 6, words: 1, bytes: 10 });
    expect(counts("word ".repeat(460))).toMatchObject({ words: 460, readingMinutes: 2 });
    expect(counts("Ünïcödé wörds, 日本語 too")).toMatchObject({ words: 4 });
  });

  it("changes case", () => {
    expect(CASES.upper?.("Little Joan")).toBe("LITTLE JOAN");
    expect(CASES.lower?.("Little Joan")).toBe("little joan");
    expect(CASES.title?.("the lord of the rings: the return of the king")).toBe("The Lord of the Rings: The Return of the King");
    expect(CASES.title?.("what it's for")).toBe("What It's For");
    expect(CASES.sentence?.("HELLO THERE. HOW ARE YOU? fine")).toBe("Hello there. How are you? Fine");
    expect(CASES.camel?.("Parse HTTP response")).toBe("parseHttpResponse");
    expect(CASES.pascal?.("parse http response")).toBe("ParseHttpResponse");
    expect(CASES.snake?.("parseHTTPResponse")).toBe("parse_http_response");
    expect(CASES.kebab?.("Little Joan — v0.2")).toBe("little-joan-v0-2");
    expect(CASES.constant?.("max retry count")).toBe("MAX_RETRY_COUNT");
    expect(splitWords("userID2Name")).toEqual(["user", "ID2", "Name"]);
  });

  it("encodes and decodes Base64 as UTF-8", () => {
    expect(base64Encode("Little Joan")).toBe("TGl0dGxlIEpvYW4=");
    expect(base64Encode("café ☕")).toBe("Y2Fmw6kg4piV");
    expect(base64Decode("Y2Fmw6kg4piV")).toBe("café ☕");
    expect(base64Decode("TGl0dGxlIEpvYW4")).toBe("Little Joan");
    expect(base64Decode(" TGl0 dGxl\nIEpvYW4= ")).toBe("Little Joan");
    // URL-safe Base64.
    expect(base64Decode(base64Encode("??>>").replace(/\+/g, "-").replace(/\//g, "_"))).toBe("??>>");
    expect(() => base64Decode("not base64!")).toThrow(TextToolError);
    expect(() => base64Decode("A")).toThrow("That isn't Base64.");
    expect(() => base64Decode("/w==")).toThrow("That Base64 isn't text (its bytes aren't UTF-8).");
    expect(base64Encode("x".repeat(100_000))).toHaveLength(133_336);
  });

  it("encodes and decodes for URLs", () => {
    expect(urlEncode("a b&c=d/é")).toBe("a%20b%26c%3Dd%2F%C3%A9");
    expect(urlDecode("a%20b%26c%3Dd%2F%C3%A9")).toBe("a b&c=d/é");
    expect(urlDecode("one+two")).toBe("one two");
    expect(() => urlDecode("100%")).toThrow(TextToolError);
  });

  it("sorts and tidies lines", () => {
    expect(LINES.sort?.("pear\nApple\nbanana\nitem 10\nitem 2")).toBe("Apple\nbanana\nitem 2\nitem 10\npear");
    expect(LINES.reverseSort?.("a\nc\nb")).toBe("c\nb\na");
    expect(LINES.reverse?.("1\n2\n3")).toBe("3\n2\n1");
    expect(LINES.unique?.("a\nb\na\nc\nb")).toBe("a\nb\nc");
    expect(LINES.trim?.("  a  \n\tb")).toBe("a\nb");
    expect(LINES.removeEmpty?.("a\n\n  \nb")).toBe("a\nb");
    expect(LINES.sort?.("b\r\na")).toBe("a\nb");
  });

  it("applies a tool to the selection, or to everything", () => {
    expect(apply("case.upper", "one two three", 4, 7)).toEqual({ text: "one TWO three", start: 4, end: 7 });
    expect(apply("case.upper", "one two", 3, 3)).toEqual({ text: "ONE TWO", start: 0, end: 7 });
    expect(apply("base64.encode", "x hi", 2, 4)).toEqual({ text: "x aGk=", start: 2, end: 6 });
    expect(() => apply("nope", "x", 0, 1)).toThrow(TextToolError);
  });
});
