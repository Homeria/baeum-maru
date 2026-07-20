"""재현 가능한 강좌 추첨(미리보기·확정)과 결과 조회 규칙을 담는 service."""

import random
import secrets
from typing import Any

from app.core.exceptions import ConflictError, ResourceNotFoundError
from app.repositories import course_repository as course_repo
from app.repositories import lottery_repository as lottery_repo


def _draw_offering(offering: dict[str, Any], seed: int) -> dict[str, Any]:
    """한 개설 강좌의 신청자를 seed로 뽑아 당첨/대기 결과를 만든다(순수 함수)."""
    rng = random.Random(f"{seed}:{offering['offering_id']}")
    eligible = list(offering["eligible"])
    capacity_type = offering["capacity_type"]
    selected_ids: list[int] = []
    waitlisted_ids: list[int] = []
    eligible_male: int | None = None
    eligible_female: int | None = None

    if capacity_type == "open":
        rng.shuffle(eligible)
        selected_ids = [e["registration_id"] for e in eligible]
    elif capacity_type == "fixed":
        rng.shuffle(eligible)
        cap = offering["capacity_total"] or 0
        selected_ids = [e["registration_id"] for e in eligible[:cap]]
        waitlisted_ids = [e["registration_id"] for e in eligible[cap:]]
    else:  # gender_split
        males = [e for e in eligible if e["gender"] == "male"]
        females = [e for e in eligible if e["gender"] == "female"]
        eligible_male, eligible_female = len(males), len(females)
        rng.shuffle(males)
        rng.shuffle(females)
        mcap = offering["male_capacity"] or 0
        fcap = offering["female_capacity"] or 0
        selected_ids = [e["registration_id"] for e in males[:mcap] + females[:fcap]]
        leftovers = males[mcap:] + females[fcap:]
        rng.shuffle(leftovers)
        waitlisted_ids = [e["registration_id"] for e in leftovers]

    results = [
        {"registration_id": rid, "result": "selected", "result_order": i}
        for i, rid in enumerate(selected_ids, start=1)
    ]
    results += [
        {"registration_id": rid, "result": "waitlisted", "result_order": i}
        for i, rid in enumerate(waitlisted_ids, start=1)
    ]
    return {
        "offering_id": offering["offering_id"],
        "capacity_type": capacity_type,
        "capacity_total": offering["capacity_total"],
        "male_capacity": offering["male_capacity"],
        "female_capacity": offering["female_capacity"],
        "eligible_count": len(eligible),
        "eligible_male": eligible_male,
        "eligible_female": eligible_female,
        "results": results,
    }


def _compute_plan(term_id: int, seed: int) -> list[dict[str, Any]]:
    rows = lottery_repo.get_draw_candidates(term_id)
    offerings: dict[int, dict[str, Any]] = {}
    for row in rows:
        oid = int(row["offering_id"])
        if oid not in offerings:
            offerings[oid] = {
                "offering_id": oid,
                "capacity_type": row["capacity_type"],
                "capacity_total": row["capacity_total"],
                "male_capacity": row["male_capacity"],
                "female_capacity": row["female_capacity"],
                "eligible": [],
            }
        offerings[oid]["eligible"].append(
            {"registration_id": int(row["registration_id"]), "gender": row["gender"]}
        )
    return [_draw_offering(offerings[oid], seed) for oid in sorted(offerings)]


def _require_term(term_id: int) -> None:
    if course_repo.get_term(term_id) is None:
        raise ResourceNotFoundError("term_not_found", "학기를 찾을 수 없습니다.")


def preview(term_id: int) -> dict[str, Any]:
    """seed를 생성하고 결과를 계산해 반환한다(저장하지 않음)."""
    _require_term(term_id)
    seed = secrets.randbits(63)
    return {"seed": seed, "offerings": _compute_plan(term_id, seed)}


def commit(
    term_id: int,
    *,
    seed: int,
    executed_by_operator_id: int | None = None,
    actor_display_name: str | None = None,
) -> dict[str, Any]:
    """같은 seed로 재계산해 결과를 원자적으로 저장하고 registrations에 반영한다."""
    _require_term(term_id)
    targets = _compute_plan(term_id, seed)
    if not targets:
        raise ConflictError("no_applicants", "추첨할 신청자가 없습니다.")
    run_id = lottery_repo.commit_lottery(
        term_id=term_id,
        seed=seed,
        executed_by_operator_id=executed_by_operator_id,
        actor_display_name=actor_display_name,
        targets=targets,
    )
    run = lottery_repo.get_run(run_id)
    assert run is not None
    return run


def list_runs(term_id: int | None = None) -> list[dict[str, Any]]:
    return lottery_repo.list_runs(term_id)


def get_run(run_id: int) -> dict[str, Any]:
    run = lottery_repo.get_run(run_id)
    if run is None:
        raise ResourceNotFoundError("lottery_run_not_found", "추첨 실행을 찾을 수 없습니다.")
    return run


def get_run_results(run_id: int) -> list[dict[str, Any]]:
    get_run(run_id)  # 존재 확인
    return lottery_repo.get_run_results(run_id)
