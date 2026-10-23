/** Mirrors the server's display name rules; empty means anonymous. */
export function displayNameError(name: string): string | null {
  const trimmed = name.trim();
  if (!trimmed) return null;
  const length = [...trimmed].length;
  if (length < 2 || length > 32) return "Display name must be 2 to 32 characters.";
  if (/\p{Cc}/u.test(trimmed)) return "Display name contains invalid characters.";
  return null;
}
