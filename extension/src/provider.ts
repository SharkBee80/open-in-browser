import { watchDebounced } from "@vueuse/core";
import { ref } from "vue";

export type BrowserConfig = {
  label: string;
  path: string;
  id: number;
  args?: string;
  show?: boolean;
  is?: boolean;
};

type Config = {
  key: {
    port: number;
    secret: string;
  };
  browser: BrowserConfig[];
};

export const DEFAULT_BROWSER: BrowserConfig[] = [
  {
    id: 1,
    label: "•edge•",
    path: "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
    is: true,
  },
  {
    id: 2,
    label: "•chrome•",
    path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    is: true,
  },
];

export const DEFAULT_KEY = {
  port: 52798,
  secret: "open-in-browser",
};

const DEFAULT_CONFIG: Config = {
  key: DEFAULT_KEY,
  browser: DEFAULT_BROWSER,
};

export const config = ref<Config>(
  Object.assign(
    DEFAULT_CONFIG,
    JSON.parse(localStorage.getItem("app-config") || "[]"),
  ),
);

watchDebounced(
  config.value,
  (config) => {
    localStorage.setItem("app-config", JSON.stringify(config));
  },
  { debounce: 500, deep: true, immediate: true },
);
