<script lang="ts">
  import { config, saveConfig } from "../../lib/config.svelte";
  import { live } from "../../lib/live.svelte";
  import { checkForUpdate, installUpdate, updater } from "../../lib/updater.svelte";
  import OnlineSettings from "../components/OnlineSettings.svelte";
  import Toggle from "../components/Toggle.svelte";

  const g = $derived(config.value.general);
</script>

<div class="mx-auto max-w-3xl p-6">
  <h1 class="mb-5 page-title">Settings</h1>

  <div class="divide-y divide-line card text-sm">
    <div class="flex items-center justify-between p-4">
      <div>
        <div class="font-medium">Units</div>
        <div class="text-muted">Speed, temperature, pressure and fuel.</div>
      </div>
      <select class="field" bind:value={g.units} onchange={saveConfig}>
        <option value="metric">Metric (km/h, °C, kPa, L)</option>
        <option value="imperial">Imperial (mph, °F, psi, gal)</option>
      </select>
    </div>
    <div class="flex items-center justify-between p-4">
      <div>
        <div class="font-medium">Hide overlay off track</div>
        <div class="text-muted">Only show widgets while driving (always shown in the layout editor).</div>
      </div>
      <Toggle checked={g.onlyOnTrack} label="Hide off track" onchange={(v) => { g.onlyOnTrack = v; saveConfig(); }} />
    </div>
    <div class="flex items-center justify-between p-4">
      <div>
        <div class="font-medium">Hide overlay when game is not focused</div>
        <div class="text-muted">Hide widgets while you alt-tab to another window (always shown in the layout editor and demo mode).</div>
      </div>
      <Toggle checked={g.hideUnfocused} label="Hide when unfocused" onchange={(v) => { g.hideUnfocused = v; saveConfig(); }} />
    </div>
    <div class="flex items-center justify-between gap-6 p-4">
      <div>
        <div class="font-medium">Overlay refresh rate</div>
        <div class="text-muted">Updates per second sent to the widgets. Lower values use less CPU.</div>
      </div>
      <div class="flex items-center gap-3">
        <input type="range" min="10" max="60" step="5" bind:value={g.updateHz} oninput={saveConfig} class="accent-accent" />
        <span class="w-12 text-right tnum">{g.updateHz} Hz</span>
      </div>
    </div>
    <div class="flex items-center justify-between gap-6 p-4">
      <div>
        <div class="font-medium">Layout grid snap</div>
        <div class="text-muted">Pixel step when dragging widgets.</div>
      </div>
      <div class="flex items-center gap-3">
        <input type="range" min="1" max="20" step="1" bind:value={g.snap} oninput={saveConfig} class="accent-accent" />
        <span class="w-12 text-right tnum">{g.snap} px</span>
      </div>
    </div>
    <div class="p-4">
      <div class="font-medium">Data folder</div>
      <div class="mt-1 font-mono text-xs break-all text-muted">{live.status.dataDir}</div>
      <div class="mt-1 text-muted">Laps are stored in <code>laps.db</code> (SQLite) in this folder.</div>
    </div>
    <div class="flex items-center justify-between gap-6 p-4">
      <div>
        <div class="font-medium">Show changelog after updates</div>
        <div class="text-muted">See what's new when you first open a new version.</div>
      </div>
      <Toggle checked={g.showChangelog} label="Show changelog after updates" onchange={(v) => { g.showChangelog = v; saveConfig(); }} />
    </div>
    <div class="flex items-center justify-between gap-6 p-4">
      <div>
        <div class="font-medium">Version {updater.version}</div>
        <div class="text-muted">
          {#if updater.phase === "checking"}
            Checking for updates…
          {:else if updater.phase === "available"}
            Version {updater.available} is available.
          {:else if updater.phase === "downloading"}
            Downloading update… {Math.round(updater.progress * 100)}%
          {:else if updater.phase === "upToDate"}
            You're on the latest version.
          {:else if updater.phase === "error"}
            <span title={updater.error}>Couldn't check for updates.</span>
          {:else}
            Updates are checked when the app starts.
          {/if}
        </div>
      </div>
      <div class="flex shrink-0 flex-wrap justify-end gap-2">
        <button class="btn" disabled={!updater.version} onclick={() => (updater.changelogOpen = true)}>View changelog</button>
        {#if updater.phase === "available"}
          <button class="btn btn-primary" onclick={installUpdate}>Install and restart</button>
        {:else}
          <button class="btn" disabled={updater.phase === "checking" || updater.phase === "downloading"} onclick={checkForUpdate}>
            Check for updates
          </button>
        {/if}
      </div>
    </div>
  </div>

  <h2 class="mt-8 mb-3 section-title">Online</h2>
  <OnlineSettings />
</div>
