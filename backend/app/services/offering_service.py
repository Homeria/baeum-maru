"""개설 강좌와 시간표 CRUD 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ResourceNotFoundError
from app.repositories import course_repository as course_repo
from app.repositories import offering_repository as offering_repo
from app.repositories import space_repository as space_repo

# --- course_offerings (개설 강좌) ---


def _ensure_offering_refs(course_id: int, instructor_id: int | None) -> None:
    if course_repo.get_course(course_id) is None:
        raise ResourceNotFoundError("course_not_found", "과목을 찾을 수 없습니다.")
    if instructor_id is not None and course_repo.get_instructor(instructor_id) is None:
        raise ResourceNotFoundError("instructor_not_found", "강사를 찾을 수 없습니다.")


def list_offerings() -> list[dict[str, Any]]:
    return offering_repo.list_offerings()


def get_offering(offering_id: int) -> dict[str, Any]:
    offering = offering_repo.get_offering(offering_id)
    if offering is None:
        raise ResourceNotFoundError("offering_not_found", "개설 강좌를 찾을 수 없습니다.")
    return offering


def create_offering(
    *,
    course_id: int,
    section_label: str | None,
    instructor_id: int | None,
    capacity_type: str,
    capacity_total: int | None,
    male_capacity: int | None,
    female_capacity: int | None,
    status: str,
    sort_order: int,
) -> dict[str, Any]:
    _ensure_offering_refs(course_id, instructor_id)
    return offering_repo.create_offering(
        course_id=course_id,
        section_label=section_label,
        instructor_id=instructor_id,
        capacity_type=capacity_type,
        capacity_total=capacity_total,
        male_capacity=male_capacity,
        female_capacity=female_capacity,
        status=status,
        sort_order=sort_order,
    )


def update_offering(
    offering_id: int,
    *,
    course_id: int,
    section_label: str | None,
    instructor_id: int | None,
    capacity_type: str,
    capacity_total: int | None,
    male_capacity: int | None,
    female_capacity: int | None,
    status: str,
    sort_order: int,
) -> dict[str, Any]:
    _ensure_offering_refs(course_id, instructor_id)
    offering = offering_repo.update_offering(
        offering_id,
        course_id=course_id,
        section_label=section_label,
        instructor_id=instructor_id,
        capacity_type=capacity_type,
        capacity_total=capacity_total,
        male_capacity=male_capacity,
        female_capacity=female_capacity,
        status=status,
        sort_order=sort_order,
    )
    if offering is None:
        raise ResourceNotFoundError("offering_not_found", "개설 강좌를 찾을 수 없습니다.")
    return offering


def delete_offering(offering_id: int) -> None:
    if not offering_repo.delete_offering(offering_id):
        raise ResourceNotFoundError("offering_not_found", "개설 강좌를 찾을 수 없습니다.")


# --- course_schedules (시간표) ---


def _ensure_schedule_refs(time_slot_id: int, space_id: int) -> None:
    if course_repo.get_time_slot(time_slot_id) is None:
        raise ResourceNotFoundError("time_slot_not_found", "교시를 찾을 수 없습니다.")
    if space_repo.get_space(space_id) is None:
        raise ResourceNotFoundError("space_not_found", "장소를 찾을 수 없습니다.")


def list_schedules(offering_id: int) -> list[dict[str, Any]]:
    get_offering(offering_id)  # 존재 확인
    return offering_repo.list_schedules(offering_id)


def get_schedule(schedule_id: int) -> dict[str, Any]:
    schedule = offering_repo.get_schedule(schedule_id)
    if schedule is None:
        raise ResourceNotFoundError("schedule_not_found", "시간표를 찾을 수 없습니다.")
    return schedule


def create_schedule(
    offering_id: int, *, weekday: int, time_slot_id: int, space_id: int
) -> dict[str, Any]:
    get_offering(offering_id)  # 개설 강좌 존재 확인
    _ensure_schedule_refs(time_slot_id, space_id)
    return offering_repo.create_schedule(
        offering_id=offering_id, weekday=weekday, time_slot_id=time_slot_id, space_id=space_id
    )


def update_schedule(
    schedule_id: int, *, weekday: int, time_slot_id: int, space_id: int
) -> dict[str, Any]:
    _ensure_schedule_refs(time_slot_id, space_id)
    schedule = offering_repo.update_schedule(
        schedule_id, weekday=weekday, time_slot_id=time_slot_id, space_id=space_id
    )
    if schedule is None:
        raise ResourceNotFoundError("schedule_not_found", "시간표를 찾을 수 없습니다.")
    return schedule


def delete_schedule(schedule_id: int) -> None:
    if not offering_repo.delete_schedule(schedule_id):
        raise ResourceNotFoundError("schedule_not_found", "시간표를 찾을 수 없습니다.")
