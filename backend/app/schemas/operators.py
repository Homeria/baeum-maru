"""관계자와 접속 코드 API의 요청·응답 schema."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

Role = Literal["staff", "temporary_staff", "viewer"]


class OperatorCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    display_name: str = Field(min_length=1, max_length=80)
    role: Role


class OperatorUpdate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    display_name: str = Field(min_length=1, max_length=80)
    role: Role
    is_active: bool = True


class OperatorResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    display_name: str
    role: Role
    is_active: bool
    created_at: str
    updated_at: str


class AccessCodeIssueRequest(BaseModel):
    # 미지정 시 서버 기본 TTL(설정 파일)을 사용한다.
    ttl_minutes: int | None = Field(default=None, ge=1)


class AccessCodeResponse(BaseModel):
    """접속 코드 메타데이터(평문 코드는 포함하지 않는다)."""

    model_config = ConfigDict(from_attributes=True)

    id: int
    operator_id: int
    issued_at: str
    expires_at: str
    revoked_at: str | None = None
    last_used_at: str | None = None


class AccessCodeIssueResponse(AccessCodeResponse):
    """발급 직후 단 한 번 평문 코드를 함께 반환한다."""

    code: str
