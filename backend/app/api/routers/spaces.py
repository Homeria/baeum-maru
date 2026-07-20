"""장소 유형과 장소 CRUD 요청을 받는 router."""

from fastapi import APIRouter, Query, status

import app.services.space_service as space_service
from app.schemas.spaces import (
    SpaceCreate,
    SpaceResponse,
    SpaceTypeCreate,
    SpaceTypeResponse,
    SpaceTypeUpdate,
    SpaceUpdate,
)

router = APIRouter(tags=["spaces"])


# --- space_types ---


@router.post(
    "/space-types",
    response_model=SpaceTypeResponse,
    status_code=status.HTTP_201_CREATED,
    summary="장소 유형 등록",
)
def create_space_type(payload: SpaceTypeCreate) -> SpaceTypeResponse:
    space_type = space_service.create_space_type(
        name=payload.name,
        is_course_eligible=payload.is_course_eligible,
        sort_order=payload.sort_order,
    )
    return SpaceTypeResponse.model_validate(space_type)


@router.get("/space-types", response_model=list[SpaceTypeResponse], summary="장소 유형 목록")
def list_space_types(include_inactive: bool = Query(default=False)) -> list[SpaceTypeResponse]:
    space_types = space_service.list_space_types(include_inactive=include_inactive)
    return [SpaceTypeResponse.model_validate(item) for item in space_types]


@router.get(
    "/space-types/{space_type_id}", response_model=SpaceTypeResponse, summary="장소 유형 조회"
)
def get_space_type(space_type_id: int) -> SpaceTypeResponse:
    return SpaceTypeResponse.model_validate(space_service.get_space_type(space_type_id))


@router.patch(
    "/space-types/{space_type_id}", response_model=SpaceTypeResponse, summary="장소 유형 수정"
)
def update_space_type(space_type_id: int, payload: SpaceTypeUpdate) -> SpaceTypeResponse:
    space_type = space_service.update_space_type(
        space_type_id,
        name=payload.name,
        is_course_eligible=payload.is_course_eligible,
        sort_order=payload.sort_order,
        is_active=payload.is_active,
    )
    return SpaceTypeResponse.model_validate(space_type)


@router.delete(
    "/space-types/{space_type_id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="장소 유형 삭제",
)
def delete_space_type(space_type_id: int) -> None:
    space_service.delete_space_type(space_type_id)


# --- spaces ---


@router.post(
    "/spaces",
    response_model=SpaceResponse,
    status_code=status.HTTP_201_CREATED,
    summary="장소 등록",
)
def create_space(payload: SpaceCreate) -> SpaceResponse:
    space = space_service.create_space(
        building_floor_id=payload.building_floor_id,
        space_type_id=payload.space_type_id,
        name=payload.name,
        sort_order=payload.sort_order,
    )
    return SpaceResponse.model_validate(space)


@router.get("/spaces", response_model=list[SpaceResponse], summary="장소 목록")
def list_spaces(
    building_floor_id: int | None = Query(default=None),
    include_inactive: bool = Query(default=False),
) -> list[SpaceResponse]:
    spaces = space_service.list_spaces(
        building_floor_id=building_floor_id, include_inactive=include_inactive
    )
    return [SpaceResponse.model_validate(item) for item in spaces]


@router.get("/spaces/{space_id}", response_model=SpaceResponse, summary="장소 조회")
def get_space(space_id: int) -> SpaceResponse:
    return SpaceResponse.model_validate(space_service.get_space(space_id))


@router.patch("/spaces/{space_id}", response_model=SpaceResponse, summary="장소 수정")
def update_space(space_id: int, payload: SpaceUpdate) -> SpaceResponse:
    space = space_service.update_space(
        space_id,
        building_floor_id=payload.building_floor_id,
        space_type_id=payload.space_type_id,
        name=payload.name,
        sort_order=payload.sort_order,
        is_active=payload.is_active,
    )
    return SpaceResponse.model_validate(space)


@router.delete("/spaces/{space_id}", status_code=status.HTTP_204_NO_CONTENT, summary="장소 삭제")
def delete_space(space_id: int) -> None:
    space_service.delete_space(space_id)
