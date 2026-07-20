"""회원 CRUD 요청을 받는 router."""

from fastapi import APIRouter, Query, status

import app.services.member_service as member_service
from app.schemas.members import MemberCreate, MemberResponse, MemberUpdate

router = APIRouter(tags=["members"])


@router.post(
    "/members",
    response_model=MemberResponse,
    status_code=status.HTTP_201_CREATED,
    summary="회원 등록",
)
def create_member(payload: MemberCreate) -> MemberResponse:
    member = member_service.create_member(
        member_no=payload.member_no,
        name=payload.name,
        gender=payload.gender,
        phone=payload.phone,
    )
    return MemberResponse.model_validate(member)


@router.get("/members", response_model=list[MemberResponse], summary="회원 목록/검색")
def list_members(
    q: str | None = Query(default=None, description="이름·전화·회원번호 검색어"),
    include_inactive: bool = Query(default=False),
) -> list[MemberResponse]:
    members = member_service.list_members(q=q, include_inactive=include_inactive)
    return [MemberResponse.model_validate(item) for item in members]


@router.get("/members/{member_id}", response_model=MemberResponse, summary="회원 조회")
def get_member(member_id: int) -> MemberResponse:
    return MemberResponse.model_validate(member_service.get_member(member_id))


@router.patch("/members/{member_id}", response_model=MemberResponse, summary="회원 수정")
def update_member(member_id: int, payload: MemberUpdate) -> MemberResponse:
    member = member_service.update_member(
        member_id,
        name=payload.name,
        gender=payload.gender,
        phone=payload.phone,
        is_active=payload.is_active,
    )
    return MemberResponse.model_validate(member)


@router.delete("/members/{member_id}", status_code=status.HTTP_204_NO_CONTENT, summary="회원 삭제")
def delete_member(member_id: int) -> None:
    member_service.delete_member(member_id)
