"""건물, 층, 공간과 사용자 정의 공간 역할 SQLAlchemy 모델."""

from sqlalchemy import (
    Boolean,
    CheckConstraint,
    ForeignKey,
    Index,
    Integer,
    String,
    Text,
    UniqueConstraint,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin


class Building(TimestampVersionMixin, Base):
    __tablename__ = "buildings"
    __table_args__ = (CheckConstraint("version >= 1", name="version_positive"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(120), unique=True, nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class BuildingFloor(TimestampVersionMixin, Base):
    __tablename__ = "building_floors"
    __table_args__ = (
        UniqueConstraint("building_id", "label"),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_building_floors_building_id", "building_id"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    building_id: Mapped[int] = mapped_column(
        ForeignKey("buildings.id", ondelete="CASCADE"), nullable=False
    )
    label: Mapped[str] = mapped_column(String(80), nullable=False)
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class LocationRole(TimestampVersionMixin, Base):
    __tablename__ = "location_roles"
    __table_args__ = (CheckConstraint("version >= 1", name="version_positive"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    name: Mapped[str] = mapped_column(String(80), unique=True, nullable=False)
    is_course_eligible: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_course_eligible_bool"),
        default=False,
        server_default="0",
        nullable=False,
    )
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class Location(TimestampVersionMixin, Base):
    __tablename__ = "locations"
    __table_args__ = (
        UniqueConstraint("building_floor_id", "name"),
        CheckConstraint("version >= 1", name="version_positive"),
        Index("ix_locations_floor_active", "building_floor_id", "is_active"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    building_floor_id: Mapped[int] = mapped_column(
        ForeignKey("building_floors.id", ondelete="RESTRICT"), nullable=False
    )
    name: Mapped[str] = mapped_column(String(120), nullable=False)
    description: Mapped[str | None] = mapped_column(Text)
    sort_order: Mapped[int] = mapped_column(Integer, default=0, server_default="0", nullable=False)
    is_active: Mapped[bool] = mapped_column(
        Boolean(create_constraint=True, name="is_active_bool"),
        default=True,
        server_default="1",
        nullable=False,
    )


class LocationRoleAssignment(Base):
    __tablename__ = "location_role_assignments"
    __table_args__ = (Index("ix_location_role_assignments_role_id", "role_id"),)

    location_id: Mapped[int] = mapped_column(
        ForeignKey("locations.id", ondelete="CASCADE"), primary_key=True
    )
    role_id: Mapped[int] = mapped_column(
        ForeignKey("location_roles.id", ondelete="CASCADE"), primary_key=True
    )
