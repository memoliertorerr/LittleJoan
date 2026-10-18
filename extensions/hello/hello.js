// @ts-check
// Hello: the example extension (and the CI smoke test's target). It takes Little Joan's colors,
// counts how often it was opened with littleJoan.storage, and checks what a panel can't do: call
// Little Joan's own commands (the page webview has no IPC), reach the network (its manifest
// declares no hosts, so the Content-Security-Policy blocks every fetch), or install a service
// worker. The results go into the page and into its title,
// `LJ-EXT hello storage=… theme=… ipc=… net=… sw=…`, which CI reads.

/** @param {number} ms */
const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

/** Opens, counts and reads back; sets and removes a scratch key. */
async function checkStorage() {
  const before = await littleJoan.storage.get("opened");
  const count = (typeof before === "number" ? before : 0) + 1;
  await littleJoan.storage.set("opened", count);
  const again = await littleJoan.storage.get("opened");
  const keys = await littleJoan.storage.keys();
  await littleJoan.storage.set("scratch", { at: Date.now() });
  const removed = await littleJoan.storage.remove("scratch");
  const missing = await littleJoan.storage.get("scratch");
  const countText = document.getElementById("count");
  if (countText) {
    countText.textContent = count === 1 ? "This is the first time you opened me." : `You opened me ${count} times.`;
  }
  return again === count && keys.includes("opened") && removed && missing === undefined ? "ok" : "wrong";
}

/** Little Joan's colors, on this page. */
async function checkTheme() {
  const theme = await littleJoan.theme.apply();
  return (theme.scheme === "light" || theme.scheme === "dark") && typeof theme.vars["--lj-bg-0"] === "string" ? "ok" : "wrong";
}

/** Tauri's IPC object may be there (the engine injects it), but every command must be refused. */
async function checkIpc() {
  const internals = /** @type {unknown} */ (Reflect.get(window, "__TAURI_INTERNALS__"));
  const invoke = typeof internals === "object" && internals !== null ? /** @type {unknown} */ (Reflect.get(internals, "invoke")) : undefined;
  if (typeof invoke !== "function") {
    return "denied";
  }
  const timeout = sleep(4000).then(() => "timeout");
  const call = Promise.resolve(Reflect.apply(invoke, internals, ["start_info"])).then(
    () => "open",
    () => "denied",
  );
  return /** @type {string} */ (await Promise.race([call, timeout]));
}

/** A fetch to a host the manifest doesn't declare: the CSP must block it (and say so). */
async function checkNet() {
  let violated = false;
  document.addEventListener("securitypolicyviolation", (event) => {
    const directive = event.effectiveDirective || event.violatedDirective;
    // Only this fetch's report counts (the engine's own blocked requests report too).
    if (directive.startsWith("connect-src") && event.blockedURI.startsWith("https://example.com")) {
      violated = true;
    }
  });
  try {
    await fetch("https://example.com/", { mode: "no-cors", cache: "no-store" });
    return "open";
  } catch {
    // The violation event comes as its own task, shortly after.
    for (let waited = 0; waited < 1500 && !violated; waited += 100) {
      await sleep(100);
    }
    return violated ? "blocked" : "failed";
  }
}

/**
 * A service worker would answer this extension's requests without Little Joan's checks and
 * outlive the panel, so registering one (sw.js is a real file here) must fail: Little Joan
 * refuses its script ("refused"), or the engine offers no service workers to an extension at all
 * ("unsupported"). Should one ever register, it is unregistered at once and reported.
 */
async function checkServiceWorker() {
  if (!("serviceWorker" in navigator)) {
    return "unsupported";
  }
  const timeout = sleep(5000).then(() => "timeout");
  const attempt = navigator.serviceWorker.register("sw.js").then(
    async (registration) => {
      await registration.unregister().catch(() => false);
      return "registered";
    },
    /** @param {unknown} error */
    (error) =>
      error instanceof DOMException && (error.name === "SecurityError" || error.name === "NotSupportedError")
        ? "unsupported"
        : "refused",
  );
  return /** @type {string} */ (await Promise.race([attempt, timeout]));
}

/**
 * @param {() => Promise<string>} check
 * @returns {Promise<string>}
 */
async function run(check) {
  try {
    return await check();
  } catch (error) {
    return error instanceof Error && "code" in error ? `error-${String(error.code)}` : "error";
  }
}

const LABELS = {
  storage: "Keep its own data (storage)",
  theme: "Use Little Joan's colors",
  ipc: "Call into Little Joan itself",
  net: "Reach the internet (no hosts declared)",
  sw: "Install a service worker",
};

async function main() {
  const results = {
    storage: await run(checkStorage),
    theme: await run(checkTheme),
    ipc: await run(checkIpc),
    net: await run(checkNet),
    sw: await run(checkServiceWorker),
  };
  const list = document.getElementById("checks");
  for (const [key, label] of Object.entries(LABELS)) {
    const item = document.createElement("li");
    const name = document.createElement("span");
    name.textContent = label;
    const result = document.createElement("span");
    result.className = "result";
    result.textContent = results[/** @type {keyof typeof results} */ (key)];
    item.append(name, result);
    list?.append(item);
  }
  document.title = `LJ-EXT hello storage=${results.storage} theme=${results.theme} ipc=${results.ipc} net=${results.net} sw=${results.sw}`;
}

document.getElementById("close")?.addEventListener("click", () => {
  void littleJoan.close();
});
void main();
