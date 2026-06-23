/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), REMOTE_HTTP_TIMEOUT_MS);

  const { signal: upstreamSignal, ...restInit } = init;
  let onAbort: (() => void) | undefined;
  let hasUpstreamAbortListener = false;

  if (upstreamSignal) {
    if (upstreamSignal.aborted) {
      clearTimeout(timeoutId);
      controller.abort();
    } else {
      onAbort = () => controller.abort();
      upstreamSignal.addEventListener("abort", onAbort, { once: true });
      hasUpstreamAbortListener = true;
    }
  }

  return fetch(url, { ...restInit, signal: controller.signal }).finally(() => {
    clearTimeout(timeoutId);
    if (upstreamSignal && onAbort && hasUpstreamAbortListener) {
      upstreamSignal.removeEventListener("abort", onAbort);
    }
  });
}
