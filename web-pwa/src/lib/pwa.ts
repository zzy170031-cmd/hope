export function registerServiceWorker(): void {
  if (!("serviceWorker" in navigator)) {
    return;
  }

  const isLocalPreview =
    window.location.hostname === "127.0.0.1" ||
    window.location.hostname === "localhost";

  if (!import.meta.env.PROD || isLocalPreview) {
    navigator.serviceWorker.getRegistrations().then((registrations) => {
      registrations.forEach((registration) => {
        registration.unregister().catch(() => undefined);
      });
    }).catch(() => undefined);
    if ("caches" in window) {
      caches.keys().then((keys) => {
        keys
          .filter((key) => key.startsWith("hope-web-pwa-"))
          .forEach((key) => {
            caches.delete(key).catch(() => undefined);
          });
      }).catch(() => undefined);
    }
    return;
  }

  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch(() => undefined);
  });
}
