"""강좌 기준 정보(분류·난도·강사·학기·교시) API의 요청·응답 schema."""

from typing import Literal

from pydantic import BaseModel, ConfigDict, Field, model_validator

TermStatus = Literal["draft", "open", "closed", "finalized"]


# --- course_categories (분류) ---


class CourseCategoryCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=100)
    sort_order: int = Field(default=0, ge=0)


class CourseCategoryUpdate(CourseCategoryCreate):
    is_active: bool = True


class CourseCategoryResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str


# --- course_levels (난도) ---


class CourseLevelCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=80)
    sort_order: int = Field(default=0, ge=0)


class CourseLevelUpdate(CourseLevelCreate):
    is_active: bool = True


class CourseLevelResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str


# --- instructors (강사) ---


class InstructorCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=80)
    phone: str | None = Field(default=None, max_length=20)


class InstructorUpdate(InstructorCreate):
    is_active: bool = True


class InstructorResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    phone: str | None
    is_active: bool
    created_at: str
    updated_at: str


# --- terms (학기) ---


class TermCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=120)
    starts_on: str | None = None
    ends_on: str | None = None
    registration_opens_at: str | None = None
    registration_closes_at: str | None = None
    max_registrations_per_member: int = Field(default=0, ge=0)
    status: TermStatus = "draft"

    @model_validator(mode="after")
    def _check_order(self) -> "TermCreate":
        if self.starts_on and self.ends_on and self.starts_on > self.ends_on:
            raise ValueError("starts_on must be on or before ends_on")
        if (
            self.registration_opens_at
            and self.registration_closes_at
            and self.registration_opens_at >= self.registration_closes_at
        ):
            raise ValueError("registration_opens_at must be before registration_closes_at")
        return self


class TermUpdate(TermCreate):
    pass


class TermResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    starts_on: str | None
    ends_on: str | None
    registration_opens_at: str | None
    registration_closes_at: str | None
    max_registrations_per_member: int
    status: TermStatus
    created_at: str
    updated_at: str


# --- time_slots (교시) ---


class TimeSlotCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    name: str = Field(min_length=1, max_length=80)
    start_time: str = Field(min_length=1, max_length=8)
    end_time: str = Field(min_length=1, max_length=8)
    sort_order: int = Field(default=0, ge=0)

    @model_validator(mode="after")
    def _check_order(self) -> "TimeSlotCreate":
        if self.start_time >= self.end_time:
            raise ValueError("start_time must be before end_time")
        return self


class TimeSlotUpdate(TimeSlotCreate):
    is_active: bool = True


class TimeSlotResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    name: str
    start_time: str
    end_time: str
    sort_order: int
    is_active: bool
    created_at: str
    updated_at: str


# --- courses (과목) ---


class CourseCreate(BaseModel):
    model_config = ConfigDict(str_strip_whitespace=True)

    category_id: int
    level_id: int | None = None
    name: str = Field(min_length=1, max_length=160)
    description: str | None = Field(default=None, max_length=2000)


class CourseUpdate(CourseCreate):
    is_active: bool = True


class CourseResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    category_id: int
    level_id: int | None
    name: str
    description: str | None
    is_active: bool
    created_at: str
    updated_at: str
