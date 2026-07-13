from datetime import date, datetime, time

from sqlalchemy import (
    Boolean,
    CheckConstraint,
    Date,
    DateTime,
    ForeignKey,
    Index,
    Integer,
    String,
    Text,
    Time,
    UniqueConstraint,
    text,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin
from app.db.constraints import CAPACITY_CHECK


class CourseCategory(TimestampVersionMixin, Base):
    __tablename__ = "course_categories"
    __table_args__ = (CheckConstraint("version >= 1", name="version_positive"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(100), unique=True, nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class Course(TimestampVersionMixin, Base):
    __tablename__ = "courses"
    __table_args__ = (
        UniqueConstraint("category_id", "name"),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_courses_category_active", "category_id", "is_active"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    category_id: Mapped[int] = mapped_column(
        ForeignKey("course_categories.id", ondelete="RESTRICT"),
        nullable=False,
    )
    name: Mapped[str] = mapped_column(String(160), nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class Instructor(TimestampVersionMixin, Base):
    __tablename__ = "instructors"
    __table_args__ = (CheckConstraint("version >= 1", name="version_positive"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(80), nullable=False)
    phone: Mapped[str | None] = mapped_column(String(20))
    note: Mapped[str | None] = mapped_column(Text)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class Term(TimestampVersionMixin, Base):
    __tablename__ = "terms"
    __table_args__ = (
        CheckConstraint(
            "starts_on IS NULL OR ends_on IS NULL OR starts_on <= ends_on",
            name="date_range_valid",
        ),
        CheckConstraint(
            "registration_opens_at IS NULL OR registration_closes_at IS NULL "
            "OR registration_opens_at < registration_closes_at",
            name="registration_range_valid",
        ),
        CheckConstraint(
            "max_registrations_per_member >= 0",
            name="registration_limit_nonnegative",
        ),
        CheckConstraint(
            "status IN ('draft', 'open', 'closed', 'finalized')",
            name="status_allowed",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(120), unique=True, nullable=False)
    starts_on: Mapped[date | None] = mapped_column(Date)
    ends_on: Mapped[date | None] = mapped_column(Date)
    registration_opens_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    registration_closes_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    max_registrations_per_member: Mapped[int] = mapped_column(
        Integer,
        default=0,
        server_default="0",
        nullable=False,
    )
    status: Mapped[str] = mapped_column(
        String(16),
        default="draft",
        server_default="draft",
        nullable=False,
    )


class TimeSlot(TimestampVersionMixin, Base):
    __tablename__ = "time_slots"
    __table_args__ = (
        UniqueConstraint("start_time", "end_time"),
        CheckConstraint("start_time < end_time", name="time_range_valid"),
        CheckConstraint("version >= 1", name="version_positive"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(80), unique=True, nullable=False)
    start_time: Mapped[time] = mapped_column(Time, nullable=False)
    end_time: Mapped[time] = mapped_column(Time, nullable=False)
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class CourseOffering(TimestampVersionMixin, Base):
    __tablename__ = "course_offerings"
    __table_args__ = (
        CheckConstraint(
            "section_label IS NULL OR length(trim(section_label)) > 0",
            name="section_not_blank",
        ),
        CheckConstraint(
            "capacity_type IN ('fixed', 'open', 'gender_split')",
            name="capacity_type_allowed",
        ),
        CheckConstraint(CAPACITY_CHECK, name="capacity_shape_valid"),
        CheckConstraint(
            "status IN ('draft', 'open', 'closed', 'cancelled')",
            name="status_allowed",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_course_offerings_term_status", "term_id", "status"),
        Index("ix_course_offerings_course_id", "course_id"),
        Index("ix_course_offerings_instructor_id", "instructor_id"),
        Index(
            "uq_course_offerings_term_course_no_section",
            "term_id",
            "course_id",
            unique=True,
            sqlite_where=text("section_label IS NULL"),
        ),
        Index(
            "uq_course_offerings_term_course_section",
            "term_id",
            "course_id",
            "section_label",
            unique=True,
            sqlite_where=text("section_label IS NOT NULL"),
        ),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    term_id: Mapped[int] = mapped_column(
        ForeignKey("terms.id", ondelete="RESTRICT"), nullable=False
    )
    course_id: Mapped[int] = mapped_column(
        ForeignKey("courses.id", ondelete="RESTRICT"),
        nullable=False,
    )
    section_label: Mapped[str | None] = mapped_column(String(80))
    instructor_id: Mapped[int | None] = mapped_column(
        ForeignKey("instructors.id", ondelete="RESTRICT")
    )
    capacity_type: Mapped[str] = mapped_column(
        String(20),
        default="fixed",
        server_default="fixed",
        nullable=False,
    )
    capacity_total: Mapped[int | None] = mapped_column(Integer)
    male_capacity: Mapped[int | None] = mapped_column(Integer)
    female_capacity: Mapped[int | None] = mapped_column(Integer)
    status: Mapped[str] = mapped_column(
        String(16),
        default="draft",
        server_default="draft",
        nullable=False,
    )
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    note: Mapped[str | None] = mapped_column(Text)


class CourseSchedule(Base):
    __tablename__ = "course_schedules"
    __table_args__ = (
        UniqueConstraint("offering_id", "weekday", "time_slot_id"),
        CheckConstraint("weekday BETWEEN 1 AND 7", name="weekday_range"),
        Index("ix_course_schedules_offering_id", "offering_id"),
        Index("ix_course_schedules_weekday_slot", "weekday", "time_slot_id"),
        Index("ix_course_schedules_location_id", "location_id"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    offering_id: Mapped[int] = mapped_column(
        ForeignKey("course_offerings.id", ondelete="CASCADE"),
        nullable=False,
    )
    weekday: Mapped[int] = mapped_column(Integer, nullable=False)
    time_slot_id: Mapped[int] = mapped_column(
        ForeignKey("time_slots.id", ondelete="RESTRICT"),
        nullable=False,
    )
    location_id: Mapped[int] = mapped_column(
        ForeignKey("locations.id", ondelete="RESTRICT"),
        nullable=False,
    )
