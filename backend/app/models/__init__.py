"""Alembic metadata에 모든 도메인 SQLAlchemy 모델을 등록한다."""

from types import ModuleType

from app.models import (
    attendance,
    courses,
    identity,
    locations,
    lottery,
    members,
    operations,
    organization,
    registrations,
)

MODEL_MODULES: tuple[ModuleType, ...] = (
    organization,
    identity,
    locations,
    courses,
    members,
    registrations,
    lottery,
    attendance,
    operations,
)
