// Swapped in for `@tauri-apps/api/event` with `VITE_DEMO=1` (see core.ts): app events stay inside this
// window so the demo telemetry isn't mixed with the real backend's.
import * as real from "../../../node_modules/@tauri-apps/api/event.js";
export * from "../../../node_modules/@tauri-apps/api/event.js";

const listeners = new Map<string, Set<real.EventCallback<any>>>();
const demo = () => "__MULSANNE_DEMO_IPC__" in globalThis;

export async function listen<T>(event: string, handler: real.EventCallback<T>, options?: real.Options): Promise<real.UnlistenFn> {
  if (!demo()) return real.listen(event, handler, options);
  if (!listeners.has(event)) listeners.set(event, new Set());
  listeners.get(event)!.add(handler);
  return () => void listeners.get(event)?.delete(handler);
}

export async function emit<T>(event: string, payload?: T): Promise<void> {
  if (!demo()) return real.emit(event, payload);
  for (const h of listeners.get(event) ?? []) h({ event, id: 0, payload: payload as T });
}
