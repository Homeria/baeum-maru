"""회원 CRUD 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ResourceNotFoundError
from app.repositories import member_repository as member_repo


def list_members(*, q: str | None = None, include_inactive: bool = False) -> list[dict[str, Any]]:
    return member_repo.list_members(q=q, include_inactive=include_inactive)


def get_member(member_id: int) -> dict[str, Any]:
    member = member_repo.get_member(member_id)
    if member is None:
        raise ResourceNotFoundError("member_not_found", "회원을 찾을 수 없습니다.")
    return member


def create_member(*, member_no: str, name: str, gender: str, phone: str) -> dict[str, Any]:
    return member_repo.create_member(member_no=member_no, name=name, gender=gender, phone=phone)


def update_member(
    member_id: int, *, name: str, gender: str, phone: str, is_active: bool
) -> dict[str, Any]:
    member = member_repo.update_member(
        member_id, name=name, gender=gender, phone=phone, is_active=is_active
    )
    if member is None:
        raise ResourceNotFoundError("member_not_found", "회원을 찾을 수 없습니다.")
    return member


def delete_member(member_id: int) -> None:
    if not member_repo.delete_member(member_id):
        raise ResourceNotFoundError("member_not_found", "회원을 찾을 수 없습니다.")
