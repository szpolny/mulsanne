// `VITE_DEMO=1 pnpm tauri dev` swaps this in for `@tauri-apps/api/core` (see vite.config.ts) so the real
// app window shows the browser preview's demo data. Window controls and plugins still reach Tauri.
import * as real from "../../../node_modules/@tauri-apps/api/core.js";
export * from "../../../node_modules/@tauri-apps/api/core.js";

const PASSTHROUGH = /^plugin:(window|webview|event|app|opener|process|updater)\|/;

export async function invoke<T>(cmd: string, args?: real.InvokeArgs, options?: real.InvokeOptions): Promise<T> {
  const demo = (globalThis as any).__MULSANNE_DEMO_IPC__ as ((cmd: string, args: unknown) => unknown) | undefined;
  if (demo && !PASSTHROUGH.test(cmd)) return (await demo(cmd, args ?? {})) as T;
  return real.invoke<T>(cmd, args, options);
}
