"""장소 유형과 장소 API의 요청·응답 schema."""

from pydantic import BaseModel, ConfigDict, Field


class SpaceTypeCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=80)
    is_course_eligible: bool = True
    sort_order: int = Field(default=0, ge=0)


class SpaceTypeUpdate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=80)
    is_course_eligible: bool = True
    sort_order: int = Field(default=0, ge=0)
    is_active: bool = True


class SpaceTypeResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    is_course_eligible: bool
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str


class SpaceCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    building_floor_id: int
    space_type_id: int
    name: str = Field(min_length=1, max_length=120)
    sort_order: int = Field(default=0, ge=0)


class SpaceUpdate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    building_floor_id: int
    space_type_id: int
    name: str = Field(min_length=1, max_length=120)
    sort_order: int = Field(default=0, ge=0)
    is_active: bool = True


class SpaceResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    building_floor_id: int
    space_type_id: int
    name: str
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str
