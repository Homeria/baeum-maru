from datetime import datetime
from typing import Any

from sqlalchemy import (
    JSON,
    CheckConstraint,
    DateTime,
    ForeignKey,
    Index,
    Integer,
    String,
    UniqueConstraint,
    func,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin

REGISTRATION_STATUSES = "'applied', 'selected', 'waitlisted', 'rejected', 'confirmed', 'cancelled'"


class Registration(TimestampVersionMixin, Base):
    __tablename__ = "registrations"
    __table_args__ = (
        UniqueConstraint("member_id", "offering_id"),
        CheckConstraint(
            f"status IN ({REGISTRATION_STATUSES})",
            name="status_allowed",
        ),
        CheckConstraint(
            "(status = 'cancelled' AND cancelled_at IS NOT NULL) "
            "OR (status <> 'cancelled' AND cancelled_at IS NULL)",
            name="cancelled_at_matches_status",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_registrations_member_id", "member_id"),
        Index("ix_registrations_offering_status", "offering_id", "status"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    member_id: Mapped[int] = mapped_column(
        ForeignKey("members.id", ondelete="RESTRICT"),
        nullable=False,
    )
    offering_id: Mapped[int] = mapped_column(
        ForeignKey("course_offerings.id", ondelete="RESTRICT"),
        nullable=False,
    )
    status: Mapped[str] = mapped_column(
        String(16),
        default="applied",
        server_default="applied",
        nullable=False,
    )
    cancelled_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))


class RegistrationStatusHistory(Base):
    __tablename__ = "registration_status_history"
    __table_args__ = (
        CheckConstraint(
            f"from_status IS NULL OR from_status IN ({REGISTRATION_STATUSES})",
            name="from_status_allowed",
        ),
        CheckConstraint(
            f"to_status IN ({REGISTRATION_STATUSES})",
            name="to_status_allowed",
        ),
        CheckConstraint(
            "actor_kind IN ('user', 'launcher', 'system')",
            name="actor_kind_allowed",
        ),
        Index(
            "ix_registration_status_history_registration_changed",
            "registration_id",
            "changed_at",
        ),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    registration_id: Mapped[int] = mapped_column(
        ForeignKey("registrations.id", ondelete="CASCADE"),
        nullable=False,
    )
    from_status: Mapped[str | None] = mapped_column(String(16))
    to_status: Mapped[str] = mapped_column(String(16), nullable=False)
    reason: Mapped[str | None] = mapped_column(String(255))
    actor_kind: Mapped[str] = mapped_column(String(16), nullable=False)
    actor_user_id: Mapped[int | None] = mapped_column(ForeignKey("users.id", ondelete="RESTRICT"))
    actor_access_code_id: Mapped[int | None] = mapped_column(
        ForeignKey("access_codes.id", ondelete="RESTRICT")
    )
    actor_display_name: Mapped[str | None] = mapped_column(String(80))
    metadata_json: Mapped[dict[str, Any] | None] = mapped_column(JSON)
    changed_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.current_timestamp(),
        nullable=False,
    )
