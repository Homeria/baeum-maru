"""접속 코드 로그인과 현재 세션 API의 요청·응답 schema."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

Role = Literal["staff", "temporary_staff", "viewer"]


class LoginRequest(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    code: str = Field(min_length=1, max_length=32)


class OperatorIdentity(BaseModel):
    """로그인·현재 세션 응답이 공유하는 관계자 정보."""

    model_config = ConfigDict(from_attributes=True)

    id: int
    display_name: str
    role: Role
