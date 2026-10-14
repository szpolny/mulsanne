<script lang="ts">
  import { config, createProfile, deleteProfile, renameProfile, switchProfile } from "../../lib/config.svelte";

  type Mode = { kind: "idle" } | { kind: "rename" } | { kind: "new"; fromDefaults: boolean } | { kind: "delete" };

  let mode = $state<Mode>({ kind: "idle" });
  let name = $state("");
  let input = $state<HTMLInputElement>();

  const active = $derived(config.value.activeProfile);
  const profiles = $derived(Object.entries(config.value.profiles));
  const activeName = $derived(config.value.profiles[active]?.name ?? "");

  function start(m: Mode) {
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
</script>

<div class="mb-4 flex flex-wrap items-center gap-x-4 gap-y-3 card px-5 py-3 text-sm">
  <span class="text-xs font-semibold tracking-wider text-muted uppercase">Profile</span>

  {#if mode.kind === "rename" || mode.kind === "new"}
    <input
      bind:this={input}
      bind:value={name}
      onkeydown={onKey}
      class="field w-56"
      placeholder="Profile name"
      maxlength="40"
    />
    <button class="btn btn-primary" disabled={!name.trim()} onclick={commit}>
      {mode.kind === "rename" ? "Rename" : "Create"}
    </button>
    <button class="btn" onclick={() => (mode = { kind: "idle" })}>Cancel</button>
    {#if mode.kind === "new"}
      <span class="text-xs text-muted">
        {mode.fromDefaults ? "Starts from the default layout." : `Starts as a copy of “${activeName}”.`}
      </span>
    {/if}
  {:else if mode.kind === "delete"}
    <span>Delete “{activeName}”? This can't be undone.</span>
    <button
      class="btn bg-bad! text-white"
      onclick={() => {
        deleteProfile(active);
        mode = { kind: "idle" };
      }}>Delete</button
    >
    <button class="btn" onclick={() => (mode = { kind: "idle" })}>Cancel</button>
  {:else}
    <select class="field w-56" value={active} onchange={(e) => switchProfile(e.currentTarget.value)}>
      {#each profiles as [id, p] (id)}<option value={id}>{p.name}</option>{/each}
    </select>
    <div class="flex gap-2">
      <button class="btn" onclick={() => start({ kind: "new", fromDefaults: false })}>Duplicate</button>
      <button class="btn" onclick={() => start({ kind: "new", fromDefaults: true })}>New</button>
      <button class="btn" onclick={() => start({ kind: "rename" })}>Rename</button>
      <button class="btn" disabled={profiles.length < 2} onclick={() => start({ kind: "delete" })}>Delete</button>
    </div>
    <span class="text-xs text-muted">Layout, widget options, scale and opacity are saved per profile.</span>
  {/if}
</div>
