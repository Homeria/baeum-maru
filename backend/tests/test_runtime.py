from pathlib import Path

import pytest

from app.core.runtime import RUNTIME_DIR_ENV, RuntimePaths


def test_runtime_paths_create_managed_directories(tmp_path: Path) -> None:
    runtime_root = tmp_path / "한글 사용자" / "배움마루 운영 자료"
    paths = RuntimePaths.discover(runtime_root)

    paths.ensure_directories()

    assert paths.root == runtime_root.resolve()
    assert all(directory.is_dir() for directory in paths.managed_directories)
    assert paths.settings_file == paths.root / "config" / "settings.json"
    assert paths.database_file == paths.root / "data" / "baeum-maru.db"
    assert paths.log_file == paths.root / "logs" / "baeum-maru.log"


def test_runtime_paths_accept_environment_override(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    environment = tmp_path / "environment-runtime"
    monkeypatch.setenv(RUNTIME_DIR_ENV, str(environment))

    paths = RuntimePaths.discover()

    assert paths.root == environment.resolve()
