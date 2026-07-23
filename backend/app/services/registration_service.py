"""회원의 복수 강좌 신청과 상태 변경 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ConflictError, ResourceNotFoundError
from app.repositories import member_repository as member_repo
from app.repositories import offering_repository as offering_repo
from app.repositories import registration_repository as registration_repo

_ACTIVE = {"applied", "selected", "waitlisted", "confirmed"}


def _validate_apply(member_id: int, offering_ids: list[int]) -> None:
    member = member_repo.get_member(member_id)
    if member is None:
        raise ResourceNotFoundError("member_not_found", "회원을 찾을 수 없습니다.")
    if not member["is_active"]:
        raise ConflictError("member_inactive", "이용 중지된 회원은 신청할 수 없습니다.")

    active_offering_ids = {
        r["offering_id"]
        for r in registration_repo.list_registrations(member_id=member_id)
        if r["status"] in _ACTIVE
    }
    existing_slots = set(registration_repo.get_member_active_slots(member_id))
    batch_slots: set[tuple[int, int]] = set()

    for offering_id in offering_ids:
        if offering_id in active_offering_ids:
            raise ConflictError("already_applied", "이미 신청한 강좌입니다.")
        offering = offering_repo.get_offering(offering_id)
        if offering is None:
            raise ResourceNotFoundError("offering_not_found", "개설 강좌를 찾을 수 없습니다.")
        if offering["status"] != "open":
            raise ConflictError("offering_not_open", "신청을 받지 않는 개설 강좌입니다.")
        # 시간 충돌 검사 (정책 seam: 기본 차단)
        for slot in registration_repo.get_offering_slots(offering_id):
            if slot in existing_slots or slot in batch_slots:
                raise ConflictError("time_conflict", "시간이 겹치는 강좌가 있습니다.")
            batch_slots.add(slot)


def apply(
    member_id: int,
    offering_ids: list[int],
    *,
    actor_operator_id: int | None = None,
    actor_display_name: str | None = None,
) -> list[dict[str, Any]]:
    offering_ids = list(dict.fromkeys(offering_ids))  # 중복 제거(순서 유지)
    _validate_apply(member_id, offering_ids)
    return registration_repo.apply_registrations(
        member_id,
        offering_ids,
        actor_operator_id=actor_operator_id,
        actor_display_name=actor_display_name,
    )


def list_registrations(
    *,
    member_id: int | None = None,
    offering_id: int | None = None,
    status: str | None = None,
) -> list[dict[str, Any]]:
    return registration_repo.list_registrations(
        member_id=member_id, offering_id=offering_id, status=status
    )


def get_registration(registration_id: int) -> dict[str, Any]:
    registration = registration_repo.get_registration(registration_id)
    if registration is None:
        raise ResourceNotFoundError("registration_not_found", "신청을 찾을 수 없습니다.")
    return registration


def list_history(registration_id: int) -> list[dict[str, Any]]:
    get_registration(registration_id)  # 존재 확인
    return registration_repo.list_history(registration_id)


def cancel(
    registration_id: int,
    *,
    reason: str | None,
    actor_operator_id: int | None = None,
    actor_display_name: str | None = None,
) -> dict[str, Any]:
    registration = registration_repo.cancel_registration(
        registration_id,
        reason,
        actor_operator_id=actor_operator_id,
        actor_display_name=actor_display_name,
    )
    if registration is None:
        raise ResourceNotFoundError("registration_not_found", "신청을 찾을 수 없습니다.")
    return registration
