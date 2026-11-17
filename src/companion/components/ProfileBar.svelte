<script lang="ts">
  import Select from "./Select.svelte";
  import { config, createProfile, deleteProfile, renameProfile, switchProfile } from "../../lib/config.svelte";

  type Mode = { kind: "idle" } | { kind: "rename" } | { kind: "new"; fromDefaults: boolean } | { kind: "delete" };

  let mode = $state<Mode>({ kind: "idle" });
  let name = $state("");
  let input = $state<HTMLInputElement>();
  let menuOpen = $state(false);
  let menu = $state<HTMLDivElement>();

  const active = $derived(config.value.activeProfile);
  const profiles = $derived(Object.entries(config.value.profiles));
  const activeName = $derived(config.value.profiles[active]?.name ?? "");

  function start(m: Mode) {
    menuOpen = false;
    mode = m;
    name = m.kind === "rename" ? activeName : m.kind === "new" ? `${activeName} copy` : "";
    if (m.kind === "new" && m.fromDefaults) name = "New profile";
    queueMicrotask(() => input?.select());
  }

  function commit() {
    const n = name.trim();
    if (!n) return;
    if (mode.kind === "rename") renameProfile(active, n);
    else if (mode.kind === "new") createProfile(n, mode.fromDefaults);
    mode = { kind: "idle" };
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter") commit();
    else if (e.key === "Escape") mode = { kind: "idle" };
  }

  function onWindowClick(e: MouseEvent) {
    if (menuOpen && menu && !menu.contains(e.target as Node)) menuOpen = false;
  }
</script>

<svelte:window onclick={onWindowClick} onkeydown={(e) => e.key === "Escape" && (menuOpen = false)} />

<div class="flex flex-wrap items-center gap-2 text-sm">
  {#if mode.kind === "rename" || mode.kind === "new"}
    <input
      bind:this={input}
      bind:value={name}
      onkeydown={onKey}
      class="field w-52"
      placeholder="Profile name"
      maxlength="40"
      title={mode.kind === "new"
        ? mode.fromDefaults
          ? "Starts from the default layout."
          : `Starts as a copy of “${activeName}”.`
        : undefined}
    />
    <button class="btn btn-primary" disabled={!name.trim()} onclick={commit}>
      {mode.kind === "rename" ? "Rename" : "Create"}
    </button>
    <button class="btn" onclick={() => (mode = { kind: "idle" })}>Cancel</button>
  {:else if mode.kind === "delete"}
    <span>Delete “{activeName}”?</span>
    <button
      class="btn bg-bad! text-white"
      onclick={() => {
        deleteProfile(active);
        mode = { kind: "idle" };
      }}>Delete</button
    >
    <button class="btn" onclick={() => (mode = { kind: "idle" })}>Cancel</button>
  {:else}
    <Select
      class="w-52"
      value={active}
      label="Profile"
      title="Layout, widget options, sizes and opacity, and global scale and opacity are saved per profile."
      options={profiles.map(([id, p]) => ({ value: id, label: p.name }))}
      onchange={switchProfile}
    />
    <div class="relative" bind:this={menu}>
      <button class="btn" aria-haspopup="menu" aria-expanded={menuOpen} onclick={() => (menuOpen = !menuOpen)}>
        Manage
        <svg viewBox="0 0 12 12" class="h-3 w-3" aria-hidden="true"><path d="M2.5 4.5 6 8l3.5-3.5" fill="none" stroke="currentColor" stroke-width="1.6" /></svg>
      </button>
      {#if menuOpen}
        <div
          role="menu"
          class="absolute top-full left-0 z-20 mt-1 flex w-48 flex-col rounded-md border border-line bg-surface-2 p-1 shadow-2xl"
        >
          <button role="menuitem" class="menu-item" onclick={() => start({ kind: "new", fromDefaults: false })}>Duplicate</button>
          <button role="menuitem" class="menu-item" onclick={() => start({ kind: "new", fromDefaults: true })}>New from defaults</button>
          <button role="menuitem" class="menu-item" onclick={() => start({ kind: "rename" })}>Rename</button>
          <button
            role="menuitem"
            class="menu-item text-bad"
            disabled={profiles.length < 2}
            onclick={() => start({ kind: "delete" })}>Delete</button
          >
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .menu-item {
    border-radius: 4px;
    padding: 6px 10px;
    text-align: left;
  }
  .menu-item:hover:not(:disabled) {
    background: var(--color-surface-3);
  }
  .menu-item:disabled {
    opacity: 0.4;
  }
</style>
