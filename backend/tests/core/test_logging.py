"""구조화 로그와 민감 정보 제거를 검증한다."""

import json
import logging
from pathlib import Path
from typing import Any

from app.core.logging import configure_logging
from app.core.settings import LoggingSettings


def _close_handlers(logger: logging.Logger) -> None:
    for handler in logger.handlers:
        handler.close()
    logger.handlers.clear()


def _read_log(path: Path) -> dict[str, Any]:
    value: object = json.loads(path.read_text(encoding="utf-8").strip())
    assert isinstance(value, dict)
    return value


def test_file_log_is_structured_and_keeps_context(tmp_path: Path) -> None:
    log_path = tmp_path / "logs" / "app.jsonl"
    logger = logging.getLogger("tests.logging.structured")
    logger.propagate = False
    configure_logging(LoggingSettings(), log_path, logger=logger)

    logger.info("member saved", extra={"member_id": 17, "operation": "create"})
    _close_handlers(logger)

    payload = _read_log(log_path)
    assert payload["level"] == "INFO"
    assert payload["message"] == "member saved"
    assert payload["member_id"] == 17
    assert payload["operation"] == "create"


def test_sensitive_values_are_redacted_from_file_log(tmp_path: Path) -> None:
    log_path = tmp_path / "logs" / "app.jsonl"
    logger = logging.getLogger("tests.logging.redaction")
    logger.propagate = False
    configure_logging(LoggingSettings(), log_path, logger=logger)

    logger.warning(
        "access_code=ABCD phone=010-1234-5678",
        extra={"password": "plain-text", "context": {"token": "secret-token"}},
    )
    _close_handlers(logger)

    serialized = log_path.read_text(encoding="utf-8")
    assert "ABCD" not in serialized
    assert "010-1234-5678" not in serialized
    assert "plain-text" not in serialized
    assert "secret-token" not in serialized
    assert serialized.count("[REDACTED]") >= 4
