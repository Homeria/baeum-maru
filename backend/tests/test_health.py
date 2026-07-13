from pathlib import Path

import httpx
import pytest

from app.main import create_app


@pytest.mark.asyncio
async def test_health_endpoint(tmp_path: Path) -> None:
    app = create_app(runtime_dir=tmp_path / "runtime")
    transport = httpx.ASGITransport(app=app)

    async with (
        app.router.lifespan_context(app),
        httpx.AsyncClient(transport=transport, base_url="http://test") as client,
    ):
        response = await client.get("/api/v1/health")

    assert response.status_code == 200
    assert response.json() == {
        "status": "ok",
        "service": "baeum-maru-api",
    }
