"""portable runtime 경로 탐색과 디렉터리 생성을 검증한다."""

from pathlib import Path

import pytest

from app.core.runtime import RUNTIME_DIR_ENV, RuntimePaths, distribution_root


def test_explicit_runtime_path_supports_korean_and_spaces(tmp_path: Path) -> None:
    runtime_root = tmp_path / "기관 운영 자료" / "배움마루"

    paths = RuntimePaths.discover(runtime_root)
    paths.ensure_directories()

    assert paths.root == runtime_root.resolve()
    assert paths.config_dir.is_dir()
    assert paths.data_dir.is_dir()
    assert paths.logs_dir.is_dir()
    assert paths.settings_file == runtime_root.resolve() / "config" / "settings.json"


def test_environment_variable_overrides_default_runtime_path(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    configured = tmp_path / "custom-runtime"
    monkeypatch.setenv(RUNTIME_DIR_ENV, str(configured))

    paths = RuntimePaths.discover()

    assert paths.root == configured.resolve()


def test_relative_runtime_path_is_based_on_distribution_root(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv(RUNTIME_DIR_ENV, raising=False)

    paths = RuntimePaths.discover("local-data")

    assert paths.root == (distribution_root() / "local-data").resolve()
