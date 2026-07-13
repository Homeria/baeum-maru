from __future__ import annotations

import json
from collections.abc import Mapping
from copy import deepcopy
from pathlib import Path
from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator
from pydantic_settings import (
    BaseSettings,
    DotEnvSettingsSource,
    EnvSettingsSource,
    SettingsConfigDict,
    SettingsError,
)

from app.core.runtime import RuntimePaths


class SettingsLoadError(RuntimeError):
    """Raised when a settings source cannot be parsed or validated."""


class ServerSettings(BaseModel):
    model_config = ConfigDict(extra="forbid")

    host: str = "127.0.0.1"
    port: int = Field(default=18080, ge=1024, le=65535)

    @field_validator("host")
    @classmethod
    def validate_host(cls, value: str) -> str:
        normalized = value.strip()
        if not normalized:
            raise ValueError("server host must not be empty")
        return normalized


class LoggingSettings(BaseModel):
    model_config = ConfigDict(extra="forbid")

    level: Literal["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"] = "INFO"
    json_console: bool = False
    max_bytes: int = Field(default=5 * 1024 * 1024, ge=1024)
    backup_count: int = Field(default=5, ge=1, le=20)

    @field_validator("level", mode="before")
    @classmethod
    def normalize_level(cls, value: object) -> object:
        if isinstance(value, str):
            return value.upper()
        return value


class AppSettings(BaseModel):
    model_config = ConfigDict(extra="forbid")

    environment: Literal["development", "test", "production"] = "development"
    server: ServerSettings = Field(default_factory=ServerSettings)
    logging: LoggingSettings = Field(default_factory=LoggingSettings)


class _ServerOverrides(BaseModel):
    host: str | None = None
    port: int | None = None


class _LoggingOverrides(BaseModel):
    level: str | None = None
    json_console: bool | None = None
    max_bytes: int | None = None
    backup_count: int | None = None


class _EnvironmentOverrides(BaseSettings):
    model_config = SettingsConfigDict(
        env_prefix="BAEUM_MARU_",
        env_nested_delimiter="__",
        case_sensitive=False,
        extra="ignore",
    )

    environment: Literal["development", "test", "production"] | None = None
    server: _ServerOverrides | None = None
    logging: _LoggingOverrides | None = None


def load_settings(paths: RuntimePaths) -> AppSettings:
    try:
        file_values = _read_json_settings(paths.settings_file)
        environment_values = _read_environment_overrides(paths)
        merged_values = _deep_merge(file_values, environment_values)
        return AppSettings.model_validate(merged_values)
    except SettingsLoadError:
        raise
    except (SettingsError, ValidationError) as error:
        raise SettingsLoadError(f"Invalid application settings: {error}") from error


def _read_json_settings(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}

    try:
        raw_value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise SettingsLoadError(f"Unable to read settings file: {path}") from error

    if not isinstance(raw_value, dict):
        raise SettingsLoadError(f"Settings file must contain a JSON object: {path}")

    return raw_value


def _read_environment_overrides(paths: RuntimePaths) -> dict[str, Any]:
    dotenv_values: dict[str, Any] = {}
    if paths.env_file.is_file():
        dotenv_values = DotEnvSettingsSource(
            _EnvironmentOverrides,
            env_file=paths.env_file,
            env_file_encoding="utf-8",
        )()

    environment_values = EnvSettingsSource(_EnvironmentOverrides)()
    merged_values = _deep_merge(dotenv_values, environment_values)
    return _EnvironmentOverrides.model_validate(merged_values).model_dump(exclude_none=True)


def _deep_merge(base: Mapping[str, Any], overrides: Mapping[str, Any]) -> dict[str, Any]:
    merged = deepcopy(dict(base))

    for key, value in overrides.items():
        current = merged.get(key)
        if isinstance(current, dict) and isinstance(value, Mapping):
            merged[key] = _deep_merge(current, value)
        else:
            merged[key] = deepcopy(value)

    return merged
