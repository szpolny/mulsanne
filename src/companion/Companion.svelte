<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { loadConfig } from "../lib/config.svelte";
  import { live, startLive } from "../lib/live.svelte";
  import Logo from "../lib/Logo.svelte";
  import { initUpdater, installUpdate, updater } from "../lib/updater.svelte";
  import { nav, type Page } from "./nav.svelte";
  import Analysis from "./pages/Analysis.svelte";
  import Live from "./pages/Live.svelte";
  import Overlays from "./pages/Overlays.svelte";
  import Records from "./pages/Records.svelte";
  import Schedule from "./pages/Schedule.svelte";
  import Sessions from "./pages/Sessions.svelte";
  import Settings from "./pages/Settings.svelte";

  let maximized = $state(false);

  onMount(() => {
    startLive();
    loadConfig();
    void initUpdater();

    if (!("__TAURI_INTERNALS__" in window)) return;

    const appWindow = getCurrentWindow();
    void appWindow.isMaximized().then((value) => (maximized = value));
    let unlistenResize: (() => void) | undefined;
    void appWindow.onResized(() => {
      void appWindow.isMaximized().then((value) => (maximized = value));
    }).then((unlisten) => (unlistenResize = unlisten));

    return () => unlistenResize?.();
  });

  function startDragging(event: MouseEvent) {
    if (event.button === 0 && "__TAURI_INTERNALS__" in window) {
      void getCurrentWindow().startDragging();
    }
  }

  async function toggleMaximize() {
    if (!("__TAURI_INTERNALS__" in window)) return;
    const appWindow = getCurrentWindow();
    await appWindow.toggleMaximize();
    maximized = await appWindow.isMaximized();
  }

  const PAGES: { id: Page; label: string; icon: string }[] = [
    { id: "live", label: "Live", icon: "M3 12h4l3-8 4 16 3-8h4" },
    { id: "overlays", label: "Overlays", icon: "M4 5h16v14H4z M8 9h5 M8 13h8" },
    { id: "sessions", label: "Sessions", icon: "M4 6h16 M4 12h16 M4 18h10" },
    { id: "analysis", label: "Analysis", icon: "M4 19V5 M4 19h16 M7 15l4-5 3 3 5-7" },
    { id: "records", label: "Records", icon: "M8 21h8 M12 17v4 M7 4h10v5a5 5 0 0 1-10 0z M17 6h3v2a3 3 0 0 1-3 3 M7 6H4v2a3 3 0 0 0 3 3" },
    { id: "schedule", label: "Schedule", icon: "M4 6h16v14H4z M4 10h16 M8 3v4 M16 3v4 M12 13v3l2 1" },
    { id: "settings", label: "Settings", icon: "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M19 12h2 M3 12h2 M12 3v2 M12 19v2 M17 7l1.5-1.5 M5.5 18.5 7 17 M17 17l1.5 1.5 M5.5 5.5 7 7" },
  ];

  const snap = $derived(live.snap);
  const statusText = $derived(
    !snap || snap.source === "none"
      ? "Waiting for Le Mans Ultimate"
      : snap.source === "demo"
        ? "Demo data"
        : snap.player
          ? `${snap.session.track}`
          : "Connected · in menus",
  );
  const statusColor = $derived(
    !snap || snap.source === "none" ? "bg-faint" : snap.source === "demo" ? "bg-warn" : "bg-good",
  );
</script>

<div class="flex h-full flex-col">
  <header class="flex h-11 shrink-0 items-stretch border-b border-line bg-bg">
    <div
      class="flex min-w-0 flex-1 cursor-default items-center gap-2.5 px-4"
      onmousedown={startDragging}
      ondblclick={toggleMaximize}
    >
      <Logo size={22} tagline />
    </div>
    <div class="flex shrink-0 items-stretch">
      <button
        class="flex w-12 items-center justify-center text-muted transition-colors hover:bg-surface-3 hover:text-fg"
        aria-label="Minimize window"
        title="Minimize"
        onclick={() => {
          if ("__TAURI_INTERNALS__" in window) void getCurrentWindow().minimize();
        }}
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true"><path d="M3 7h8" stroke="currentColor" stroke-width="1.2" /></svg>
      </button>
      <button
        class="flex w-12 items-center justify-center text-muted transition-colors hover:bg-surface-3 hover:text-fg"
        aria-label={maximized ? "Restore window" : "Maximize window"}
        title={maximized ? "Restore" : "Maximize"}
        onclick={toggleMaximize}
      >
        {#if maximized}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true"><path d="M4.5 3.5h6v6h-6zM3.5 5.5v5h5" stroke="currentColor" stroke-width="1.1" /></svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true"><rect x="3.5" y="3.5" width="7" height="7" stroke="currentColor" stroke-width="1.1" /></svg>
        {/if}
      </button>
      <button
        class="flex w-12 items-center justify-center text-muted transition-colors hover:bg-red-600 hover:text-white"
        aria-label="Close window"
        title="Close"
        onclick={() => {
          if ("__TAURI_INTERNALS__" in window) void getCurrentWindow().close();
        }}
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true"><path d="m3.5 3.5 7 7m0-7-7 7" stroke="currentColor" stroke-width="1.2" /></svg>
      </button>
    </div>
  </header>

  <div class="flex min-h-0 flex-1">
    <nav class="flex w-[220px] shrink-0 flex-col border-r border-line bg-surface">
      <div class="flex flex-col gap-0.5 px-2 pt-4">
        {#each PAGES as p}
          <button
            class={[
              "relative flex items-center gap-3 rounded-lg px-3 py-2 text-left text-[14px] font-medium transition-colors",
              nav.page === p.id
                ? "bg-surface-3 text-fg before:absolute before:top-2 before:bottom-2 before:left-0 before:w-[3px] before:rounded-full before:bg-accent"
                : "text-muted hover:bg-surface-2 hover:text-fg",
            ]}
            onclick={() => (nav.page = p.id)}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d={p.icon} /></svg>
            {p.label}
          </button>
        {/each}
      </div>
      <div class="mt-auto">
        {#if updater.phase === "available" || updater.phase === "downloading"}
          <div class="mx-2 mb-2 rounded-lg border border-line bg-surface-2 p-3 text-xs">
            <div class="font-medium text-fg">Update available</div>
            <div class="mt-0.5 text-muted">Version {updater.available} is ready to install.</div>
            <button class="btn btn-primary mt-2 w-full" disabled={updater.phase === "downloading"} onclick={installUpdate}>
              {updater.phase === "downloading" ? `Downloading ${Math.round(updater.progress * 100)}%` : "Install and restart"}
            </button>
          </div>
        {/if}
        <div class="border-t border-line px-4 py-3 text-xs">
          <div class="flex items-center gap-2">
            <span class={["h-2 w-2 rounded-full", statusColor]}></span>
            <span class="truncate text-muted" title={statusText}>{statusText}</span>
          </div>
        </div>
      </div>
    </nav>

    <main class="min-w-0 flex-1 overflow-y-auto">
      {#if nav.page === "live"}
        <Live />
      {:else if nav.page === "overlays"}
        <Overlays />
      {:else if nav.page === "sessions"}
        <Sessions />
      {:else if nav.page === "analysis"}
        <Analysis />
      {:else if nav.page === "records"}
        <Records />
      {:else if nav.page === "schedule"}
        <Schedule />
      {:else}
        <Settings />
      {/if}
    </main>
  </div>
</div>
