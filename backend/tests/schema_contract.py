from __future__ import annotations

import json
from pathlib import Path
from typing import cast

from sqlalchemy.engine.reflection import Inspector

SCHEMA_CONTRACT_PATH = Path(__file__).parent / "contracts" / "sqlite_schema.json"


def build_sqlite_schema_contract(inspector: Inspector) -> dict[str, object]:
    tables: dict[str, object] = {}

    for table_name in sorted(inspector.get_table_names()):
        if table_name == "alembic_version":
            continue

        columns = [
            {
                "name": column["name"],
                "type": str(column["type"]),
                "nullable": column["nullable"],
                "default": _normalize_sql(column.get("default")),
            }
            for column in inspector.get_columns(table_name)
        ]
        primary_key = inspector.get_pk_constraint(table_name)
        foreign_keys: list[dict[str, object]] = [
            {
                "name": foreign_key["name"],
                "columns": foreign_key["constrained_columns"],
                "referred_table": foreign_key["referred_table"],
                "referred_columns": foreign_key["referred_columns"],
                "ondelete": foreign_key.get("options", {}).get("ondelete"),
            }
            for foreign_key in inspector.get_foreign_keys(table_name)
        ]
        unique_constraints: list[dict[str, object]] = [
            {
                "name": constraint["name"],
                "columns": constraint["column_names"],
            }
            for constraint in inspector.get_unique_constraints(table_name)
        ]
        indexes: list[dict[str, object]] = [
            {
                "name": index["name"],
                "columns": index["column_names"],
                "unique": bool(index["unique"]),
                "where": _normalize_sql(index.get("dialect_options", {}).get("sqlite_where")),
            }
            for index in inspector.get_indexes(table_name)
        ]
        checks: list[dict[str, object]] = [
            {
                "name": check["name"],
                "sql": _normalize_sql(check["sqltext"]),
            }
            for check in inspector.get_check_constraints(table_name)
        ]
        foreign_keys.sort(key=lambda item: str(item["name"]))
        unique_constraints.sort(key=lambda item: str(item["name"]))
        indexes.sort(key=lambda item: str(item["name"]))
        checks.sort(key=lambda item: str(item["name"]))

        tables[table_name] = {
            "columns": columns,
            "primary_key": {
                "name": primary_key["name"],
                "columns": primary_key["constrained_columns"],
            },
            "foreign_keys": foreign_keys,
            "unique_constraints": unique_constraints,
            "indexes": indexes,
            "checks": checks,
        }

    return {"format_version": 1, "tables": tables}


def load_sqlite_schema_contract() -> dict[str, object]:
    return cast(
        dict[str, object],
        json.loads(SCHEMA_CONTRACT_PATH.read_text(encoding="utf-8")),
    )


def write_sqlite_schema_contract(contract: dict[str, object]) -> None:
    SCHEMA_CONTRACT_PATH.parent.mkdir(parents=True, exist_ok=True)
    SCHEMA_CONTRACT_PATH.write_text(
        json.dumps(contract, ensure_ascii=False, indent=2) + "\n",
        encoding="utf-8",
    )


def _normalize_sql(value: object) -> str | None:
    if value is None:
        return None
    return " ".join(str(value).split())
