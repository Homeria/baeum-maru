"""회원 API의 요청·응답 schema."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

Gender = Literal["male", "female"]


class MemberCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    member_no: str = Field(min_length=1, max_length=40)
    name: str = Field(min_length=1, max_length=80)
    gender: Gender
    phone: str = Field(min_length=1, max_length=20)


class MemberUpdate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=80)
    gender: Gender
    phone: str = Field(min_length=1, max_length=20)
    is_active: bool = True


class MemberResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    member_no: str
    name: str
    gender: Gender
    phone: str
    is_active: bool
    created_at: str
    updated_at: str
