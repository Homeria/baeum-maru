"""강좌 기준 정보(분류·난도·강사·학기·교시) CRUD 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ResourceNotFoundError
from app.repositories import course_repository as course_repo

# --- course_categories (분류) ---


def list_categories(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return course_repo.list_categories(include_inactive=include_inactive)


def get_category(category_id: int) -> dict[str, Any]:
    category = course_repo.get_category(category_id)
    if category is None:
        raise ResourceNotFoundError("category_not_found", "분류를 찾을 수 없습니다.")
    return category


def create_category(*, name: str, sort_order: int) -> dict[str, Any]:
    return course_repo.create_category(name=name, sort_order=sort_order)


def update_category(
    category_id: int, *, name: str, sort_order: int, is_active: bool
) -> dict[str, Any]:
    category = course_repo.update_category(
        category_id, name=name, sort_order=sort_order, is_active=is_active
    )
    if category is None:
        raise ResourceNotFoundError("category_not_found", "분류를 찾을 수 없습니다.")
    return category


def delete_category(category_id: int) -> None:
    if not course_repo.delete_category(category_id):
        raise ResourceNotFoundError("category_not_found", "분류를 찾을 수 없습니다.")


# --- course_levels (난도) ---


def list_levels(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return course_repo.list_levels(include_inactive=include_inactive)


def get_level(level_id: int) -> dict[str, Any]:
    level = course_repo.get_level(level_id)
    if level is None:
        raise ResourceNotFoundError("level_not_found", "난도를 찾을 수 없습니다.")
    return level


def create_level(*, name: str, sort_order: int) -> dict[str, Any]:
    return course_repo.create_level(name=name, sort_order=sort_order)


def update_level(level_id: int, *, name: str, sort_order: int, is_active: bool) -> dict[str, Any]:
    level = course_repo.update_level(
        level_id, name=name, sort_order=sort_order, is_active=is_active
    )
    if level is None:
        raise ResourceNotFoundError("level_not_found", "난도를 찾을 수 없습니다.")
    return level


def delete_level(level_id: int) -> None:
    if not course_repo.delete_level(level_id):
        raise ResourceNotFoundError("level_not_found", "난도를 찾을 수 없습니다.")


# --- instructors (강사) ---


def list_instructors(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return course_repo.list_instructors(include_inactive=include_inactive)


def get_instructor(instructor_id: int) -> dict[str, Any]:
    instructor = course_repo.get_instructor(instructor_id)
    if instructor is None:
        raise ResourceNotFoundError("instructor_not_found", "강사를 찾을 수 없습니다.")
    return instructor


def create_instructor(*, name: str, phone: str | None) -> dict[str, Any]:
    return course_repo.create_instructor(name=name, phone=phone)


def update_instructor(
    instructor_id: int, *, name: str, phone: str | None, is_active: bool
) -> dict[str, Any]:
    instructor = course_repo.update_instructor(
        instructor_id, name=name, phone=phone, is_active=is_active
    )
    if instructor is None:
        raise ResourceNotFoundError("instructor_not_found", "강사를 찾을 수 없습니다.")
    return instructor


def delete_instructor(instructor_id: int) -> None:
    if not course_repo.delete_instructor(instructor_id):
        raise ResourceNotFoundError("instructor_not_found", "강사를 찾을 수 없습니다.")


# --- time_slots (교시) ---


def list_time_slots(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return course_repo.list_time_slots(include_inactive=include_inactive)


def get_time_slot(time_slot_id: int) -> dict[str, Any]:
    time_slot = course_repo.get_time_slot(time_slot_id)
    if time_slot is None:
        raise ResourceNotFoundError("time_slot_not_found", "교시를 찾을 수 없습니다.")
    return time_slot


def create_time_slot(
    *, name: str, start_time: str, end_time: str, sort_order: int
) -> dict[str, Any]:
    return course_repo.create_time_slot(
        name=name, start_time=start_time, end_time=end_time, sort_order=sort_order
    )


def update_time_slot(
    time_slot_id: int,
    *,
    name: str,
    start_time: str,
    end_time: str,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any]:
    time_slot = course_repo.update_time_slot(
        time_slot_id,
        name=name,
        start_time=start_time,
        end_time=end_time,
        sort_order=sort_order,
        is_active=is_active,
    )
    if time_slot is None:
        raise ResourceNotFoundError("time_slot_not_found", "교시를 찾을 수 없습니다.")
    return time_slot


def delete_time_slot(time_slot_id: int) -> None:
    if not course_repo.delete_time_slot(time_slot_id):
        raise ResourceNotFoundError("time_slot_not_found", "교시를 찾을 수 없습니다.")


# --- courses (과목) ---


def _ensure_category_and_level(category_id: int, level_id: int | None) -> None:
    if course_repo.get_category(category_id) is None:
        raise ResourceNotFoundError("category_not_found", "분류를 찾을 수 없습니다.")
    if level_id is not None and course_repo.get_level(level_id) is None:
        raise ResourceNotFoundError("level_not_found", "난도를 찾을 수 없습니다.")


def list_courses(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return course_repo.list_courses(include_inactive=include_inactive)


def get_course(course_id: int) -> dict[str, Any]:
    course = course_repo.get_course(course_id)
    if course is None:
        raise ResourceNotFoundError("course_not_found", "과목을 찾을 수 없습니다.")
    return course


def create_course(
    *, category_id: int, level_id: int | None, name: str, description: str | None
) -> dict[str, Any]:
    _ensure_category_and_level(category_id, level_id)
    return course_repo.create_course(
        category_id=category_id, level_id=level_id, name=name, description=description
    )


def update_course(
    course_id: int,
    *,
    category_id: int,
    level_id: int | None,
    name: str,
    description: str | None,
    is_active: bool,
) -> dict[str, Any]:
    _ensure_category_and_level(category_id, level_id)
    course = course_repo.update_course(
        course_id,
        category_id=category_id,
        level_id=level_id,
        name=name,
        description=description,
        is_active=is_active,
    )
    if course is None:
        raise ResourceNotFoundError("course_not_found", "과목을 찾을 수 없습니다.")
    return course


def delete_course(course_id: int) -> None:
    if not course_repo.delete_course(course_id):
        raise ResourceNotFoundError("course_not_found", "과목을 찾을 수 없습니다.")
