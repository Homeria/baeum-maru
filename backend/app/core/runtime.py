"""실행 파일과 분리된 writable runtime 경로를 결정한다."""

from __future__ import annotations

import os
import sys
from dataclasses import dataclass
from pathlib import Path

RUNTIME_DIR_ENV = "BAEUM_MARU_RUNTIME_DIR"


def distribution_root() -> Path:
    """개발 중에는 저장소, 패키징 후에는 실행 파일이 있는 디렉터리를 반환한다."""
    if getattr(sys, "frozen", False):
        return Path(sys.executable).resolve().parent
    return Path(__file__).resolve().parents[3]


@dataclass(frozen=True, slots=True)
class RuntimePaths:
    """설정, 데이터, 로그처럼 실행 중 변경되는 파일의 위치 모음."""

    root: Path

    @classmethod
    def discover(cls, runtime_dir: str | Path | None = None) -> RuntimePaths:
        configured = runtime_dir or os.getenv(RUNTIME_DIR_ENV) or "runtime"
        root = Path(configured).expanduser()
        if not root.is_absolute():
            root = distribution_root() / root
        return cls(root=root.resolve())

    @property
    def config_dir(self) -> Path:
        return self.root / "config"

    @property
    def data_dir(self) -> Path:
        return self.root / "data"

    @property
    def logs_dir(self) -> Path:
        return self.root / "logs"

    @property
    def backups_dir(self) -> Path:
        return self.root / "backups"

    @property
    def exports_dir(self) -> Path:
        return self.root / "exports"

    @property
    def imports_dir(self) -> Path:
        return self.root / "imports"

    @property
    def certificates_dir(self) -> Path:
        return self.root / "certificates"

    @property
    def temp_dir(self) -> Path:
        return self.root / "tmp"

    @property
    def settings_file(self) -> Path:
        return self.config_dir / "settings.json"

    @property
    def env_file(self) -> Path:
        return self.config_dir / ".env"

    @property
    def database_file(self) -> Path:
        return self.data_dir / "baeum-maru.db"

    @property
    def application_log_file(self) -> Path:
        return self.logs_dir / "baeum-maru.jsonl"

    def ensure_directories(self) -> None:
        for path in (
            self.config_dir,
            self.data_dir,
            self.logs_dir,
            self.backups_dir,
            self.exports_dir,
            self.imports_dir,
            self.certificates_dir,
            self.temp_dir,
        ):
            path.mkdir(parents=True, exist_ok=True)
