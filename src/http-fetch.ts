/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  const controller = new AbortController();
  // Bound the full fetch lifecycle (connect + response/body read); expiry aborts via AbortController.
  const timeoutId = setTimeout(() => {
    controller.abort(
      new DOMException(
        `Remote fetch timed out after ${REMOTE_HTTP_TIMEOUT_MS}ms for URL: ${url}`,
        "TimeoutError",
      ),
    );
  }, REMOTE_HTTP_TIMEOUT_MS);

  const { signal: upstreamSignal, ...restInit } = init;
  let onAbort: (() => void) | undefined;

  if (upstreamSignal) {
    if (upstreamSignal.aborted) {
      const abortError = new DOMException("Remote fetch operation was aborted.", "AbortError");
      controller.abort(abortError);
      clearTimeout(timeoutId);
      return Promise.reject(abortError);
    }
    onAbort = () => {
      const reason =
        upstreamSignal.reason === undefined
          ? new DOMException("Remote fetch operation was aborted.", "AbortError")
          : upstreamSignal.reason;
      controller.abort(reason);
    };
    upstreamSignal.addEventListener("abort", onAbort, { once: true });
  }

  return fetch(url, { ...restInit, signal: controller.signal }).finally(() => {
    clearTimeout(timeoutId);
    if (upstreamSignal && onAbort) {
      upstreamSignal.removeEventListener("abort", onAbort);
    }
  });
}
