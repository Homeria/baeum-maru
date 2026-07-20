"""건물과 층 CRUD 요청을 받는 router."""

from fastapi import APIRouter, Depends, Query, status

import app.services.building_service as building_service
from app.api.dependencies import get_current_operator
from app.schemas.buildings import (
    BuildingCreate,
    BuildingFloorCreate,
    BuildingFloorResponse,
    BuildingFloorUpdate,
    BuildingResponse,
    BuildingUpdate,
)

router = APIRouter(tags=["buildings"], dependencies=[Depends(get_current_operator)])


# --- buildings ---


@router.post(
    "/buildings",
    response_model=BuildingResponse,
    status_code=status.HTTP_201_CREATED,
    summary="건물 등록",
)
def create_building(payload: BuildingCreate) -> BuildingResponse:
    building = building_service.create_building(
        name=payload.name, description=payload.description, sort_order=payload.sort_order
    )
    return BuildingResponse.model_validate(building)


@router.get("/buildings", response_model=list[BuildingResponse], summary="건물 목록")
def list_buildings(include_inactive: bool = Query(default=False)) -> list[BuildingResponse]:
    buildings = building_service.list_buildings(include_inactive=include_inactive)
    return [BuildingResponse.model_validate(item) for item in buildings]


@router.get("/buildings/{building_id}", response_model=BuildingResponse, summary="건물 조회")
def get_building(building_id: int) -> BuildingResponse:
    return BuildingResponse.model_validate(building_service.get_building(building_id))


@router.patch("/buildings/{building_id}", response_model=BuildingResponse, summary="건물 수정")
def update_building(building_id: int, payload: BuildingUpdate) -> BuildingResponse:
    building = building_service.update_building(
        building_id,
        name=payload.name,
        description=payload.description,
        sort_order=payload.sort_order,
        is_active=payload.is_active,
    )
    return BuildingResponse.model_validate(building)


@router.delete(
    "/buildings/{building_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="건물 삭제",
)
def delete_building(building_id: int) -> None:
    building_service.delete_building(building_id)


# --- building_floors ---


@router.post(
    "/buildings/{building_id}/floors",
    response_model=BuildingFloorResponse,
    status_code=status.HTTP_201_CREATED,
    summary="층 등록",
)
def create_floor(building_id: int, payload: BuildingFloorCreate) -> BuildingFloorResponse:
    floor = building_service.create_floor(
        building_id, label=payload.label, sort_order=payload.sort_order
    )
    return BuildingFloorResponse.model_validate(floor)


@router.get(
    "/buildings/{building_id}/floors",
    response_model=list[BuildingFloorResponse],
    summary="건물의 층 목록",
)
def list_floors(
    building_id: int, include_inactive: bool = Query(default=False)
) -> list[BuildingFloorResponse]:
    floors = building_service.list_floors(building_id, include_inactive=include_inactive)
    return [BuildingFloorResponse.model_validate(item) for item in floors]


@router.get(
    "/building-floors/{floor_id}",
    response_model=BuildingFloorResponse,
    summary="층 조회",
)
def get_floor(floor_id: int) -> BuildingFloorResponse:
    return BuildingFloorResponse.model_validate(building_service.get_floor(floor_id))


@router.patch(
    "/building-floors/{floor_id}",
    response_model=BuildingFloorResponse,
    summary="층 수정",
)
def update_floor(floor_id: int, payload: BuildingFloorUpdate) -> BuildingFloorResponse:
    floor = building_service.update_floor(
        floor_id, label=payload.label, sort_order=payload.sort_order, is_active=payload.is_active
    )
    return BuildingFloorResponse.model_validate(floor)


@router.delete(
    "/building-floors/{floor_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="층 삭제",
)
def delete_floor(floor_id: int) -> None:
    building_service.delete_floor(floor_id)
