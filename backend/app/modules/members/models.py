from sqlalchemy import (
    Boolean,
    CheckConstraint,
    ForeignKey,
    Index,
    Integer,
    String,
    Text,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin


class GenderCode(Base):
    __tablename__ = "gender_codes"

    code: Mapped[str] = mapped_column(String(16), primary_key=True)
    label: Mapped[str] = mapped_column(String(40), unique=True, nullable=False)
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)


class Member(TimestampVersionMixin, Base):
    __tablename__ = "members"
    __table_args__ = (
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_members_name", "name"),
        Index("ix_members_phone", "phone"),
        Index("ix_members_is_active", "is_active"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    member_no: Mapped[str] = mapped_column(String(40), unique=True, nullable=False)
    name: Mapped[str] = mapped_column(String(80), nullable=False)
    gender_code: Mapped[str] = mapped_column(
        ForeignKey("gender_codes.code", ondelete="RESTRICT"),
        default="unknown",
        server_default="unknown",
        nullable=False,
    )
    phone: Mapped[str] = mapped_column(String(20), nullable=False)
    note: Mapped[str | None] = mapped_column(Text)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )
