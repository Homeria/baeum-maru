from datetime import datetime

from sqlalchemy import (
    CheckConstraint,
    DateTime,
    ForeignKey,
    Index,
    Integer,
    String,
    Text,
    UniqueConstraint,
    func,
)
from sqlalchemy.orm import Mapped, mapped_column

from app.db.base import Base
from app.db.constraints import CAPACITY_CHECK


class LotteryRun(Base):
    __tablename__ = "lottery_runs"
    __table_args__ = (
        CheckConstraint(
            "status IN ('prepared', 'running', 'completed', 'failed', 'cancelled')",
            name="status_allowed",
        ),
        Index("ix_lottery_runs_term_status", "term_id", "status"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    term_id: Mapped[int] = mapped_column(
        ForeignKey("terms.id", ondelete="RESTRICT"),
        nullable=False,
    )
    seed: Mapped[int] = mapped_column(Integer, nullable=False)
    status: Mapped[str] = mapped_column(
        String(16),
        default="prepared",
        server_default="prepared",
        nullable=False,
    )
    executed_by_user_id: Mapped[int] = mapped_column(
        ForeignKey("users.id", ondelete="RESTRICT"),
        nullable=False,
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.current_timestamp(),
        nullable=False,
    )
    started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    note: Mapped[str | None] = mapped_column(Text)


class LotteryRunTarget(Base):
    __tablename__ = "lottery_run_targets"
    __table_args__ = (
        UniqueConstraint("lottery_run_id", "offering_id"),
        CheckConstraint(
            "capacity_type IN ('fixed', 'open', 'gender_split')",
            name="capacity_type_allowed",
        ),
        CheckConstraint(CAPACITY_CHECK, name="capacity_shape_valid"),
        CheckConstraint("eligible_count >= 0", name="eligible_count_nonnegative"),
        Index("ix_lottery_run_targets_run_id", "lottery_run_id"),
        Index("ix_lottery_run_targets_offering_id", "offering_id"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    lottery_run_id: Mapped[int] = mapped_column(
        ForeignKey("lottery_runs.id", ondelete="CASCADE"),
        nullable=False,
    )
    offering_id: Mapped[int] = mapped_column(
        ForeignKey("course_offerings.id", ondelete="RESTRICT"),
        nullable=False,
    )
    capacity_type: Mapped[str] = mapped_column(String(20), nullable=False)
    capacity_total: Mapped[int | None] = mapped_column(Integer)
    male_capacity: Mapped[int | None] = mapped_column(Integer)
    female_capacity: Mapped[int | None] = mapped_column(Integer)
    eligible_count: Mapped[int] = mapped_column(Integer, nullable=False)


class LotteryResult(Base):
    __tablename__ = "lottery_results"
    __table_args__ = (
        UniqueConstraint("lottery_run_target_id", "registration_id"),
        UniqueConstraint("lottery_run_target_id", "result", "result_order"),
        CheckConstraint(
            "result IN ('selected', 'waitlisted', 'rejected')",
            name="result_allowed",
        ),
        CheckConstraint("result_order >= 1", name="result_order_positive"),
        Index("ix_lottery_results_target_id", "lottery_run_target_id"),
        Index("ix_lottery_results_registration_id", "registration_id"),
    )

    id: Mapped[int] = mapped_column(Integer, primary_key=True)
    lottery_run_target_id: Mapped[int] = mapped_column(
        ForeignKey("lottery_run_targets.id", ondelete="CASCADE"),
        nullable=False,
    )
    registration_id: Mapped[int] = mapped_column(
        ForeignKey("registrations.id", ondelete="RESTRICT"),
        nullable=False,
    )
    result: Mapped[str] = mapped_column(String(16), nullable=False)
    result_order: Mapped[int] = mapped_column(Integer, nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True),
        server_default=func.current_timestamp(),
        nullable=False,
    )
