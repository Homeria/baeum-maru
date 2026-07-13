from datetime import datetime

from sqlalchemy import (
    Boolean,
    CheckConstraint,
    DateTime,
    ForeignKey,
    Index,
    Integer,
    String,
    Text,
    func,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin


class User(TimestampVersionMixin, Base):
    __tablename__ = "users"
    __table_args__ = (
        CheckConstraint(
            "role IN ('staff', 'temporary_staff', 'viewer')",
            name="role_allowed",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_users_role_is_active", "role", "is_active"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    display_name: Mapped[str] = mapped_column(String(80), nullable=False)
    affiliation: Mapped[str | None] = mapped_column(String(120))
    contact_note: Mapped[str | None] = mapped_column(String(120))
    role: Mapped[str] = mapped_column(String(24), nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class AccessCode(TimestampVersionMixin, Base):
    __tablename__ = "access_codes"
    __table_args__ = (
        CheckConstraint("expires_at > issued_at", name="expiry_after_issue"),
        CheckConstraint(
            "revoked_at IS NULL OR revoked_at >= issued_at",
            name="revoked_after_issue",
        ),
        CheckConstraint(
            "hidden_at IS NULL OR hidden_at >= issued_at",
            name="hidden_after_issue",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_access_codes_user_id", "user_id"),
        Index(
            "ix_access_codes_lifecycle",
            "expires_at",
            "revoked_at",
            "hidden_at",
        ),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    user_id: Mapped[int] = mapped_column(
        ForeignKey("users.id", ondelete="RESTRICT"),
        nullable=False,
    )
    code_hash: Mapped[str] = mapped_column(String(255), unique=True, nullable=False)
    display_code: Mapped[str] = mapped_column(String(32), unique=True, nullable=False)
    label: Mapped[str | None] = mapped_column(String(120))
    issued_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.current_timestamp(),
        nullable=False,
    )
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    revoked_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    hidden_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    last_used_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    note: Mapped[str | None] = mapped_column(Text)


class UserSession(Base):
    __tablename__ = "user_sessions"
    __table_args__ = (
        CheckConstraint("expires_at > issued_at", name="expiry_after_issue"),
        CheckConstraint(
            "last_seen_at >= issued_at",
            name="last_seen_after_issue",
        ),
        CheckConstraint(
            "revoked_at IS NULL OR revoked_at >= issued_at",
            name="revoked_after_issue",
        ),
        Index("ix_user_sessions_user_id", "user_id"),
        Index("ix_user_sessions_access_code_id", "access_code_id"),
        Index("ix_user_sessions_lifecycle", "expires_at", "revoked_at"),
    )

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    user_id: Mapped[int] = mapped_column(
        ForeignKey("users.id", ondelete="RESTRICT"),
        nullable=False,
    )
    access_code_id: Mapped[int] = mapped_column(
        ForeignKey("access_codes.id", ondelete="RESTRICT"),
        nullable=False,
    )
    token_hash: Mapped[str] = mapped_column(String(255), unique=True, nullable=False)
    issued_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.current_timestamp(),
        nullable=False,
    )
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    last_seen_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    revoked_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
