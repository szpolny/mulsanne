<script lang="ts">
  import { api } from "../../lib/api";
  import { config, saveConfig } from "../../lib/config.svelte";
  import { displayNameError } from "../../lib/online";
  import type { OnlineMe } from "../../lib/types";
  import OnlineDialog from "./OnlineDialog.svelte";
  import Toggle from "./Toggle.svelte";

  const online = $derived(config.value.online);

  let dialogOpen = $state(false);
  let me = $state<OnlineMe | null>(null);
  let loadError = $state("");
  let name = $state("");
  let busy = $state(false);
  let actionError = $state("");
  let confirmDelete = $state(false);

  const nameError = $derived(displayNameError(name));
  const nameChanged = $derived(me !== null && name.trim() !== (me.displayName ?? ""));

  function setMe(value: OnlineMe) {
    me = value;
    name = value.displayName ?? "";
  }

  // Only talks to the server while online features are on.
  $effect(() => {
    if (!online.enabled || me) return;
    loadError = "";
    api.onlineMe().then(setMe, (e) => (loadError = String(e)));
  });

  function setEnabled(enabled: boolean) {
    if (enabled) {
      dialogOpen = true;
      return;
    }
    online.enabled = false;
    confirmDelete = false;
    saveConfig();
  }

  async function run(action: () => Promise<void>) {
    busy = true;
    actionError = "";
    try {
      await action();
    } catch (e) {
      actionError = String(e);
    } finally {
      busy = false;
    }
  }

  const saveName = () => run(async () => setMe(await api.onlineSetDisplayName(name.trim() || null)));

  const deleteData = () =>
    run(async () => {
      await api.onlineDeleteData();
      me = null;
      name = "";
      confirmDelete = false;
      online.enabled = false;
      saveConfig();
    });
</script>

<div class="divide-y divide-line card text-sm">
  <div class="flex items-center justify-between gap-6 p-4">
    <div>
      <div class="font-medium">Online features</div>
      <div class="text-muted">Lap rankings and car setup sharing. While off, Mulsanne doesn't contact its server.</div>
    </div>
    <Toggle checked={online.enabled} label="Online features" onchange={setEnabled} />
  </div>

  {#if online.enabled}
    {#if me}
      <form
        class="flex items-center justify-between gap-6 p-4"
        onsubmit={(event) => {
          event.preventDefault();
          if (nameChanged && !nameError && !busy) saveName();
        }}
      >
        <div>
          <div class="font-medium">Display name</div>
          <div class="text-muted">Shown on rankings and shared setups. Leave empty to appear as Anonymous.</div>
          {#if nameError}
            <div class="mt-1 text-bad">{nameError}</div>
          {/if}
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <input class="field w-44" maxlength="32" placeholder="Anonymous" aria-label="Display name" bind:value={name} />
          <button type="submit" class="btn" disabled={!nameChanged || !!nameError || busy}>Save</button>
        </div>
      </form>
      <div class="flex items-center justify-between gap-6 p-4">
        <div>
          <div class="font-medium">Delete my online data</div>
          <div class="text-muted">
            Removes everything you've uploaded and this PC's online identity, then turns online features off.
          </div>
        </div>
        <div class="flex shrink-0 gap-2">
          {#if confirmDelete}
            <button class="btn" disabled={busy} onclick={() => (confirmDelete = false)}>Cancel</button>
            <button class="btn btn-primary" disabled={busy} onclick={deleteData}>Delete everything</button>
          {:else}
            <button class="btn" onclick={() => (confirmDelete = true)}>Delete…</button>
          {/if}
        </div>
      </div>
      {#if actionError}
        <div class="p-4 text-bad">{actionError}</div>
      {/if}
    {:else if loadError}
      <div class="flex items-center justify-between gap-6 p-4">
        <div class="text-bad">{loadError}</div>
        <button class="btn shrink-0" onclick={() => (dialogOpen = true)}>Set up again</button>
      </div>
    {:else}
      <div class="p-4 text-muted">Connecting…</div>
    {/if}
  {/if}
</div>

<OnlineDialog
  open={dialogOpen}
  onclose={() => (dialogOpen = false)}
  onenabled={(value) => {
    setMe(value);
    loadError = "";
    dialogOpen = false;
    online.enabled = true;
    saveConfig();
  }}
/>
