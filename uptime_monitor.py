"""
==============================================================================
B6-CORE HYPERSYSTEM — AUTOMATED UPTIME & HEALTH MONITOR
Monitored Endpoints:
  1. Streamlit Cockpit
  2. Render API (Krisler Guard)
==============================================================================
"""

import asyncio
import time
import logging
from typing import Dict, Any, List
import httpx

# Logging-Konfiguration
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s"
)
logger = logging.getLogger("b6_uptime_monitor")

# HIER DEINE ENDPUNKTE EINTRAGEN
ENDPOINTS: List[Dict[str, Any]] = [
    {
        "name": "Streamlit Cockpit UI",
        "url": "https://black-six-matrix-pam3ly3dahquagxagaif4w.streamlit.app/",
        "method": "GET",
        "expected_status": 200,
    },
    {
        "name": "Render Krisler 2.1 API",
        "url": "https://aragon-backend-api.onrender.com/prompt/check",
        "method": "POST",
        "payload": {
            "prompt": "Uptime Health Check",
            "domain_code": "0052"
        },
        "expected_status": 200,
    }
]

class B6UptimeMonitor:
    def __init__(self, check_interval: int = 60, timeout: float = 15.0):
        self.check_interval = check_interval
        self.timeout = timeout

    async def check_endpoint(self, client: httpx.AsyncClient, endpoint: Dict[str, Any]) -> Dict[str, Any]:
        name = endpoint["name"]
        url = endpoint["url"]
        method = endpoint["method"]
        expected = endpoint["expected_status"]
        payload = endpoint.get("payload", None)

        start_time = time.perf_counter()
        try:
            if method == "POST":
                response = await client.post(url, json=payload, timeout=self.timeout)
            else:
                response = await client.get(url, timeout=self.timeout)

            latency_ms = round((time.perf_counter() - start_time) * 1000, 2)
            is_healthy = response.status_code == expected

            status_symbol = "✅ ONLINE" if is_healthy else f"⚠️ DEGRADED ({response.status_code})"
            
            logger.info(
                f"[{status_symbol}] {name} | Latenz: {latency_ms}ms | Status Code: {response.status_code}"
            )

            return {
                "name": name,
                "url": url,
                "status": "UP" if is_healthy else "DEGRADED",
                "status_code": response.status_code,
                "latency_ms": latency_ms,
                "error": None
            }

        except httpx.TimeoutException:
            latency_ms = round((time.perf_counter() - start_time) * 1000, 2)
            logger.error(f"[❌ TIMEOUT] {name} | Latenz: >{self.timeout}s")
            return {
                "name": name,
                "url": url,
                "status": "DOWN",
                "status_code": 0,
                "latency_ms": latency_ms,
                "error": "Request Timeout"
            }
        except Exception as e:
            latency_ms = round((time.perf_counter() - start_time) * 1000, 2)
            logger.error(f"[❌ ERROR] {name} | Fehler: {str(e)}")
            return {
                "name": name,
                "url": url,
                "status": "DOWN",
                "status_code": 0,
                "latency_ms": latency_ms,
                "error": str(e)
            }

    async def run_single_audit(self) -> List[Dict[str, Any]]:
        """Führt eine parallele Überprüfung aller Endpunkte aus."""
        logger.info("Starte Auditing aller B6-Core Endpunkte...")
        async with httpx.AsyncClient(follow_redirects=True) as client:
            tasks = [self.check_endpoint(client, ep) for ep in ENDPOINTS]
            results = await asyncio.gather(*tasks)
        return results

if __name__ == "__main__":
    monitor = B6UptimeMonitor(check_interval=30)
    # Einmaligen Audit-Run starten
    asyncio.run(monitor.run_single_audit())
