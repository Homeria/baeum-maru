"""승인된 SQLite table/index SQL fingerprint와 migration 결과를 비교한다."""

from hashlib import sha256
from pathlib import Path

from app.db.database import Database

SNAPSHOT_FILE = Path(__file__).parent / "snapshots" / "sqlite_schema.sha256"


def schema_fingerprint(database: Database) -> str:
    with database.connection() as connection:
        rows = connection.execute(
            "SELECT type, name, tbl_name, sql FROM sqlite_master "
            "WHERE sql IS NOT NULL "
            "AND name NOT LIKE 'sqlite_%' "
            "ORDER BY CASE type WHEN 'table' THEN 0 ELSE 1 END, name"
        ).fetchall()

    lines: list[str] = []
    for object_type, name, table_name, sql in rows:
        normalized_sql = " ".join(str(sql).split())
        digest = sha256(normalized_sql.encode("utf-8")).hexdigest()
        lines.append(f"{object_type} {name} {table_name} {digest}")
    return "\n".join(lines) + "\n"


def test_sqlite_schema_matches_approved_snapshot(initialized_database: Database) -> None:
    expected = SNAPSHOT_FILE.read_text(encoding="utf-8")

    assert schema_fingerprint(initialized_database) == expected
