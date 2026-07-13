"""Declarative Base의 metadata 규칙을 검증한다."""

from app.db.base import NAMING_CONVENTION, Base


def test_base_uses_stable_constraint_names_for_alembic() -> None:
    assert Base.metadata.naming_convention == NAMING_CONVENTION
    assert NAMING_CONVENTION == {
        "ix": "ix_%(table_name)s_%(column_0_N_name)s",
        "uq": "uq_%(table_name)s_%(column_0_N_name)s",
        "ck": "ck_%(table_name)s_%(constraint_name)s",
        "fk": "fk_%(table_name)s_%(column_0_N_name)s_%(referred_table_name)s",
        "pk": "pk_%(table_name)s",
    }
