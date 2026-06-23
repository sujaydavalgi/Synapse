/**
 * Shared fetch helpers for remote deploy and fleet clients.
 * @module
 */

export const REMOTE_HTTP_TIMEOUT_MS = 30_000;

function safeUrlForError(url: string): string {
  // Redact query strings and fragments from URLs embedded in fetch error messages.
  //
  // Parameters:
  // - `url` — raw request URL
  //
  // Returns:
  // Origin plus pathname, or a placeholder when parsing fails.
  //
  // Options:
  // None.
  //
  // Example:
  // safeUrlForError("https://api.example.com/deploy?token=secret")

  try {
    const parsed = new URL(url);
    return `${parsed.origin}${parsed.pathname}`;
  } catch {
    return "[unparseable-url]";
  }
}

export function remoteFetch(url: string, init: RequestInit = {}): Promise<Response> {
  // Issue one HTTP request with a bounded wait for connect and response body.
  const controller = new AbortController();
  const { signal: upstreamSignal, ...restInit } = init;
  let onAbort: (() => void) | undefined;
  const detachUpstreamAbortListener = () => {
    if (upstreamSignal && onAbort) {
      upstreamSignal.removeEventListener("abort", onAbort);
    }
  };

  if (upstreamSignal?.aborted) {
    return Promise.reject(
      new DOMException("Remote fetch operation was aborted.", "AbortError"),
    );
  }

  // Bound the full fetch lifecycle (connect + response/body read); expiry aborts via AbortController.
  const timeoutId = setTimeout(() => {
    detachUpstreamAbortListener();
    controller.abort(
      new DOMException(
        `Remote fetch timed out after ${REMOTE_HTTP_TIMEOUT_MS}ms for URL: ${safeUrlForError(url)}`,
        "TimeoutError",
      ),
    );
  }, REMOTE_HTTP_TIMEOUT_MS);

  if (upstreamSignal) {
    onAbort = () => {
      detachUpstreamAbortListener();
      clearTimeout(timeoutId);
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
    detachUpstreamAbortListener();
  });
}
