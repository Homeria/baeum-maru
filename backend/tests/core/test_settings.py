"""설정 source 우선순위와 Pydantic 검증을 확인한다."""

import json
from pathlib import Path

import pytest

from app.core.runtime import RuntimePaths
from app.core.settings import SettingsLoadError, load_settings


def _runtime_paths(tmp_path: Path) -> RuntimePaths:
    paths = RuntimePaths.discover(tmp_path / "runtime")
    paths.ensure_directories()
    return paths


def test_load_settings_uses_defaults_when_files_are_absent(tmp_path: Path) -> None:
    settings = load_settings(_runtime_paths(tmp_path))

    assert settings.environment == "development"
    assert settings.server.host == "127.0.0.1"
    assert settings.server.port == 18080


def test_environment_sources_override_json_in_order(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    paths = _runtime_paths(tmp_path)
    paths.settings_file.write_text(
        json.dumps(
            {
                "environment": "production",
                "server": {"host": "10.0.0.1", "port": 18001},
                "logging": {"level": "WARNING"},
            }
        ),
        encoding="utf-8",
    )
    paths.env_file.write_text(
        "BAEUM_MARU_SERVER__HOST=192.168.0.10\n"
        "BAEUM_MARU_SERVER__PORT=18002\n"
        "BAEUM_MARU_LOGGING__LEVEL=ERROR\n",
        encoding="utf-8",
    )
    monkeypatch.setenv("BAEUM_MARU_SERVER__PORT", "18003")

    settings = load_settings(paths)

    assert settings.environment == "production"
    assert settings.server.host == "192.168.0.10"
    assert settings.server.port == 18003
    assert settings.logging.level == "ERROR"


def test_invalid_json_setting_raises_readable_error(tmp_path: Path) -> None:
    paths = _runtime_paths(tmp_path)
    paths.settings_file.write_text('{"server": {"port": 80}}', encoding="utf-8")

    with pytest.raises(SettingsLoadError, match="올바르지 않습니다"):
        load_settings(paths)


def test_json_root_must_be_an_object(tmp_path: Path) -> None:
    paths = _runtime_paths(tmp_path)
    paths.settings_file.write_text("[]", encoding="utf-8")

    with pytest.raises(SettingsLoadError, match="JSON 객체"):
        load_settings(paths)
