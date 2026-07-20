"""건물과 층 CRUD 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ResourceNotFoundError
from app.repositories import building_repository as building_repo

# --- buildings ---


def list_buildings(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return building_repo.list_buildings(include_inactive=include_inactive)


def get_building(building_id: int) -> dict[str, Any]:
    building = building_repo.get_building(building_id)
    if building is None:
        raise ResourceNotFoundError("building_not_found", "건물을 찾을 수 없습니다.")
    return building


def create_building(*, name: str, description: str | None, sort_order: int) -> dict[str, Any]:
    return building_repo.create_building(name=name, description=description, sort_order=sort_order)


def update_building(
    building_id: int,
    *,
    name: str,
    description: str | None,
    sort_order: int,
    is_active: bool,
) -> dict[str, Any]:
    building = building_repo.update_building(
        building_id,
        name=name,
        description=description,
        sort_order=sort_order,
        is_active=is_active,
    )
    if building is None:
        raise ResourceNotFoundError("building_not_found", "건물을 찾을 수 없습니다.")
    return building


def delete_building(building_id: int) -> None:
    if not building_repo.delete_building(building_id):
        raise ResourceNotFoundError("building_not_found", "건물을 찾을 수 없습니다.")


# --- building_floors ---


def list_floors(building_id: int, *, include_inactive: bool = False) -> list[dict[str, Any]]:
    get_building(building_id)  # 존재 확인
    return building_repo.list_floors(building_id, include_inactive=include_inactive)


def get_floor(floor_id: int) -> dict[str, Any]:
    floor = building_repo.get_floor(floor_id)
    if floor is None:
        raise ResourceNotFoundError("floor_not_found", "층을 찾을 수 없습니다.")
    return floor


def create_floor(building_id: int, *, label: str, sort_order: int) -> dict[str, Any]:
    get_building(building_id)  # 건물 존재 확인 후 층 등록
    return building_repo.create_floor(building_id=building_id, label=label, sort_order=sort_order)


def update_floor(floor_id: int, *, label: str, sort_order: int, is_active: bool) -> dict[str, Any]:
    floor = building_repo.update_floor(
        floor_id, label=label, sort_order=sort_order, is_active=is_active
    )
    if floor is None:
        raise ResourceNotFoundError("floor_not_found", "층을 찾을 수 없습니다.")
    return floor


def delete_floor(floor_id: int) -> None:
    if not building_repo.delete_floor(floor_id):
        raise ResourceNotFoundError("floor_not_found", "층을 찾을 수 없습니다.")
