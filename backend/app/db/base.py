"""SQLAlchemy Declarative Base와 공통 변경 추적 컬럼을 정의한다."""

from datetime import UTC, datetime

from sqlalchemy import DateTime, Integer, MetaData, func
from sqlalchemy.orm import DeclarativeBase, Mapped, declared_attr, mapped_column

NAMING_CONVENTION = {
    "ix": "ix_%(table_name)s_%(column_0_N_name)s",
    "uq": "uq_%(table_name)s_%(column_0_N_name)s",
    "ck": "ck_%(table_name)s_%(constraint_name)s",
    "fk": "fk_%(table_name)s_%(column_0_N_name)s_%(referred_table_name)s",
    "pk": "pk_%(table_name)s",
}


class Base(DeclarativeBase):
    """모든 SQLAlchemy table model이 상속하는 기준 클래스."""

    metadata = MetaData(naming_convention=NAMING_CONVENTION)


def utc_now() -> datetime:
    return datetime.now(UTC)


class TimestampVersionMixin:
    """수정 가능한 엔티티에 UTC 시각과 optimistic version을 제공한다."""

    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=utc_now,
        server_default=func.current_timestamp(),
        nullable=False,
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        default=utc_now,
        onupdate=utc_now,
        server_default=func.current_timestamp(),
        nullable=False,
    )
    version: Mapped[int] = mapped_column(
        Integer,
        default=1,
        server_default="1",
        nullable=False,
    )

    @declared_attr.directive
    def __mapper_args__(cls) -> dict[str, object]:
        return {"version_id_col": cls.version}
