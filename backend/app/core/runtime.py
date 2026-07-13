from __future__ import annotations

import os
import sys
from dataclasses import dataclass
from pathlib import Path

RUNTIME_DIR_ENV = "BAEUM_MARU_RUNTIME_DIR"


class RuntimePathError(RuntimeError):
    """Raised when writable runtime directories cannot be prepared."""


@dataclass(frozen=True, slots=True)
class RuntimePaths:
    root: Path
    config_dir: Path
    data_dir: Path
    backups_dir: Path
    exports_dir: Path
    imports_dir: Path
    logs_dir: Path
    certificates_dir: Path
    temp_dir: Path
    settings_file: Path
    env_file: Path
    database_file: Path
    log_file: Path

    @classmethod
    def discover(cls, runtime_dir: str | Path | None = None) -> RuntimePaths:
        distribution_root = _distribution_root()
        configured_root = runtime_dir or os.getenv(RUNTIME_DIR_ENV)

        if configured_root is None:
            root = distribution_root / "runtime"
        else:
            root = Path(configured_root).expanduser()
            if not root.is_absolute():
                root = distribution_root / root

        root = root.resolve(strict=False)
        config_dir = root / "config"
        data_dir = root / "data"
        logs_dir = root / "logs"

        return cls(
            root=root,
            config_dir=config_dir,
            data_dir=data_dir,
            backups_dir=root / "backups",
            exports_dir=root / "exports",
            imports_dir=root / "imports",
            logs_dir=logs_dir,
            certificates_dir=root / "certificates",
            temp_dir=root / "tmp",
            settings_file=config_dir / "settings.json",
            env_file=config_dir / ".env",
            database_file=data_dir / "baeum-maru.db",
            log_file=logs_dir / "baeum-maru.log",
        )

    @property
    def managed_directories(self) -> tuple[Path, ...]:
        return (
            self.root,
            self.config_dir,
            self.data_dir,
            self.backups_dir,
            self.exports_dir,
            self.imports_dir,
            self.logs_dir,
            self.certificates_dir,
            self.temp_dir,
        )

    def ensure_directories(self) -> None:
        try:
            for directory in self.managed_directories:
                directory.mkdir(parents=True, exist_ok=True)
        except OSError as error:
            raise RuntimePathError(f"Unable to prepare runtime directory: {directory}") from error


def _distribution_root() -> Path:
    if getattr(sys, "frozen", False):
        return Path(sys.executable).resolve().parent

    return Path(__file__).resolve().parents[3]
