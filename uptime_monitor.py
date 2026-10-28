# uptime_monitor.py
"""
Watchdog für das Blue Core System.
Prüft Go-Core, Python-API und Streamlit-Cockpit in einem Loop.
"""
import httpx
import time
import logging
import sys
from datetime import datetime

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [WATCHDOG] [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("WATCHDOG")

GO_HEALTH    = "http://localhost:8080/health"
PY_HEALTH    = "http://localhost:8000/health"
PY_STATS     = "http://localhost:8000/stats"
STREAMLIT    = "http://localhost:8501"
CHECK_INTERVAL = 10
TIMEOUT      = 3.0

def check_service(name: str, url: str, is_streamlit: bool = False) -> dict:
    try:
        r = httpx.get(url, timeout=TIMEOUT, follow_redirects=True)
        if r.status_code == 200:
            if is_streamlit:
                return {"name": name, "ok": True, "detail": "alive"}
            try:
                data = r.json()
                return {"name": name, "ok": True, "detail": data}
            except Exception:
                return {"name": name, "ok": True, "detail": "no-json"}
        return {"name": name, "ok": False, "detail": f"HTTP {r.status_code}"}
    except Exception as e:
        return {"name": name, "ok": False, "detail": str(e)}

def format_report(results: list) -> str:
    lines = []
    for r in results:
        icon = "✅" if r["ok"] else "❌"
        if isinstance(r["detail"], dict):
            if r["name"] == "GO_CORE":
                lines.append(
                    f"  {icon} {r['name']:<10} Blöcke={r['detail'].get('worm_block', '?')} "
                    f"Uptime={r['detail'].get('uptime_seconds', '?')}s"
                )
            elif r["name"] == "PY_API":
                lss = r["detail"].get("lss", {})
                lines.append(
                    f"  {icon} {r['name']:<10} Entitäten={r['detail'].get('entities_loaded', '?')} "
                    f"Bio={lss.get('bio_state', '?')}"
                )
            else:
                lines.append(f"  {icon} {r['name']:<10} {r['detail']}")
        else:
            lines.append(f"  {icon} {r['name']:<10} {r['detail']}")
    return "\n".join(lines)

def main():
    log.info("═" * 65)
    log.info("🚀 BLUE CORE WATCHDOG STARTET")
    log.info(f"   Intervall: {CHECK_INTERVAL}s | Timeout: {TIMEOUT}s")
    log.info(f"   Go-Core:   {GO_HEALTH}")
    log.info(f"   Python:    {PY_HEALTH}")
    log.info(f"   Streamlit: {STREAMLIT}")
    log.info("═" * 65)

    cycle = 0
    try:
        while True:
            cycle += 1
            results = [
                check_service("GO_CORE",   GO_HEALTH),
                check_service("PY_API",    PY_HEALTH),
                check_service("STREAMLIT", STREAMLIT, is_streamlit=True),
            ]

            ok_count = sum(1 for r in results if r["ok"])
            log.info(f"[Cycle {cycle}] {ok_count}/{len(results)} Services OK")
            print(format_report(results), flush=True)

            if ok_count < len(results):
                log.warning("⚠️ Mindestens ein Service DOWN")
            else:
                log.info("✅ Alle Services ONLINE")

            time.sleep(CHECK_INTERVAL)
    except KeyboardInterrupt:
        log.info("Watchdog manuell beendet.")
        sys.exit(0)

if __name__ == "__main__":
    main()
