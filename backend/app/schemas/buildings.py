"""건물과 층 API의 요청·응답 schema."""

from pydantic import BaseModel, ConfigDict, Field


class BuildingCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=120)
    description: str | None = Field(default=None, max_length=1000)
    sort_order: int = Field(default=0, ge=0)


class BuildingUpdate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=120)
    description: str | None = Field(default=None, max_length=1000)
    sort_order: int = Field(default=0, ge=0)
    is_active: bool = True


class BuildingResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    description: str | None
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str


class BuildingFloorCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    label: str = Field(min_length=1, max_length=80)
    sort_order: int = Field(default=0, ge=0)


class BuildingFloorUpdate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    label: str = Field(min_length=1, max_length=80)
    sort_order: int = Field(default=0, ge=0)
    is_active: bool = True


class BuildingFloorResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    building_id: int
    label: str
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str
