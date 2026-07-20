"""회원의 강좌 신청, 조회와 상태 변경 요청을 받는 router."""

from fastapi import APIRouter, Query, status

import app.services.registration_service as registration_service
from app.schemas.registrations import (
    CancelRequest,
    RegistrationApply,
    RegistrationResponse,
    StatusHistoryResponse,
)

router = APIRouter(tags=["registrations"])


@router.post(
    "/registrations",
    response_model=list[RegistrationResponse],
    status_code=status.HTTP_201_CREATED,
    summary="수강 신청(다중, 원자적)",
)
def apply(payload: RegistrationApply) -> list[RegistrationResponse]:
    items = registration_service.apply(payload.member_id, payload.offering_ids)
    return [RegistrationResponse.model_validate(i) for i in items]


@router.get("/registrations", response_model=list[RegistrationResponse], summary="신청 목록")
def list_registrations(
    member_id: int | None = Query(default=None),
    offering_id: int | None = Query(default=None),
    term_id: int | None = Query(default=None),
    status: str | None = Query(default=None),
) -> list[RegistrationResponse]:
    items = registration_service.list_registrations(
        member_id=member_id, offering_id=offering_id, term_id=term_id, status=status
    )
    return [RegistrationResponse.model_validate(i) for i in items]


@router.get(
    "/registrations/{registration_id}", response_model=RegistrationResponse, summary="신청 조회"
)
def get_registration(registration_id: int) -> RegistrationResponse:
    return RegistrationResponse.model_validate(
        registration_service.get_registration(registration_id)
    )


@router.get(
    "/registrations/{registration_id}/history",
    response_model=list[StatusHistoryResponse],
    summary="신청 상태 이력",
)
def get_history(registration_id: int) -> list[StatusHistoryResponse]:
    items = registration_service.list_history(registration_id)
    return [StatusHistoryResponse.model_validate(i) for i in items]


@router.post(
    "/registrations/{registration_id}/cancel",
    response_model=RegistrationResponse,
    summary="신청 취소(당첨 시 대기 승계)",
)
def cancel(registration_id: int, payload: CancelRequest) -> RegistrationResponse:
    return RegistrationResponse.model_validate(
        registration_service.cancel(registration_id, reason=payload.reason)
    )
