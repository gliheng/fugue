import { useSyncExternalStore } from "react";

export type DashboardTheme = "dark" | "light";

function getTheme(): DashboardTheme {
  if (typeof document === "undefined") return "dark";
  const attr = document.documentElement.dataset.theme;
  if (attr === "light") return "light";
  if (attr === "dark") return "dark";
  if (window.matchMedia("(prefers-color-scheme: light)").matches) return "light";
  return "dark";
}

function subscribe(callback: () => void) {
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "attributes" && mutation.attributeName === "data-theme") {
        callback();
      }
    }
  });
  observer.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });

  const media = window.matchMedia("(prefers-color-scheme: light)");
  media.addEventListener("change", callback);

  return () => {
    observer.disconnect();
    media.removeEventListener("change", callback);
  };
}

export function useDashboardTheme(): DashboardTheme {
  return useSyncExternalStore(
    subscribe,
    getTheme,
    () => "dark",
  );
}

export function toMonacoTheme(theme: DashboardTheme): "vs-dark" | "vs" | "hc-black" {
  switch (theme) {
    case "light":
      return "vs";
    case "dark":
    default:
      return "vs-dark";
  }
}
