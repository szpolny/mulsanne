<script lang="ts">
  import { api } from "../../lib/api";
  import { displayNameError } from "../../lib/online";
  import type { OnlineMe } from "../../lib/types";

  let { open, onclose, onenabled }: { open: boolean; onclose: () => void; onenabled: (me: OnlineMe) => void } = $props();

  let dialog: HTMLDialogElement;
  let name = $state("");
  let busy = $state(false);
  let error = $state("");
  const nameError = $derived(displayNameError(name));

  $effect(() => {
    if (open) {
      error = "";
      dialog.showModal();
    } else dialog.close();
  });

  async function enable() {
    busy = true;
    error = "";
    try {
      onenabled(await api.onlineRegister(name.trim() || null));
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="online-title"
  class="m-auto w-[min(34rem,calc(100vw-3rem))] max-h-[calc(100vh-3rem)] overflow-hidden rounded-xl border border-line bg-surface p-0 text-fg shadow-2xl backdrop:bg-black/70"
  onclose={onclose}
>
  <form
    class="flex max-h-[calc(100vh-3rem-2px)] flex-col"
    onsubmit={(event) => {
      event.preventDefault();
      if (!nameError && !busy) enable();
    }}
  >
    <header class="shrink-0 border-b border-line p-6">
      <div class="mb-1 text-xs font-semibold tracking-wider text-accent uppercase">Online features</div>
      <h2 id="online-title" class="page-title">Before you turn this on</h2>
    </header>

    <div class="min-h-0 space-y-4 overflow-y-auto p-6 text-sm leading-relaxed text-muted">
      <p>
        Mulsanne creates a private key for this PC and registers it with the Mulsanne server. There's no account or
        password: the <code>identity.key</code> file in your data folder is your identity.
      </p>
      <ul class="list-disc space-y-2 pl-5 marker:text-accent">
        <li>Turning this on sends only your display name, if you set one.</li>
        <li>
          When you upload laps or share car setups, their lap and sector times, car, track, conditions and telemetry
          trace are sent to the server and shown to other drivers.
        </li>
        <li>
          Without a display name you appear as <span class="text-fg">Anonymous</span>. Other drivers can't see who you
          are, but the server still links your uploads to this PC's key.
        </li>
        <li>You can delete everything you've uploaded from Settings at any time.</li>
      </ul>

      <label class="block">
        <span class="mb-1 block font-medium text-fg">Display name <span class="font-normal text-muted">(optional)</span></span>
        <input class="field w-full" maxlength="32" placeholder="Anonymous" bind:value={name} />
      </label>
      {#if nameError}
        <p class="text-bad">{nameError}</p>
      {/if}
      {#if error}
        <p class="text-bad">{error}</p>
      {/if}
    </div>

    <footer class="flex shrink-0 justify-end gap-2 border-t border-line p-6">
      <button type="button" class="btn" onclick={onclose}>Cancel</button>
      <button type="submit" class="btn btn-primary" disabled={busy || !!nameError}>
        {busy ? "Turning on…" : "Turn on"}
      </button>
    </footer>
  </form>
</dialog>
