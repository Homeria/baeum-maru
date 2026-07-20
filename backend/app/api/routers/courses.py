"""강좌 기준 정보(분류·난도·강사·학기·교시) CRUD 요청을 받는 router."""

from fastapi import APIRouter, Depends, Query, status

import app.services.course_service as course_service
from app.api.dependencies import get_current_operator
from app.schemas.courses import (
    CourseCategoryCreate,
    CourseCategoryResponse,
    CourseCategoryUpdate,
    CourseCreate,
    CourseLevelCreate,
    CourseLevelResponse,
    CourseLevelUpdate,
    CourseResponse,
    CourseUpdate,
    InstructorCreate,
    InstructorResponse,
    InstructorUpdate,
    TermCreate,
    TermResponse,
    TermUpdate,
    TimeSlotCreate,
    TimeSlotResponse,
    TimeSlotUpdate,
)

router = APIRouter(tags=["course-masters"], dependencies=[Depends(get_current_operator)])


# --- course_categories (분류) ---


@router.post(
    "/course-categories",
    response_model=CourseCategoryResponse,
    status_code=status.HTTP_201_CREATED,
    summary="분류 등록",
)
def create_category(payload: CourseCategoryCreate) -> CourseCategoryResponse:
    item = course_service.create_category(name=payload.name, sort_order=payload.sort_order)
    return CourseCategoryResponse.model_validate(item)


@router.get("/course-categories", response_model=list[CourseCategoryResponse], summary="분류 목록")
def list_categories(include_inactive: bool = Query(default=False)) -> list[CourseCategoryResponse]:
    items = course_service.list_categories(include_inactive=include_inactive)
    return [CourseCategoryResponse.model_validate(i) for i in items]


@router.get(
    "/course-categories/{category_id}", response_model=CourseCategoryResponse, summary="분류 조회"
)
def get_category(category_id: int) -> CourseCategoryResponse:
    return CourseCategoryResponse.model_validate(course_service.get_category(category_id))


@router.patch(
    "/course-categories/{category_id}", response_model=CourseCategoryResponse, summary="분류 수정"
)
def update_category(category_id: int, payload: CourseCategoryUpdate) -> CourseCategoryResponse:
    item = course_service.update_category(
        category_id, name=payload.name, sort_order=payload.sort_order, is_active=payload.is_active
    )
    return CourseCategoryResponse.model_validate(item)


@router.delete(
    "/course-categories/{category_id}", status_code=status.HTTP_204_NO_CONTENT, summary="분류 삭제"
)
def delete_category(category_id: int) -> None:
    course_service.delete_category(category_id)


# --- course_levels (난도) ---


@router.post(
    "/course-levels",
    response_model=CourseLevelResponse,
    status_code=status.HTTP_201_CREATED,
    summary="난도 등록",
)
def create_level(payload: CourseLevelCreate) -> CourseLevelResponse:
    item = course_service.create_level(name=payload.name, sort_order=payload.sort_order)
    return CourseLevelResponse.model_validate(item)


@router.get("/course-levels", response_model=list[CourseLevelResponse], summary="난도 목록")
def list_levels(include_inactive: bool = Query(default=False)) -> list[CourseLevelResponse]:
    items = course_service.list_levels(include_inactive=include_inactive)
    return [CourseLevelResponse.model_validate(i) for i in items]


@router.get("/course-levels/{level_id}", response_model=CourseLevelResponse, summary="난도 조회")
def get_level(level_id: int) -> CourseLevelResponse:
    return CourseLevelResponse.model_validate(course_service.get_level(level_id))


@router.patch("/course-levels/{level_id}", response_model=CourseLevelResponse, summary="난도 수정")
def update_level(level_id: int, payload: CourseLevelUpdate) -> CourseLevelResponse:
    item = course_service.update_level(
        level_id, name=payload.name, sort_order=payload.sort_order, is_active=payload.is_active
    )
    return CourseLevelResponse.model_validate(item)


@router.delete(
    "/course-levels/{level_id}", status_code=status.HTTP_204_NO_CONTENT, summary="난도 삭제"
)
def delete_level(level_id: int) -> None:
    course_service.delete_level(level_id)


# --- instructors (강사) ---


@router.post(
    "/instructors",
    response_model=InstructorResponse,
    status_code=status.HTTP_201_CREATED,
    summary="강사 등록",
)
def create_instructor(payload: InstructorCreate) -> InstructorResponse:
    item = course_service.create_instructor(name=payload.name, phone=payload.phone)
    return InstructorResponse.model_validate(item)


@router.get("/instructors", response_model=list[InstructorResponse], summary="강사 목록")
def list_instructors(include_inactive: bool = Query(default=False)) -> list[InstructorResponse]:
    items = course_service.list_instructors(include_inactive=include_inactive)
    return [InstructorResponse.model_validate(i) for i in items]


@router.get("/instructors/{instructor_id}", response_model=InstructorResponse, summary="강사 조회")
def get_instructor(instructor_id: int) -> InstructorResponse:
    return InstructorResponse.model_validate(course_service.get_instructor(instructor_id))


@router.patch(
    "/instructors/{instructor_id}", response_model=InstructorResponse, summary="강사 수정"
)
def update_instructor(instructor_id: int, payload: InstructorUpdate) -> InstructorResponse:
    item = course_service.update_instructor(
        instructor_id, name=payload.name, phone=payload.phone, is_active=payload.is_active
    )
    return InstructorResponse.model_validate(item)


@router.delete(
    "/instructors/{instructor_id}", status_code=status.HTTP_204_NO_CONTENT, summary="강사 삭제"
)
def delete_instructor(instructor_id: int) -> None:
    course_service.delete_instructor(instructor_id)


# --- terms (학기) ---


@router.post(
    "/terms", response_model=TermResponse, status_code=status.HTTP_201_CREATED, summary="학기 등록"
)
def create_term(payload: TermCreate) -> TermResponse:
    item = course_service.create_term(
        name=payload.name,
        starts_on=payload.starts_on,
        ends_on=payload.ends_on,
        registration_opens_at=payload.registration_opens_at,
        registration_closes_at=payload.registration_closes_at,
        max_registrations_per_member=payload.max_registrations_per_member,
        status=payload.status,
    )
    return TermResponse.model_validate(item)


@router.get("/terms", response_model=list[TermResponse], summary="학기 목록")
def list_terms() -> list[TermResponse]:
    return [TermResponse.model_validate(i) for i in course_service.list_terms()]


@router.get("/terms/{term_id}", response_model=TermResponse, summary="학기 조회")
def get_term(term_id: int) -> TermResponse:
    return TermResponse.model_validate(course_service.get_term(term_id))


@router.patch("/terms/{term_id}", response_model=TermResponse, summary="학기 수정")
def update_term(term_id: int, payload: TermUpdate) -> TermResponse:
    item = course_service.update_term(
        term_id,
        name=payload.name,
        starts_on=payload.starts_on,
        ends_on=payload.ends_on,
        registration_opens_at=payload.registration_opens_at,
        registration_closes_at=payload.registration_closes_at,
        max_registrations_per_member=payload.max_registrations_per_member,
        status=payload.status,
    )
    return TermResponse.model_validate(item)


@router.delete("/terms/{term_id}", status_code=status.HTTP_204_NO_CONTENT, summary="학기 삭제")
def delete_term(term_id: int) -> None:
    course_service.delete_term(term_id)


# --- time_slots (교시) ---


@router.post(
    "/time-slots",
    response_model=TimeSlotResponse,
    status_code=status.HTTP_201_CREATED,
    summary="교시 등록",
)
def create_time_slot(payload: TimeSlotCreate) -> TimeSlotResponse:
    item = course_service.create_time_slot(
        name=payload.name,
        start_time=payload.start_time,
        end_time=payload.end_time,
        sort_order=payload.sort_order,
    )
    return TimeSlotResponse.model_validate(item)


@router.get("/time-slots", response_model=list[TimeSlotResponse], summary="교시 목록")
def list_time_slots(include_inactive: bool = Query(default=False)) -> list[TimeSlotResponse]:
    items = course_service.list_time_slots(include_inactive=include_inactive)
    return [TimeSlotResponse.model_validate(i) for i in items]


@router.get("/time-slots/{time_slot_id}", response_model=TimeSlotResponse, summary="교시 조회")
def get_time_slot(time_slot_id: int) -> TimeSlotResponse:
    return TimeSlotResponse.model_validate(course_service.get_time_slot(time_slot_id))


@router.patch("/time-slots/{time_slot_id}", response_model=TimeSlotResponse, summary="교시 수정")
def update_time_slot(time_slot_id: int, payload: TimeSlotUpdate) -> TimeSlotResponse:
    item = course_service.update_time_slot(
        time_slot_id,
        name=payload.name,
        start_time=payload.start_time,
        end_time=payload.end_time,
        sort_order=payload.sort_order,
        is_active=payload.is_active,
    )
    return TimeSlotResponse.model_validate(item)


@router.delete(
    "/time-slots/{time_slot_id}", status_code=status.HTTP_204_NO_CONTENT, summary="교시 삭제"
)
def delete_time_slot(time_slot_id: int) -> None:
    course_service.delete_time_slot(time_slot_id)


# --- courses (과목) ---


@router.post(
    "/courses",
    response_model=CourseResponse,
    status_code=status.HTTP_201_CREATED,
    summary="과목 등록",
)
def create_course(payload: CourseCreate) -> CourseResponse:
    item = course_service.create_course(
        category_id=payload.category_id,
        level_id=payload.level_id,
        name=payload.name,
        description=payload.description,
    )
    return CourseResponse.model_validate(item)


@router.get("/courses", response_model=list[CourseResponse], summary="과목 목록")
def list_courses(include_inactive: bool = Query(default=False)) -> list[CourseResponse]:
    items = course_service.list_courses(include_inactive=include_inactive)
    return [CourseResponse.model_validate(i) for i in items]


@router.get("/courses/{course_id}", response_model=CourseResponse, summary="과목 조회")
def get_course(course_id: int) -> CourseResponse:
    return CourseResponse.model_validate(course_service.get_course(course_id))


@router.patch("/courses/{course_id}", response_model=CourseResponse, summary="과목 수정")
def update_course(course_id: int, payload: CourseUpdate) -> CourseResponse:
    item = course_service.update_course(
        course_id,
        category_id=payload.category_id,
        level_id=payload.level_id,
        name=payload.name,
        description=payload.description,
        is_active=payload.is_active,
    )
    return CourseResponse.model_validate(item)


@router.delete("/courses/{course_id}", status_code=status.HTTP_204_NO_CONTENT, summary="과목 삭제")
def delete_course(course_id: int) -> None:
    course_service.delete_course(course_id)
