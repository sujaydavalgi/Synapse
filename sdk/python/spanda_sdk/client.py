"""Official Spanda Python SDK — thin REST client over Control Center API v1."""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
import uuid
from typing import Any, Mapping, Optional

from spanda_sdk.errors import ConnectionError, PermissionError, SpandaError, ValidationError
from spanda_sdk.stream import TelemetryStream

__all__ = ["SpandaClient", "SpandaError", "TelemetryStream"]


class SpandaClient:
    """REST v1 client with optional Bearer auth and correlation IDs."""

    def __init__(
        self,
        base_url: Optional[str] = None,
        api_key: Optional[str] = None,
        timeout: float = 30.0,
    ) -> None:
        resolved_url = base_url or os.environ.get(
            "SPANDA_CONTROL_CENTER_URL", "http://127.0.0.1:8080"
        )
        self.base_url = resolved_url.rstrip("/")
        self.api_key = api_key if api_key is not None else os.environ.get("SPANDA_API_KEY")
        self.timeout = timeout

    @classmethod
    def local(cls) -> "SpandaClient":
        """Connect to the local Control Center."""
        return cls()

    def _request(
        self,
        method: str,
        path: str,
        body: Optional[Mapping[str, Any]] = None,
        auth: bool = False,
    ) -> Any:
        url = f"{self.base_url}{path}"
        headers = {"Accept": "application/json", "X-Correlation-ID": f"py-sdk-{uuid.uuid4().hex[:12]}"}
        data = None
        if body is not None:
            headers["Content-Type"] = "application/json"
            data = json.dumps(body).encode("utf-8")
        if auth and self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        req = urllib.request.Request(url, data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                payload = resp.read().decode("utf-8")
                if not payload:
                    return {}
                return json.loads(payload)
        except urllib.error.HTTPError as exc:
            detail = exc.read().decode("utf-8", errors="replace")
            if exc.code in (401, 403):
                raise PermissionError(detail, status=exc.code) from exc
            if exc.code == 400:
                raise ValidationError(detail, status=exc.code) from exc
            raise SpandaError(detail, status=exc.code) from exc
        except urllib.error.URLError as exc:
            raise ConnectionError(str(exc.reason)) from exc

    def _program_body(self, file_or_project: str) -> dict[str, str]:
        return {"file": file_or_project}

    def readiness(self, file_or_project: str) -> Any:
        return self._request(
            "POST", "/v1/programs/readiness", self._program_body(file_or_project)
        )

    def assure(self, file_or_project: str) -> Any:
        return self._request(
            "POST", "/v1/programs/assure", self._program_body(file_or_project)
        )

    def diagnose(self, trace_or_file: str) -> Any:
        return self._request(
            "POST", "/v1/programs/diagnose", self._program_body(trace_or_file)
        )

    def heal(self, target: str) -> Any:
        return self._request("POST", "/v1/programs/recovery/heal", self._program_body(target))

    def verify_hardware(self, project: str) -> Any:
        return self._request(
            "POST", "/v1/programs/verify/hardware", self._program_body(project)
        )

    def verify_capabilities(self, project: str) -> Any:
        return self._request(
            "POST",
            "/v1/programs/verify/capabilities",
            {"file": project, "traceability": True},
        )

    def list_entities(self) -> Any:
        return self._request("GET", "/v1/entities")

    def get_entity(self, entity_id: str) -> Any:
        return self._request("GET", f"/v1/entities/{entity_id}")

    def list_devices(self) -> Any:
        return self._request("GET", "/v1/devices", auth=True)

    def provision_device(self, device_id: str, body: Optional[Mapping[str, Any]] = None) -> Any:
        return self._request(
            "POST",
            f"/v1/devices/{device_id}/provision",
            body or {},
            auth=True,
        )

    def run_simulation(self, project: str) -> Any:
        return self._request(
            "POST", "/v1/programs/simulation", self._program_body(project)
        )

    def replay(self, trace: str) -> Any:
        return self._request("POST", "/v1/programs/replay", self._program_body(trace))

    def get_health(self, entity_id: str) -> Any:
        return self._request("GET", f"/v1/entities/{entity_id}/health")

    def get_trust(self, entity_id: str) -> Any:
        return self._request("GET", f"/v1/entities/{entity_id}/trust")

    def get_package_trust(self, package: str, version: Optional[str] = None) -> Any:
        path = f"/v1/trust/package?name={package}"
        if version:
            path += f"&version={version}"
        return self._request("GET", path)

    def health_check(self) -> Any:
        return self._request("GET", "/v1/health")

    def rpc(self, method: str, params: Optional[Mapping[str, Any]] = None) -> Any:
        payload = self._request(
            "POST",
            "/v1/rpc",
            {"method": method, "params": params or {}},
        )
        return payload.get("result", payload)

    # Backward-compatible Control Center helpers
    def dashboard(self) -> Any:
        return self._request("GET", "/v1/dashboard")

    def readiness_run(self, body: Optional[Mapping[str, Any]] = None) -> Any:
        return self._request("POST", "/v1/readiness/run", body or {})

    def trust_package(self, name: str, version: Optional[str] = None) -> Any:
        return self.get_package_trust(name, version)
