<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { api } from "../../lib/api";
  import { config } from "../../lib/config.svelte";
  import type { ShareStatus } from "../../lib/types";
  import { nav } from "../nav.svelte";

  /** Shares live timing through a link that opens on mulsanne.xyz. */

  let share = $state<ShareStatus | null>(null);
  let busy = $state(false);
  let error = $state("");
  let copied = $state(false);

  const enabled = $derived(config.value.online.enabled);

  onMount(async () => {
    share = await api.liveShareStatus().catch(() => null);
  });
  const unlisten = listen<ShareStatus | null>("live-share-changed", (e) => (share = e.payload));
  onDestroy(() => unlisten.then((f) => f()));

  async function act(action: () => Promise<unknown>) {
    busy = true;
    error = "";
    try {
      await action();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const start = () => act(async () => (share = await api.liveShareStart()));
  const stop = () => act(async () => { await api.liveShareStop(); share = null; });

  async function copy() {
    if (!share) return;
    await navigator.clipboard.writeText(share.url);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  const viewers = $derived(share ? (share.viewers === 1 ? "1 viewer" : `${share.viewers} viewers`) : "");
</script>

<section class="flex flex-col gap-3 card p-4 text-sm">
  <div class="flex items-center justify-between">
    <div class="section-title text-sm text-muted">Share live timing</div>
    {#if share && !share.ended}
      <span class="flex items-center gap-1.5 text-xs font-semibold text-bad">
        <span class="size-2 animate-pulse rounded-full bg-bad"></span>LIVE
      </span>
    {/if}
  </div>

  {#if !enabled}
    <p class="text-xs leading-relaxed text-muted">
      Send a link so others can follow your session in a browser: standings, gaps, positions on the track map and your
      car's fuel and tyres. Needs online features.
    </p>
    <button class="btn" onclick={() => (nav.page = "settings")}>Turn on in Settings</button>
  {:else if !share}
    <p class="text-xs leading-relaxed text-muted">
      Anyone with the link sees the standings with every driver's in-game name, gaps and positions on the map, your
      car's fuel, energy and tyres, and the weather. It updates once a second until you stop.
    </p>
    <button class="btn btn-primary" disabled={busy} onclick={start}>{busy ? "Starting…" : "Start sharing"}</button>
  {:else}
    <div class="flex items-center gap-1 rounded-lg bg-surface-2 py-1 pr-1 pl-3">
      <span class="min-w-0 flex-1 truncate font-mono text-xs" title={share.url}>{share.url.replace("https://", "")}</span>
      <button class="rounded px-2 py-1 text-xs text-accent hover:bg-surface-3" onclick={copy}>{copied ? "Copied" : "Copy"}</button>
      <button class="rounded px-2 py-1 text-xs text-accent hover:bg-surface-3" onclick={() => act(api.liveShareOpen)}>Open</button>
    </div>
    {#if share.ended || share.error}
      <p class="text-xs text-bad">{share.error}</p>
    {:else}
      <p class="text-xs text-muted">{viewers} watching</p>
    {/if}
    {#if share.ended}
      <button class="btn btn-primary" disabled={busy} onclick={start}>Start a new link</button>
    {:else}
      <button class="btn" disabled={busy} onclick={stop}>Stop sharing</button>
    {/if}
  {/if}
  {#if error}
    <p class="text-xs text-bad">{error}</p>
  {/if}
</section>
