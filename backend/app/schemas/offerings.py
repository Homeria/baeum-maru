"""개설 강좌와 시간표 API의 요청·응답 schema."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

CapacityType = Literal["fixed", "open", "gender_split"]
OfferingStatus = Literal["draft", "open", "closed", "cancelled"]


class OfferingCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    term_id: int
    course_id: int
    section_label: str | None = Field(default=None, max_length=80)
    instructor_id: int | None = None
    capacity_type: CapacityType = "fixed"
    capacity_total: int | None = Field(default=None, ge=1)
    male_capacity: int | None = Field(default=None, ge=0)
    female_capacity: int | None = Field(default=None, ge=0)
    status: OfferingStatus = "draft"
    sort_order: int = Field(default=0, ge=0)

    @field_validator("section_label")
    @classmethod
    def _empty_to_none(cls, value: str | None) -> str | None:
        return value or None

    @model_validator(mode="after")
    def _check_capacity(self) -> "OfferingCreate":
        if self.capacity_type == "fixed":
            if self.capacity_total is None:
                raise ValueError("fixed capacity requires capacity_total")
            if self.male_capacity is not None or self.female_capacity is not None:
                raise ValueError("fixed capacity must not set gender capacities")
        elif self.capacity_type == "open":
            if (
                self.capacity_total is not None
                or self.male_capacity is not None
                or self.female_capacity is not None
            ):
                raise ValueError("open capacity must not set any capacity value")
        else:  # gender_split
            if self.capacity_total is not None:
                raise ValueError("gender_split must not set capacity_total")
            if self.male_capacity is None or self.female_capacity is None:
                raise ValueError("gender_split requires male_capacity and female_capacity")
            if self.male_capacity + self.female_capacity <= 0:
                raise ValueError("gender_split total capacity must be greater than 0")
        return self


class OfferingUpdate(OfferingCreate):
    pass


class OfferingResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    term_id: int
    course_id: int
    section_label: str | None
    instructor_id: int | None
    capacity_type: CapacityType
    capacity_total: int | None
    male_capacity: int | None
    female_capacity: int | None
    status: OfferingStatus
    sort_order: int
    created_at: str
    updated_at: str


class ScheduleCreate(BaseModel):
    weekday: int = Field(ge=1, le=7, description="1=월 ... 7=일")
    time_slot_id: int
    space_id: int


class ScheduleUpdate(ScheduleCreate):
    pass


class ScheduleResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    offering_id: int
    weekday: int
    time_slot_id: int
    space_id: int
