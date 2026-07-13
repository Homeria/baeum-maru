import json
from pathlib import Path

import pytest

from app.core.runtime import RuntimePaths
from app.core.settings import SettingsLoadError, load_settings


def test_settings_use_defaults_without_files(tmp_path: Path) -> None:
    paths = _runtime_paths(tmp_path)

    settings = load_settings(paths)

    assert settings.environment == "development"
    assert settings.server.host == "127.0.0.1"
    assert settings.server.port == 18080
    assert settings.logging.level == "INFO"


def test_environment_overrides_json_without_replacing_sibling_values(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    paths = _runtime_paths(tmp_path)
    paths.settings_file.write_text(
        json.dumps(
            {
                "environment": "production",
                "server": {"host": "0.0.0.0", "port": 18080},
                "logging": {"level": "WARNING"},
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setenv("BAEUM_MARU_SERVER__PORT", "19090")
    monkeypatch.setenv("BAEUM_MARU_LOGGING__LEVEL", "debug")

    settings = load_settings(paths)

    assert settings.environment == "production"
    assert settings.server.host == "0.0.0.0"
    assert settings.server.port == 19090
    assert settings.logging.level == "DEBUG"


def test_runtime_dotenv_overrides_json(tmp_path: Path) -> None:
    paths = _runtime_paths(tmp_path)
    paths.settings_file.write_text('{"server":{"port":18080}}', encoding="utf-8")
    paths.env_file.write_text("BAEUM_MARU_SERVER__PORT=20080\n", encoding="utf-8")

    settings = load_settings(paths)

    assert settings.server.port == 20080


def test_os_environment_overrides_runtime_dotenv(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    paths = _runtime_paths(tmp_path)
    paths.env_file.write_text("BAEUM_MARU_SERVER__PORT=20080\n", encoding="utf-8")
    monkeypatch.setenv("BAEUM_MARU_SERVER__PORT", "21080")

    settings = load_settings(paths)

    assert settings.server.port == 21080


def test_invalid_environment_value_fails_at_startup(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    paths = _runtime_paths(tmp_path)
    monkeypatch.setenv("BAEUM_MARU_SERVER__PORT", "not-a-port")

    with pytest.raises(SettingsLoadError):
        load_settings(paths)


@pytest.mark.parametrize(
    "content",
    (
        "not-json",
        "[]",
        '{"server":{"port":80}}',
        '{"unknown":true}',
    ),
)
def test_invalid_settings_fail_at_startup(tmp_path: Path, content: str) -> None:
    paths = _runtime_paths(tmp_path)
    paths.settings_file.write_text(content, encoding="utf-8")

    with pytest.raises(SettingsLoadError):
        load_settings(paths)


def _runtime_paths(tmp_path: Path) -> RuntimePaths:
    paths = RuntimePaths.discover(tmp_path / "runtime")
    paths.ensure_directories()
    return paths
