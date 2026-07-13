import json
import logging
from pathlib import Path

from app.core.logging import configure_logging, redact_sensitive_text
from app.core.settings import LoggingSettings


def test_sensitive_values_are_redacted() -> None:
    message = 'password=hunter2 token="abc" access_code:1234 phone=010-1234-5678'

    redacted = redact_sensitive_text(message)

    assert "hunter2" not in redacted
    assert "abc" not in redacted
    assert "1234" not in redacted
    assert "010-1234-5678" not in redacted
    assert redacted.count("[REDACTED]") == 4


def test_file_log_is_structured_and_redacted(tmp_path: Path) -> None:
    log_file = tmp_path / "logs" / "app.log"
    log_file.parent.mkdir()
    logger = configure_logging(LoggingSettings(), log_file)

    child_logger = logging.getLogger("baeum_maru.test")
    child_logger.info(
        "login token=top-secret",
        extra={"event": "login.succeeded", "request_id": "request-1"},
    )
    for handler in logger.handlers:
        handler.flush()

    payload = json.loads(log_file.read_text(encoding="utf-8").strip())
    assert payload["level"] == "INFO"
    assert payload["event"] == "login.succeeded"
    assert payload["request_id"] == "request-1"
    assert "top-secret" not in payload["message"]
    assert "[REDACTED]" in payload["message"]
