// Switch a running AIRI instance between its Live2D default and the VRM
// stage (AvatarSample_A) over the Chrome DevTools Protocol — the
// apples-to-apples reference for pocket-character. AIRI must have been
// launched with remote debugging enabled:
//
//   /Applications/AIRI.app/Contents/MacOS/AIRI --remote-debugging-port=9222 &
//
// Usage:
//   bun scripts/airi-vrm.ts                    # -> preset-vrm-1 (AvatarSample_A)
//   bun scripts/airi-vrm.ts preset-live2d-1    # back to the Live2D default
//
// The settings store writes the raw string (vueuse useLocalStorage), NOT
// JSON — localStorage is origin-shared across AIRI's windows, so setting it
// in any page and reloading is enough.
export {};

const preset = process.argv[2] ?? "preset-vrm-1";
const port = process.env.AIRI_CDP_PORT ?? "9222";

const pages: Array<{ type: string; url: string; webSocketDebuggerUrl: string }> =
  await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
const targets = pages.filter((p) => p.type === "page");
if (targets.length === 0) throw new Error("no AIRI pages found — launched with --remote-debugging-port?");

for (const page of targets) {
  const ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    ws.onopen = resolve;
    ws.onerror = reject;
  });
  const send = (id: number, method: string, params: object) =>
    new Promise<void>((resolve) => {
      ws.onmessage = (ev) => {
        if (JSON.parse(String(ev.data)).id === id) resolve();
      };
      ws.send(JSON.stringify({ id, method, params }));
    });
  await send(1, "Runtime.evaluate", {
    expression: `localStorage.setItem('settings/stage/model', '${preset}'); location.reload()`,
  });
  ws.close();
  console.log(`set ${preset} + reloaded: ${page.url}`);
}
console.log("give the VRM scene ~60s to load before measuring or screenshotting");
