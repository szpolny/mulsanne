<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { onMount } from "svelte";
  import { loadConfig } from "../lib/config.svelte";
  import { watchUploads } from "../lib/uploads.svelte";
  import { live, startLive } from "../lib/live.svelte";
  import Logo from "../lib/Logo.svelte";
  import { initStarPrompt } from "../lib/star.svelte";
  import { initUpdater, installUpdate, updater } from "../lib/updater.svelte";
  import { nav, type Page } from "./nav.svelte";
  import ChangelogDialog from "./components/ChangelogDialog.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import StarPrompt from "./components/StarPrompt.svelte";
  import Analysis from "./pages/Analysis.svelte";
  import Live from "./pages/Live.svelte";
  import Overlays from "./pages/Overlays.svelte";
  import Rankings from "./pages/Rankings.svelte";
  import Records from "./pages/Records.svelte";
  import Schedule from "./pages/Schedule.svelte";
  import Sessions from "./pages/Sessions.svelte";
  import Settings from "./pages/Settings.svelte";

  let maximized = $state(false);

  // Sidebar can shrink to icons only; remembered per machine.
  const SIDEBAR_KEY = "mulsanne.sidebarCollapsed";
  let collapsed = $state(readCollapsed());
  function readCollapsed() {
    try {
      return localStorage.getItem(SIDEBAR_KEY) === "1";
    } catch {
      return false;
    }
  }
  function toggleSidebar() {
    collapsed = !collapsed;
    try {
      localStorage.setItem(SIDEBAR_KEY, collapsed ? "1" : "0");
    } catch {
      // Not persisted; still toggles for this run.
    }
  }
  let reportIssueError = $state(false);
  const ISSUE_URL = "https://github.com/szpolny/mulsanne/issues/new/choose";

  async function reportIssue(event: MouseEvent) {
    if (!("__TAURI_INTERNALS__" in window)) return;
    event.preventDefault();
    reportIssueError = false;
    try {
      await openUrl(ISSUE_URL);
    } catch {
      reportIssueError = true;
    }
  }

  onMount(() => {
    startLive();
    void loadConfig().then(initUpdater).then(initStarPrompt);
    watchUploads();

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
    { id: "rankings", label: "Rankings", icon: "M4 20h16 M6 20v-6h4v6 M10 20V8h4v12 M14 20v-9h4v9" },
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
    <nav
      class={[
        "flex shrink-0 flex-col overflow-hidden border-r border-line bg-surface transition-[width] duration-150",
        collapsed ? "w-[60px]" : "w-[220px]",
      ]}
    >
      <div class="flex flex-col gap-0.5 px-2 pt-3">
        {#each PAGES as p}
          <button
            class={[
              "relative flex items-center gap-3 overflow-hidden rounded-md px-3 py-2 text-left font-display text-[15px] font-semibold tracking-[0.08em] uppercase transition-colors",
              nav.page === p.id
                ? "bg-surface-3 text-fg before:absolute before:top-2 before:bottom-2 before:left-0 before:w-[3px] before:rounded-full before:bg-accent"
                : "text-muted hover:bg-surface-2 hover:text-fg",
            ]}
            title={collapsed ? p.label : undefined}
            aria-label={collapsed ? p.label : undefined}
            aria-current={nav.page === p.id ? "page" : undefined}
            onclick={() => (nav.page = p.id)}
          >
            <svg class="shrink-0" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d={p.icon} /></svg>
            <span class={["whitespace-nowrap transition-opacity duration-150", collapsed && "opacity-0"]}>{p.label}</span>
          </button>
        {/each}
      </div>
      <div class="mt-auto">
        <button
          class={[
            "mx-2 mb-1 flex items-center gap-3 rounded-md py-2 font-display text-[15px] font-semibold tracking-[0.08em] whitespace-nowrap text-muted uppercase transition-colors hover:bg-surface-2 hover:text-fg",
            "w-[calc(100%-1rem)] overflow-hidden px-3",
          ]}
          title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
          aria-expanded={!collapsed}
          onclick={toggleSidebar}
        >
          <svg class="shrink-0" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"
            ><path d="M4 5h16v14H4z M9 5v14" /><path d={collapsed ? "m13 10 2 2-2 2" : "m16 10-2 2 2 2"} /></svg
          >
          <span class={["whitespace-nowrap transition-opacity duration-150", collapsed && "opacity-0"]}>Collapse</span>
        </button>
        <a
          class={[
            "mx-2 mb-2 flex items-center gap-3 rounded-md py-2 font-display text-[15px] font-semibold tracking-[0.08em] text-muted uppercase transition-colors hover:bg-surface-2 hover:text-fg focus-visible:outline-2 focus-visible:outline-accent",
            "overflow-hidden px-3",
          ]}
          href={ISSUE_URL}
          target="_blank"
          rel="noopener noreferrer"
          title="Report an issue on GitHub (opens in your browser)"
          aria-label={collapsed ? "Report issue" : undefined}
          onclick={reportIssue}
        >
          <svg class="shrink-0" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="9" /><path d="M12 8v4 M12 16h.01" /></svg>
          <span class={["whitespace-nowrap transition-opacity duration-150", collapsed && "opacity-0"]}>Report issue</span>
          <svg class={["ml-auto shrink-0 transition-opacity duration-150", collapsed && "opacity-0"]} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 3h7v7 M21 3 10 14 M10 3H3v18h18v-7" /></svg>
        </a>
        {#if reportIssueError && !collapsed}
          <p class="mx-4 mb-3 select-text break-words text-xs text-bad" role="alert">
            Couldn't open your browser. Visit {ISSUE_URL} to report an issue.
          </p>
        {/if}
        {#if collapsed && (updater.phase === "available" || updater.phase === "downloading" || updater.phase === "outdated")}
          <button
            class="relative mx-2 mb-2 flex w-[calc(100%-1rem)] rounded-md px-3 py-2 text-accent-hi transition-colors hover:bg-surface-2"
            title={updater.phase === "outdated" ? `Version ${updater.available} is out. Update Mulsanne with your package manager.` : `Update to ${updater.available}`}
            aria-label="Update available"
            onclick={toggleSidebar}
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 4v11 M7 10l5 5 5-5 M5 20h14" /></svg>
            <span class="absolute top-1.5 left-[30px] h-1.5 w-1.5 rounded-full bg-accent"></span>
          </button>
        {:else if updater.phase === "available" || updater.phase === "downloading"}
          <div class="mx-2 mb-2 rounded-md border border-line border-t-accent border-t-2 bg-surface-2 p-3 text-xs">
            <div class="eyebrow text-[11px] tracking-[0.16em] text-fg">Update available</div>
            <div class="mt-1 text-muted">Version {updater.available} is ready to install.</div>
            <button class="btn btn-primary mt-2 w-full" disabled={updater.phase === "downloading"} onclick={installUpdate}>
              {updater.phase === "downloading" ? `Downloading ${Math.round(updater.progress * 100)}%` : "Install and restart"}
            </button>
          </div>
        {/if}
        {#if updater.phase === "outdated" && !collapsed}
          <div class="mx-2 mb-2 rounded-md border border-line border-t-accent border-t-2 bg-surface-2 p-3 text-xs">
            <div class="eyebrow text-[11px] tracking-[0.16em] text-fg">Update available</div>
            <div class="mt-1 text-muted">Version {updater.available} is out. Update Mulsanne with your package manager.</div>
          </div>
        {/if}
        <div class="overflow-hidden border-t border-line px-[26px] py-3 text-xs" title={collapsed ? statusText : undefined}>
          <div class="flex items-center gap-2">
            <span class={["h-2 w-2 shrink-0 rounded-full", statusColor]}></span>
            <span class={["truncate text-muted transition-opacity duration-150", collapsed && "opacity-0"]} title={statusText}>{statusText}</span>
          </div>
        </div>
      </div>
    </nav>

    <main
      class="min-w-0 flex-1 overflow-y-auto bg-[radial-gradient(900px_360px_at_50%_0%,rgb(227_16_31/0.08),transparent_70%)]"
    >
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
      {:else if nav.page === "rankings"}
        <Rankings />
      {:else if nav.page === "schedule"}
        <Schedule />
      {:else}
        <Settings />
      {/if}
    </main>
  </div>
</div>

<ChangelogDialog />
<ConfirmDialog />
<StarPrompt />
