<script lang="ts">
  import { accelerator } from "../../lib/shortcuts";

  let {
    value,
    fallback,
    label,
    onchange,
  }: {
    value: string;
    /** Shortcut restored by the reset button. */
    fallback: string;
    label: string;
    /** Applies the new shortcut; rejects with a message when it can't be used. */
    onchange: (accelerator: string) => Promise<void>;
  } = $props();

  let recording = $state(false);
  let error = $state("");

  async function apply(accelerator: string) {
    error = "";
    try {
      await onchange(accelerator);
    } catch (e) {
      error = String(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    const pressed = accelerator(e);
    if (!pressed) return;
    recording = false;
    if (pressed === "Escape") return;
    apply(pressed);
  }
</script>

<div class="flex flex-col items-end gap-1">
  <div class="flex items-center gap-2">
    <button
      class={["btn min-w-40 justify-center font-mono! tracking-normal! normal-case!", recording && "ring-2 ring-accent"]}
      aria-label={label}
      onclick={() => {
        error = "";
        recording = !recording;
      }}
      onblur={() => (recording = false)}
      {onkeydown}
    >
      {recording ? "Press keys…" : value}
    </button>
    <button class="btn" disabled={value === fallback} onclick={() => apply(fallback)}>Reset</button>
  </div>
  {#if error}
    <div class="max-w-72 text-right text-xs text-bad" role="alert">{error}</div>
  {/if}
</div>
