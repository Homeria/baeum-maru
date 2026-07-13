from __future__ import annotations

import json
import logging
import re
from datetime import UTC, datetime
from logging.handlers import RotatingFileHandler
from pathlib import Path

from app.core.settings import LoggingSettings

LOGGER_NAMESPACE = "baeum_maru"
REDACTED = "[REDACTED]"
_SENSITIVE_VALUE = re.compile(
    r"\b(password|token|secret|authorization|cookie|access_code|phone)\b"
    r"(\s*[:=]\s*)"
    r"(\"[^\"]*\"|'[^']*'|[^,\s}\]]+)",
    flags=re.IGNORECASE,
)


class SensitiveDataFilter(logging.Filter):
    def filter(self, record: logging.LogRecord) -> bool:
        record.msg = redact_sensitive_text(record.getMessage())
        record.args = ()
        return True


class JsonFormatter(logging.Formatter):
    def format(self, record: logging.LogRecord) -> str:
        payload: dict[str, object] = {
            "timestamp": datetime.fromtimestamp(record.created, tz=UTC).isoformat(
                timespec="milliseconds"
            ),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
        }

        for key in ("event", "request_id", "resource_id"):
            value = getattr(record, key, None)
            if value is not None:
                payload[key] = value

        if record.exc_info:
            payload["exception"] = self.formatException(record.exc_info)

        return json.dumps(payload, ensure_ascii=False, separators=(",", ":"))


def redact_sensitive_text(value: str) -> str:
    return _SENSITIVE_VALUE.sub(
        lambda match: f"{match.group(1)}{match.group(2)}{REDACTED}",
        value,
    )


def configure_logging(settings: LoggingSettings, log_file: Path) -> logging.Logger:
    logger = logging.getLogger(LOGGER_NAMESPACE)
    _remove_handlers(logger)

    logger.setLevel(getattr(logging, settings.level))
    logger.propagate = False
    sensitive_filter = SensitiveDataFilter()

    console_handler = logging.StreamHandler()
    console_handler.addFilter(sensitive_filter)
    if settings.json_console:
        console_handler.setFormatter(JsonFormatter())
    else:
        console_handler.setFormatter(
            logging.Formatter("%(asctime)s %(levelname)s %(name)s %(message)s")
        )

    file_handler = RotatingFileHandler(
        log_file,
        maxBytes=settings.max_bytes,
        backupCount=settings.backup_count,
        encoding="utf-8",
    )
    file_handler.addFilter(sensitive_filter)
    file_handler.setFormatter(JsonFormatter())

    logger.addHandler(console_handler)
    logger.addHandler(file_handler)
    return logger


def _remove_handlers(logger: logging.Logger) -> None:
    for handler in tuple(logger.handlers):
        logger.removeHandler(handler)
        handler.close()
