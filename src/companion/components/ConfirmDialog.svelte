<script lang="ts">
  import { confirmState, settleConfirm } from "../../lib/confirm.svelte";

  let dialog: HTMLDialogElement;
  const pending = $derived(confirmState.current);

  $effect(() => {
    if (pending) {
      if (!dialog.open) dialog.showModal();
    } else if (dialog.open) dialog.close();
  });
</script>

<dialog
  bind:this={dialog}
  aria-labelledby="confirm-title"
  aria-describedby={pending?.message ? "confirm-message" : undefined}
  class="m-auto w-[min(28rem,calc(100vw-3rem))] overflow-hidden rounded-xl border border-line bg-surface p-0 text-fg shadow-2xl backdrop:bg-black/70"
  onclose={() => settleConfirm(false)}
>
  {#if pending}
    <form
      onsubmit={(event) => {
        event.preventDefault();
        settleConfirm(true);
      }}
    >
      <div class="flex gap-4 p-6">
        {#if pending.danger}
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-accent/15 text-accent" aria-hidden="true">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
              ><path d="M4 7h16 M10 11v6 M14 11v6 M6 7l1 13h10l1-13 M9 7V4h6v3" /></svg
            >
          </div>
        {/if}
        <div class="min-w-0 space-y-2">
          <h2 id="confirm-title" class="section-title text-[17px]">{pending.title}</h2>
          {#if pending.message}
            <p id="confirm-message" class="text-sm leading-relaxed text-muted">{pending.message}</p>
          {/if}
          {#if pending.note}
            <p class="rounded-lg border border-line bg-surface-2 px-3 py-2 text-xs leading-relaxed text-muted">{pending.note}</p>
          {/if}
        </div>
      </div>
      <footer class="flex justify-end gap-2 border-t border-line bg-bg/40 px-6 py-4">
        <!-- svelte-ignore a11y_autofocus -->
        <button type="button" class="btn" autofocus={pending.danger} onclick={() => settleConfirm(false)}>{pending.cancelLabel ?? "Cancel"}</button>
        <!-- svelte-ignore a11y_autofocus -->
        <button type="submit" class="btn btn-primary" autofocus={!pending.danger}>{pending.confirmLabel ?? "OK"}</button>
      </footer>
    </form>
  {/if}
</dialog>
