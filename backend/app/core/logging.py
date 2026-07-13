"""회전 파일, 구조화 출력과 민감값 제거를 포함한 로깅을 구성한다."""

from __future__ import annotations

import json
import logging
import re
from collections.abc import Mapping
from datetime import UTC, datetime
from logging.handlers import RotatingFileHandler
from pathlib import Path
from typing import Any

from app.core.settings import LoggingSettings

_SENSITIVE_KEY = re.compile(
    r"password|token|secret|authorization|cookie|access[_-]?code|phone", re.IGNORECASE
)
_SENSITIVE_TEXT = re.compile(
    r"(?i)(password|token|secret|authorization|cookie|access[_-]?code|phone)"
    r"(\s*[:=]\s*)([^\s,;]+)"
)
_STANDARD_RECORD_FIELDS = frozenset(logging.makeLogRecord({}).__dict__)


def _redact(value: Any, key: str | None = None) -> Any:
    if key is not None and _SENSITIVE_KEY.search(key):
        return "[REDACTED]"
    if isinstance(value, Mapping):
        return {str(item_key): _redact(item, str(item_key)) for item_key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_redact(item) for item in value]
    if isinstance(value, str):
        return _SENSITIVE_TEXT.sub(r"\1\2[REDACTED]", value)
    return value


class JsonFormatter(logging.Formatter):
    """LogRecord를 한 줄 JSON으로 직렬화한다."""

    def format(self, record: logging.LogRecord) -> str:
        payload: dict[str, Any] = {
            "timestamp": datetime.fromtimestamp(record.created, UTC).isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "message": _redact(record.getMessage()),
        }
        extras = {
            key: value
            for key, value in record.__dict__.items()
            if key not in _STANDARD_RECORD_FIELDS and key not in {"message", "asctime"}
        }
        payload.update(_redact(extras))
        if record.exc_info:
            payload["exception"] = self.formatException(record.exc_info)
        return json.dumps(payload, ensure_ascii=False, default=str)


class RedactingTextFormatter(logging.Formatter):
    """사람이 읽는 콘솔 로그에서도 민감값을 제거한다."""

    def format(self, record: logging.LogRecord) -> str:
        return str(_redact(super().format(record)))


def configure_logging(
    settings: LoggingSettings,
    log_file: Path,
    *,
    logger: logging.Logger | None = None,
) -> logging.Logger:
    """애플리케이션 logger에 콘솔과 회전 파일 handler를 설치한다."""
    target = logger or logging.getLogger()
    target.setLevel(settings.level)
    target.handlers.clear()

    console = logging.StreamHandler()
    if settings.json_console:
        console.setFormatter(JsonFormatter())
    else:
        console.setFormatter(RedactingTextFormatter("%(levelname)s %(name)s: %(message)s"))

    log_file.parent.mkdir(parents=True, exist_ok=True)
    file_handler = RotatingFileHandler(
        log_file,
        maxBytes=settings.max_bytes,
        backupCount=settings.backup_count,
        encoding="utf-8",
    )
    file_handler.setFormatter(JsonFormatter())

    target.addHandler(console)
    target.addHandler(file_handler)
    return target
