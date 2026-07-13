from __future__ import annotations

import logging
from dataclasses import dataclass
from pathlib import Path

from app.core.logging import configure_logging
from app.core.runtime import RuntimePaths
from app.core.settings import AppSettings, load_settings


@dataclass(frozen=True, slots=True)
class RuntimeContext:
    paths: RuntimePaths
    settings: AppSettings


def bootstrap_runtime(runtime_dir: str | Path | None = None) -> RuntimeContext:
    paths = RuntimePaths.discover(runtime_dir)
    paths.ensure_directories()
    settings = load_settings(paths)
    configure_logging(settings.logging, paths.log_file)

    logger = logging.getLogger("baeum_maru.bootstrap")
    logger.info(
        "runtime initialized",
        extra={"event": "runtime.initialized", "resource_id": str(paths.root)},
    )
    return RuntimeContext(paths=paths, settings=settings)
