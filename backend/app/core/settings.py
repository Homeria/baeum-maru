"""JSON, dotenv, 환경변수를 병합해 검증된 애플리케이션 설정을 만든다."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Literal

from pydantic import BaseModel, Field, ValidationError, model_validator
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
)

from app.core.runtime import RuntimePaths


class SettingsLoadError(RuntimeError):
    """설정 파일을 읽거나 설정값을 검증할 수 없을 때 발생한다."""


class ServerSettings(BaseModel):
    host: str = "127.0.0.1"
    port: int = Field(default=18080, ge=1024, le=65535)


class LoggingSettings(BaseModel):
    level: Literal["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"] = "INFO"
    json_console: bool = False
    max_bytes: int = Field(default=5 * 1024 * 1024, ge=1024)
    backup_count: int = Field(default=5, ge=1, le=20)


class DatabaseSettings(BaseModel):
    busy_timeout_ms: int = Field(default=5_000, ge=1_000, le=60_000)
    echo_sql: bool = False


class AuthSettings(BaseModel):
    # 접속 코드는 오래 쓰는 로그인 수단, 세션은 그 코드로 연 짧은 사용 창이다.
    access_code_ttl_minutes: int = Field(default=60 * 24 * 30, ge=1)
    session_ttl_minutes: int = Field(default=60 * 12, ge=1)


class RealtimeSettings(BaseModel):
    heartbeat_interval_seconds: float = Field(default=20.0, ge=0.05, le=300.0)
    stale_timeout_seconds: float = Field(default=60.0, ge=0.1, le=900.0)
    send_timeout_seconds: float = Field(default=2.0, ge=0.1, le=30.0)
    event_queue_size: int = Field(default=256, ge=16, le=10_000)
    max_connections: int = Field(default=20, ge=1, le=200)

    @model_validator(mode="after")
    def validate_heartbeat_window(self) -> RealtimeSettings:
        if self.stale_timeout_seconds <= self.heartbeat_interval_seconds:
            raise ValueError("stale timeout must be greater than heartbeat interval")
        return self


class AppSettings(BaseSettings):
    """애플리케이션에서 사용하는 설정의 단일 검증 모델."""

    model_config = SettingsConfigDict(
        env_prefix="BAEUM_MARU_",
        env_nested_delimiter="__",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    environment: Literal["development", "test", "production"] = "development"
    server: ServerSettings = Field(default_factory=ServerSettings)
    logging: LoggingSettings = Field(default_factory=LoggingSettings)
    database: DatabaseSettings = Field(default_factory=DatabaseSettings)
    auth: AuthSettings = Field(default_factory=AuthSettings)
    realtime: RealtimeSettings = Field(default_factory=RealtimeSettings)

    @classmethod
    def settings_customise_sources(
        cls,
        settings_cls: type[BaseSettings],
        init_settings: PydanticBaseSettingsSource,
        env_settings: PydanticBaseSettingsSource,
        dotenv_settings: PydanticBaseSettingsSource,
        file_secret_settings: PydanticBaseSettingsSource,
    ) -> tuple[PydanticBaseSettingsSource, ...]:
        # OS 환경변수와 runtime .env가 JSON 초기값보다 우선한다.
        return env_settings, dotenv_settings, init_settings, file_secret_settings


def _read_json_settings(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SettingsLoadError(f"설정 파일을 읽을 수 없습니다: {path}") from exc
    if not isinstance(value, dict):
        raise SettingsLoadError(f"설정 파일 최상위 값은 JSON 객체여야 합니다: {path}")
    return value


def load_settings(paths: RuntimePaths) -> AppSettings:
    """runtime 설정을 우선순위에 따라 읽고 검증한다."""
    json_settings = _read_json_settings(paths.settings_file)
    try:
        # BaseSettings accepts _env_file at runtime but its generated typing omits the keyword.
        return AppSettings(**json_settings, _env_file=paths.env_file)  # type: ignore[call-arg]
    except (OSError, ValidationError) as exc:
        raise SettingsLoadError("애플리케이션 설정값이 올바르지 않습니다.") from exc
