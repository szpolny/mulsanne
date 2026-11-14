// In-app replacement for window.confirm(), rendered by ConfirmDialog.
export type ConfirmOptions = {
  title: string;
  message?: string;
  /** Extra line shown below the message, e.g. a side effect worth knowing. */
  note?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  /** Style the confirm button as a destructive action. */
  danger?: boolean;
};

type Pending = ConfirmOptions & { resolve: (ok: boolean) => void };

export const confirmState = $state<{ current: Pending | null }>({ current: null });

/** Ask the user to confirm an action. Resolves to true if they accept. */
export function confirm(options: ConfirmOptions): Promise<boolean> {
  // A newer request replaces an open one, which counts as cancelled.
  confirmState.current?.resolve(false);
  return new Promise((resolve) => {
    confirmState.current = { ...options, resolve };
  });
}

export function settleConfirm(ok: boolean) {
  const pending = confirmState.current;
  if (!pending) return;
  confirmState.current = null;
  pending.resolve(ok);
}
