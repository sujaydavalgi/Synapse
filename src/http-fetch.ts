/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  const controller = new AbortController();
  // Bound the full fetch lifecycle (connect + response/body read); expiry aborts via AbortController.
  const timeoutId = setTimeout(() => controller.abort(), REMOTE_HTTP_TIMEOUT_MS);

  const { signal: upstreamSignal, ...restInit } = init;
  let onAbort: (() => void) | undefined;

  if (upstreamSignal) {
    if (upstreamSignal.aborted) {
      controller.abort();
      clearTimeout(timeoutId);
      return Promise.reject(
        new DOMException(`Remote fetch operation was aborted for URL: ${url}`, "AbortError"),
      );
    }
    onAbort = () => controller.abort();
    upstreamSignal.addEventListener("abort", onAbort, { once: true });
  }

  return fetch(url, { ...restInit, signal: controller.signal }).finally(() => {
    clearTimeout(timeoutId);
    if (upstreamSignal && onAbort) {
      upstreamSignal.removeEventListener("abort", onAbort);
    }
  });
}
