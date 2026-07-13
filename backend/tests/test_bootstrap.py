from pathlib import Path
from typing import cast

import pytest

from app.core.bootstrap import RuntimeContext
from app.main import create_app


@pytest.mark.asyncio
async def test_lifespan_bootstraps_runtime_and_writes_lifecycle_logs(tmp_path: Path) -> None:
    app = create_app(runtime_dir=tmp_path / "runtime")

    async with app.router.lifespan_context(app):
        runtime = cast(RuntimeContext, app.state.runtime)
        assert runtime.paths.database_file.parent.is_dir()
        assert runtime.settings.server.port == 18080

    log_content = runtime.paths.log_file.read_text(encoding="utf-8")
    assert "runtime.initialized" in log_content
    assert "application.started" in log_content
    assert "application.stopped" in log_content
