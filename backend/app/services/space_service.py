"""장소 유형과 장소 CRUD 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ResourceNotFoundError
from app.repositories import building_repository as building_repo
from app.repositories import space_repository as space_repo

# --- space_types ---


def list_space_types(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return space_repo.list_space_types(include_inactive=include_inactive)


def get_space_type(space_type_id: int) -> dict[str, Any]:
    space_type = space_repo.get_space_type(space_type_id)
    if space_type is None:
        raise ResourceNotFoundError("space_type_not_found", "장소 유형을 찾을 수 없습니다.")
    return space_type


def create_space_type(*, name: str, is_course_eligible: bool, sort_order: int) -> dict[str, Any]:
    return space_repo.create_space_type(
        name=name, is_course_eligible=is_course_eligible, sort_order=sort_order
    )


def update_space_type(
    space_type_id: int,
    *,
    name: str,
    is_course_eligible: bool,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any]:
    space_type = space_repo.update_space_type(
        space_type_id,
        name=name,
        is_course_eligible=is_course_eligible,
        sort_order=sort_order,
        is_active=is_active,
    )
    if space_type is None:
        raise ResourceNotFoundError("space_type_not_found", "장소 유형을 찾을 수 없습니다.")
    return space_type


def delete_space_type(space_type_id: int) -> None:
    if not space_repo.delete_space_type(space_type_id):
        raise ResourceNotFoundError("space_type_not_found", "장소 유형을 찾을 수 없습니다.")


# --- spaces ---


def _ensure_floor_and_type(building_floor_id: int, space_type_id: int) -> None:
    if building_repo.get_floor(building_floor_id) is None:
        raise ResourceNotFoundError("floor_not_found", "층을 찾을 수 없습니다.")
    if space_repo.get_space_type(space_type_id) is None:
        raise ResourceNotFoundError("space_type_not_found", "장소 유형을 찾을 수 없습니다.")


def list_spaces(
    *, building_floor_id: int | None = None, include_inactive: bool = False
) -> list[dict[str, Any]]:
    return space_repo.list_spaces(
        building_floor_id=building_floor_id, include_inactive=include_inactive
    )


def get_space(space_id: int) -> dict[str, Any]:
    space = space_repo.get_space(space_id)
    if space is None:
        raise ResourceNotFoundError("space_not_found", "장소를 찾을 수 없습니다.")
    return space


def create_space(
    *, building_floor_id: int, space_type_id: int, name: str, sort_order: int
) -> dict[str, Any]:
    _ensure_floor_and_type(building_floor_id, space_type_id)
    return space_repo.create_space(
        building_floor_id=building_floor_id,
        space_type_id=space_type_id,
        name=name,
        sort_order=sort_order,
    )


def update_space(
    space_id: int,
    *,
    building_floor_id: int,
    space_type_id: int,
    name: str,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any]:
    _ensure_floor_and_type(building_floor_id, space_type_id)
    space = space_repo.update_space(
        space_id,
        building_floor_id=building_floor_id,
        space_type_id=space_type_id,
        name=name,
        sort_order=sort_order,
        is_active=is_active,
    )
    if space is None:
        raise ResourceNotFoundError("space_not_found", "장소를 찾을 수 없습니다.")
    return space


def delete_space(space_id: int) -> None:
    if not space_repo.delete_space(space_id):
        raise ResourceNotFoundError("space_not_found", "장소를 찾을 수 없습니다.")
