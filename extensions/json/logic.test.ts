import { describe, expect, it } from "vitest";
import { MAX_DEPTH, excerpt, format, minify, parse, position, sortKeys, stats, tokens, type Node } from "./logic.js";

function value(text: string): Node {
  const parsed = parse(text);
  if (!parsed.ok) {
    throw new Error(parsed.error.message);
  }
  return parsed.value;
}

function error(text: string) {
  const parsed = parse(text);
  if (parsed.ok) {
    throw new Error(`parsed: ${text}`);
  }
  return parsed.error;
}

describe("JSON tool", () => {
  it("pretty-prints with two spaces, four or tabs", () => {
    const node = value('{"name":"Little Joan","tags":["tiny","browser"],"size":{"w":480,"h":640},"empty":{},"none":[],"ok":true,"x":null}');
    expect(format(node)).toBe(
      [
        "{",
        '  "name": "Little Joan",',
        '  "tags": [',
        '    "tiny",',
        '    "browser"',
        "  ],",
        '  "size": {',
        '    "w": 480,',
        '    "h": 640',
        "  },",
        '  "empty": {},',
        '  "none": [],',
        '  "ok": true,',
        '  "x": null',
        "}",
      ].join("\n"),
    );
    expect(format(value("[1,[2]]"), "\t")).toBe("[\n\t1,\n\t[\n\t\t2\n\t]\n]");
    expect(format(value('{"a":1}'), "    ")).toBe('{\n    "a": 1\n}');
  });

  it("minifies", () => {
    expect(minify(value(' {\n "a" : [ 1 , 2 ] ,\n "b" : { } }\n'))).toBe('{"a":[1,2],"b":{}}');
  });

  it("never changes a value: numbers and strings stay as written", () => {
    const text = '{"big":12345678901234567890,"exp":1.0E+3,"neg":-0.5,"escaped":"caf\\u00e9 \\"q\\" \\n","unicode":"☕"}';
    expect(minify(value(text))).toBe(text);
    expect(format(value(text))).toContain('"big": 12345678901234567890');
    expect(JSON.parse(format(value(text)))).toEqual(JSON.parse(text));
  });

  it("validates, saying where the error is", () => {
    expect(error('{\n  "a": 1,\n  "b" 2\n}')).toEqual({ message: "Expected ':' after the key, found '2'", offset: 18, line: 3, column: 7 });
    expect(error('{"a": 1,}')).toMatchObject({ message: "A comma before '}' (JSON has no trailing commas)", line: 1, column: 9 });
    expect(error("[1, 2,]").message).toBe("A comma before ']' (JSON has no trailing commas)");
    expect(error("{'a': 1}").message).toBe("Expected a key in double quotes, found '''");
    expect(error("['a']").message).toBe('Strings need double quotes ("), not single ones');
    expect(error('{"a": 1 "b": 2}').message).toBe("Expected ',' or '}' after a value, found '\"'");
    expect(error("[1 2]").message).toBe("Expected ',' or ']' after an item, found '2'");
    expect(error('"abc').message).toBe('A string that never ends (no closing ")');
    expect(error('"a\nb"').message).toBe("A line break inside a string (write \\n)");
    expect(error('"\\x"').message).toBe("Unknown escape \\x in a string");
    expect(error('"\\u12"').message).toBe("\\u needs four hex digits");
    expect(error("01").message).toBe("Numbers can't start with 0");
    expect(error("1.").message).toBe("A number that isn't one");
    expect(error("-").message).toBe("A number that isn't one");
    expect(error("tru").message).toBe("Expected a value, found 't'");
    expect(error('{"a": }')).toMatchObject({ message: "Expected a value, found '}'", column: 7 });
    expect(error("").message).toBe("The JSON ends where a value should be");
    expect(error("   ")).toMatchObject({ line: 1, column: 4 });
    expect(error("{} {}")).toMatchObject({ message: "More after the JSON value: '{'", column: 4 });
    expect(error("[".repeat(MAX_DEPTH + 2)).message).toBe(`Nested deeper than ${MAX_DEPTH} levels`);
  });

  it("allows a byte-order mark and surrounding whitespace", () => {
    expect(minify(value('﻿  {"a":1}\n\n'))).toBe('{"a":1}');
  });

  it("shows the line of an error with a caret", () => {
    const text = '{\n  "a": tru\n}';
    expect(excerpt(text, error(text))).toBe('  "a": tru\n       ^');
    expect(position("ab\ncd", 4)).toEqual({ line: 2, column: 2 });
  });

  it("sorts keys at every level, arrays kept in order", () => {
    expect(minify(sortKeys(value('{"b":1,"a":{"d":[{"z":1,"y":2}],"c":3}}')))).toBe('{"a":{"c":3,"d":[{"y":2,"z":1}]},"b":1}');
  });

  it("counts what the value holds", () => {
    expect(stats(value('{"a":[1,2,{"b":null}],"c":"x"}'))).toEqual({ objects: 2, arrays: 1, values: 4, depth: 3 });
    expect(stats(value("7"))).toEqual({ objects: 0, arrays: 0, values: 1, depth: 0 });
  });

  it("marks keys, strings, numbers and literals for highlighting", () => {
    expect(tokens(value('{"k":["s",1,true]}'), "")).toEqual([
      ["punctuation", "{"],
      ["key", '"k"'],
      ["punctuation", ":"],
      ["punctuation", "["],
      ["string", '"s"'],
      ["punctuation", ","],
      ["number", "1"],
      ["punctuation", ","],
      ["literal", "true"],
      ["punctuation", "]"],
      ["punctuation", "}"],
    ]);
  });
});
