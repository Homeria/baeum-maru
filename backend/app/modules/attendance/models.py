from datetime import date

from sqlalchemy import (
    CheckConstraint,
    Date,
    ForeignKey,
    Index,
    Integer,
    String,
    Text,
    UniqueConstraint,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin


class AttendanceSession(TimestampVersionMixin, Base):
    __tablename__ = "attendance_sessions"
    __table_args__ = (
        UniqueConstraint("offering_id", "session_date", "sequence_no"),
        CheckConstraint("sequence_no >= 1", name="sequence_positive"),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_attendance_sessions_offering_date", "offering_id", "session_date"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    offering_id: Mapped[int] = mapped_column(
        ForeignKey("course_offerings.id", ondelete="RESTRICT"),
        nullable=False,
    )
    schedule_id: Mapped[int | None] = mapped_column(
        ForeignKey("course_schedules.id", ondelete="RESTRICT")
    )
    session_date: Mapped[date] = mapped_column(Date, nullable=False)
    sequence_no: Mapped[int] = mapped_column(
        Integer,
        default=1,
        server_default="1",
        nullable=False,
    )
    note: Mapped[str | None] = mapped_column(Text)


class AttendanceRecord(TimestampVersionMixin, Base):
    __tablename__ = "attendance_records"
    __table_args__ = (
        UniqueConstraint("attendance_session_id", "registration_id"),
        CheckConstraint(
            "status IN ('present', 'absent', 'late', 'excused')",
            name="status_allowed",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_attendance_records_registration_id", "registration_id"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    attendance_session_id: Mapped[int] = mapped_column(
        ForeignKey("attendance_sessions.id", ondelete="CASCADE"),
        nullable=False,
    )
    registration_id: Mapped[int] = mapped_column(
        ForeignKey("registrations.id", ondelete="RESTRICT"),
        nullable=False,
    )
    status: Mapped[str] = mapped_column(String(16), nullable=False)
    note: Mapped[str | None] = mapped_column(Text)
