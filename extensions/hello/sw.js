// A service worker Hello tries to register (hello.js, checkServiceWorker) — and must not be able
// to: Little Joan refuses a service worker's script, or the engine has no service workers for
// extensions. If one ever ran, it would only pass requests through.
self.addEventListener("fetch", () => {});
