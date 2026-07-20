"""추첨 미리보기·확정과 결과 조회 API의 요청·응답 schema."""

from pydantic import BaseModel, ConfigDict


class PreviewRequest(BaseModel):
    term_id: int


class CommitRequest(BaseModel):
    term_id: int
    seed: int
    executed_by_operator_id: int | None = None


class ResultItem(BaseModel):
    registration_id: int
    result: str
    result_order: int


class TargetPlan(BaseModel):
    offering_id: int
    capacity_type: str
    capacity_total: int | None
    male_capacity: int | None
    female_capacity: int | None
    eligible_count: int
    eligible_male: int | None
    eligible_female: int | None
    results: list[ResultItem]


class PreviewResponse(BaseModel):
    seed: int
    offerings: list[TargetPlan]


class RunResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    term_id: int
    seed: int
    executed_by_operator_id: int | None
    created_at: str


class ResultResponse(BaseModel):
    model_config = ConfigDict(from_attributes=True)

    id: int
    lottery_run_target_id: int
    registration_id: int
    result: str
    result_order: int
    created_at: str
