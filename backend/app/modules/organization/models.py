from sqlalchemy import CheckConstraint, Integer, String
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base, TimestampVersionMixin


class OrganizationSettings(TimestampVersionMixin, Base):
    __tablename__ = "organization_settings"
    __table_args__ = (
        CheckConstraint("id = 1", name="singleton_id"),
        CheckConstraint("default_max_registrations >= 0", name="default_limit_nonnegative"),
        CheckConstraint(
            "default_access_code_ttl_minutes BETWEEN 1 AND 10080",
            name="access_code_ttl_range",
        ),
        CheckConstraint("version >= 1", name="version_positive"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    organization_name: Mapped[str] = mapped_column(String(120), nullable=False)
    logo_relative_path: Mapped[str | None] = mapped_column(String(260))
    default_max_registrations: Mapped[int] = mapped_column(
        Integer,
        default=4,
        server_default="4",
        nullable=False,
    )
    default_access_code_ttl_minutes: Mapped[int] = mapped_column(
        Integer,
        default=480,
        server_default="480",
        nullable=False,
    )
