"""sqlite3 schema 집계 순서를 검증한다."""

from app.db.schema import SCHEMA_MODULES, SCHEMA_STATEMENTS


def test_schema_collects_domain_statements_in_dependency_order() -> None:
    assert len(SCHEMA_MODULES) == 9
    assert SCHEMA_STATEMENTS
    assert "organization_settings" in SCHEMA_STATEMENTS[0]
