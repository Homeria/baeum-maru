"""개설 강좌와 시간표 CRUD 요청을 받는 router."""

from fastapi import APIRouter, Query, status

import app.services.offering_service as offering_service
from app.schemas.offerings import (
    OfferingCreate,
    OfferingResponse,
    OfferingUpdate,
    ScheduleCreate,
    ScheduleResponse,
    ScheduleUpdate,
)

router = APIRouter(tags=["offerings"])


# --- course_offerings (개설 강좌) ---


@router.post(
    "/offerings",
    response_model=OfferingResponse,
    status_code=status.HTTP_201_CREATED,
    summary="개설 강좌 등록",
)
def create_offering(payload: OfferingCreate) -> OfferingResponse:
    item = offering_service.create_offering(
        term_id=payload.term_id,
        course_id=payload.course_id,
        section_label=payload.section_label,
        instructor_id=payload.instructor_id,
        capacity_type=payload.capacity_type,
        capacity_total=payload.capacity_total,
        male_capacity=payload.male_capacity,
        female_capacity=payload.female_capacity,
        status=payload.status,
        sort_order=payload.sort_order,
    )
    return OfferingResponse.model_validate(item)


@router.get("/offerings", response_model=list[OfferingResponse], summary="개설 강좌 목록")
def list_offerings(term_id: int | None = Query(default=None)) -> list[OfferingResponse]:
    items = offering_service.list_offerings(term_id=term_id)
    return [OfferingResponse.model_validate(i) for i in items]


@router.get("/offerings/{offering_id}", response_model=OfferingResponse, summary="개설 강좌 조회")
def get_offering(offering_id: int) -> OfferingResponse:
    return OfferingResponse.model_validate(offering_service.get_offering(offering_id))


@router.patch("/offerings/{offering_id}", response_model=OfferingResponse, summary="개설 강좌 수정")
def update_offering(offering_id: int, payload: OfferingUpdate) -> OfferingResponse:
    item = offering_service.update_offering(
        offering_id,
        term_id=payload.term_id,
        course_id=payload.course_id,
        section_label=payload.section_label,
        instructor_id=payload.instructor_id,
        capacity_type=payload.capacity_type,
        capacity_total=payload.capacity_total,
        male_capacity=payload.male_capacity,
        female_capacity=payload.female_capacity,
        status=payload.status,
        sort_order=payload.sort_order,
    )
    return OfferingResponse.model_validate(item)


@router.delete(
    "/offerings/{offering_id}", status_code=status.HTTP_204_NO_CONTENT, summary="개설 강좌 삭제"
)
def delete_offering(offering_id: int) -> None:
    offering_service.delete_offering(offering_id)


# --- course_schedules (시간표) ---


@router.post(
    "/offerings/{offering_id}/schedules",
    response_model=ScheduleResponse,
    status_code=status.HTTP_201_CREATED,
    summary="시간표 등록",
)
def create_schedule(offering_id: int, payload: ScheduleCreate) -> ScheduleResponse:
    item = offering_service.create_schedule(
        offering_id,
        weekday=payload.weekday,
        time_slot_id=payload.time_slot_id,
        space_id=payload.space_id,
    )
    return ScheduleResponse.model_validate(item)


@router.get(
    "/offerings/{offering_id}/schedules",
    response_model=list[ScheduleResponse],
    summary="개설 강좌의 시간표 목록",
)
def list_schedules(offering_id: int) -> list[ScheduleResponse]:
    items = offering_service.list_schedules(offering_id)
    return [ScheduleResponse.model_validate(i) for i in items]


@router.get(
    "/course-schedules/{schedule_id}", response_model=ScheduleResponse, summary="시간표 조회"
)
def get_schedule(schedule_id: int) -> ScheduleResponse:
    return ScheduleResponse.model_validate(offering_service.get_schedule(schedule_id))


@router.patch(
    "/course-schedules/{schedule_id}", response_model=ScheduleResponse, summary="시간표 수정"
)
def update_schedule(schedule_id: int, payload: ScheduleUpdate) -> ScheduleResponse:
    item = offering_service.update_schedule(
        schedule_id,
        weekday=payload.weekday,
        time_slot_id=payload.time_slot_id,
        space_id=payload.space_id,
    )
    return ScheduleResponse.model_validate(item)


@router.delete(
    "/course-schedules/{schedule_id}", status_code=status.HTTP_204_NO_CONTENT, summary="시간표 삭제"
)
def delete_schedule(schedule_id: int) -> None:
    offering_service.delete_schedule(schedule_id)
