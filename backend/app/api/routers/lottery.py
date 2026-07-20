"""강좌 추첨 미리보기, 확정과 결과 조회 요청을 받는 router."""

from fastapi import APIRouter, Query, status

import app.services.lottery_service as lottery_service
from app.schemas.lottery import (
    CommitRequest,
    PreviewRequest,
    PreviewResponse,
    ResultResponse,
    RunResponse,
)

router = APIRouter(tags=["lottery"])


@router.post(
    "/lottery/preview", response_model=PreviewResponse, summary="추첨 미리보기(저장 안 함)"
)
def preview(payload: PreviewRequest) -> PreviewResponse:
    return PreviewResponse.model_validate(lottery_service.preview(payload.term_id))


@router.post(
    "/lottery/commit",
    response_model=RunResponse,
    status_code=status.HTTP_201_CREATED,
    summary="추첨 확정(같은 seed로 저장)",
)
def commit(payload: CommitRequest) -> RunResponse:
    run = lottery_service.commit(
        payload.term_id,
        seed=payload.seed,
        executed_by_operator_id=payload.executed_by_operator_id,
    )
    return RunResponse.model_validate(run)


@router.get("/lottery/runs", response_model=list[RunResponse], summary="추첨 실행 목록")
def list_runs(term_id: int | None = Query(default=None)) -> list[RunResponse]:
    return [RunResponse.model_validate(r) for r in lottery_service.list_runs(term_id)]


@router.get("/lottery/runs/{run_id}", response_model=RunResponse, summary="추첨 실행 조회")
def get_run(run_id: int) -> RunResponse:
    return RunResponse.model_validate(lottery_service.get_run(run_id))


@router.get(
    "/lottery/runs/{run_id}/results",
    response_model=list[ResultResponse],
    summary="추첨 결과 조회",
)
def get_run_results(run_id: int) -> list[ResultResponse]:
    return [ResultResponse.model_validate(r) for r in lottery_service.get_run_results(run_id)]
