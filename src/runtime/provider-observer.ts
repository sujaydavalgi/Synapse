/**
 * Optional hooks for recording provider and transport call telemetry.
 * @module
 */

export type ProviderCallObserver = (
  providerKey: string,
  category: string,
  durationMs: number,
  failed: boolean,
) => void;

let observer: ProviderCallObserver | undefined;

export function setProviderCallObserver(next?: ProviderCallObserver): void {
  observer = next;
}

export function notifyProviderCall(
  providerKey: string,
  category: string,
  durationMs: number,
  failed = false,
): void {
  observer?.(providerKey, category, durationMs, failed);
}
