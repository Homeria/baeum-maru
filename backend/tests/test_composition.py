from dataclasses import FrozenInstanceError
from pathlib import Path

import pytest

from app.composition import compose_application
from app.modules.system.public import GetHealthQuery


def test_composition_builds_explicit_module_services(tmp_path: Path) -> None:
    container = compose_application(tmp_path / "runtime")

    result = container.system.get_health.fetch(GetHealthQuery())

    assert result.status == "ok"
    assert result.service == "baeum-maru-api"


def test_application_container_is_immutable(tmp_path: Path) -> None:
    container = compose_application(tmp_path / "runtime")

    with pytest.raises(FrozenInstanceError):
        container.runtime = container.runtime  # type: ignore[misc]
