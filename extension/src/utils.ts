import { encodeKey, signToken } from "@/assets/js/encrypt";
import { config } from "@/provider";

const AUTH_HEADER = "x-openinbrowser-auth";
const JWT_TTL_SECONDS = 3;

export async function generateAuthToken() {
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    iat: now,
    exp: now + JWT_TTL_SECONDS,
  };
  const header = { alg: "HS256", typ: "JWT" };
  const key = encodeKey(config.value.key.secret);
  const token = await signToken(payload, key, header);

  return { [AUTH_HEADER]: token };
}

function isBlockedTabUrl(url: string): boolean {
  // Block internal / restricted pages and "empty" tabs.
  // These URLs are either inaccessible to extensions or not meaningful to open externally.
  const blockedPrefixes = [
    "chrome://",
    "edge://",
    "about:",
    "chrome-extension://",
    "moz-extension://",
  ];
  if (blockedPrefixes.some((p) => url.startsWith(p))) return true;

  // Common new tab pages (varies by browser / locale / policies)
  if (url === "chrome://newtab/" || url === "edge://newtab/") return true;

  return false;
}

async function getActiveTabUrl(): Promise<string | null> {
  // In a popup/action context, `activeTab` permission should allow reading the
  // current tab URL after user interaction. Use a callback wrapper for maximum
  // compatibility across Chromium versions.
  const tabs = await new Promise<chrome.tabs.Tab[]>((resolve, reject) => {
    try {
      chrome.tabs.query({ active: true, currentWindow: true }, (result) => {
        const err = chrome.runtime.lastError;
        if (err) return reject(new Error(err.message));
        resolve(result);
      });
    } catch (e) {
      reject(e);
    }
  });

  const url = tabs?.[0]?.url;
  if (typeof url !== "string" || url.length === 0) {
    console.log("No active tab");
    return null;
  }
  if (isBlockedTabUrl(url)) {
    console.log("Blocked tab URL");
    return null;
  }

  return url;
}

export async function send(b: string, args?: string) {
  const api = new URL("http://localhost");
  api.port = String(config.value.key.port);
  api.pathname = "/cmd";

  const url = await getActiveTabUrl();
  if (!url) return;

  // 构建命令字符串
  let cmd = b + " " + url;
  // 添加额外参数
  if (args) {
    cmd += ` ${args}`;
  }

  const authHeader = await generateAuthToken();
  return await fetch(api.href, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...authHeader,
    },
    body: JSON.stringify([cmd]),
  });
}
