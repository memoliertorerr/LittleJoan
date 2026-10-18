// Types of the Little Joan extension SDK, version 1 (extensions/sdk/v1.js; experimental), for
// editors and type checking. A panel loads the SDK with <script src="/__lj/sdk/v1.js"></script>,
// which defines the global `littleJoan`. Every call returns a promise and rejects with a
// LittleJoanError (its `code` says why: "permission", "quota", "bad-key", …) when Little Joan
// refuses or can't do it.

declare namespace LittleJoan {
  interface LittleJoanError extends Error {
    readonly code: string;
  }

  /** The bar's colors right now: `scheme` and the `--lj-*` CSS variables (`--lj-bg-0`, `--lj-fg`, …). */
  interface Theme {
    scheme: "light" | "dark";
    vars: Record<string, string>;
  }

  /** The web page a page action was opened for. */
  interface PageInfo {
    url: string;
    title: string;
  }

  interface Api {
    readonly apiVersion: 1;
    readonly LittleJoanError: new (message: string, code: string) => LittleJoanError;
    /** The extension's own storage (permission `storage`): keys of 1–128 characters, JSON values, 1 MB in all. */
    readonly storage: {
      /** The value, or `undefined` when the key isn't set. */
      get(key: string): Promise<unknown>;
      set(key: string, value: unknown): Promise<void>;
      /** Whether there was a value. */
      remove(key: string): Promise<boolean>;
      keys(): Promise<string[]>;
    };
    readonly theme: {
      get(): Promise<Theme>;
      /** Puts the colors on `root` (default: <html>) as CSS variables and sets `color-scheme`. */
      apply(root?: HTMLElement): Promise<Theme>;
    };
    /** Permission `pageInfo`: the page a page action was opened for, or null. */
    pageInfo(): Promise<PageInfo | null>;
    /** Opens an http(s) address in the page (replacing the panel; the default) or the default browser. */
    openUrl(url: string, options?: { where?: "page" | "browser" }): Promise<void>;
    /** Permission `clipboard`. */
    readonly clipboard: {
      writeText(text: string): Promise<void>;
    };
    /** Back to the page the panel was opened from, or to the command bar. */
    close(): Promise<void>;
  }
}

declare const littleJoan: LittleJoan.Api;
