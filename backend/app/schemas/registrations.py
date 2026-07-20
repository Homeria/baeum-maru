"""강좌 신청과 상태 이력 API의 요청·응답 schema."""

from pydantic import BaseModel, ConfigDict, Field


class RegistrationApply(BaseModel):
    member_id: int
    offering_ids: list[int] = Field(min_length=1, description="신청할 개설 강좌 ID 목록")


class CancelRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    reason: str | None = Field(default=None, max_length=255)


class RegistrationResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    member_id: int
    offering_id: int
    status: str
    waitlist_order: int | None
    created_at: str
    updated_at: str


class StatusHistoryResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    registration_id: int
    from_status: str | None
    to_status: str
    reason: str | None
    actor_kind: str
    actor_display_name: str | None
    changed_at: str
