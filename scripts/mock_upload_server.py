#!/usr/bin/env python3
"""One-shot HTTP server that records a POST body and exits."""

from __future__ import annotations

import sys
from http.server import BaseHTTPRequestHandler, HTTPServer


class UploadHandler(BaseHTTPRequestHandler):
    outfile: str

    def do_POST(self) -> None:  # noqa: N802
        """
        Description:
            Handle one POST upload and write the body to the configured outfile.

        Inputs:
            None.

        Outputs:
            None.

        Example:
            Used internally by HTTPServer; not called directly.
        """
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        with open(self.outfile, "wb") as handle:
            handle.write(body)
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"ok":true}')
        raise SystemExit(0)

    def log_message(self, format: str, *args: object) -> None:
        """
        Description:
            Suppress default HTTP server stderr logging during tests.

        Inputs:
            format: str
                Log format string (ignored).
            args: object
                Format arguments (ignored).

        Outputs:
            None.

        Example:
            Called by BaseHTTPRequestHandler; intentionally no-op.
        """
        return


def main() -> None:
    """
    Description:
        Start a one-shot HTTP server that records one POST body and exits.

    Inputs:
        None.

    Outputs:
        None.

    Example:
        python3 scripts/mock_upload_server.py /tmp/body.bin 18080
    """
    if len(sys.argv) != 3:
        print("usage: mock_upload_server.py <outfile> <port>", file=sys.stderr)
        raise SystemExit(2)
    outfile = sys.argv[1]
    port = int(sys.argv[2])
    UploadHandler.outfile = outfile
    HTTPServer(("127.0.0.1", port), UploadHandler).handle_request()


if __name__ == "__main__":
    main()
