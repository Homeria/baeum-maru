"""관계자(operator) CRUD 업무 규칙을 담는 service."""

from typing import Any

from app.core.exceptions import ResourceNotFoundError
from app.repositories import operator_repository as operator_repo


def list_operators(*, include_inactive: bool = False) -> list[dict[str, Any]]:
    return operator_repo.list_operators(include_inactive=include_inactive)


def get_operator(operator_id: int) -> dict[str, Any]:
    operator = operator_repo.get_operator(operator_id)
    if operator is None:
        raise ResourceNotFoundError("operator_not_found", "관계자를 찾을 수 없습니다.")
    return operator


def create_operator(*, display_name: str, role: str) -> dict[str, Any]:
    return operator_repo.create_operator(display_name=display_name, role=role)


def update_operator(
    operator_id: int, *, display_name: str, role: str, is_active: bool
) -> dict[str, Any]:
    operator = operator_repo.update_operator(
        operator_id, display_name=display_name, role=role, is_active=is_active
    )
    if operator is None:
        raise ResourceNotFoundError("operator_not_found", "관계자를 찾을 수 없습니다.")
    return operator


def delete_operator(operator_id: int) -> None:
    if not operator_repo.delete_operator(operator_id):
        raise ResourceNotFoundError("operator_not_found", "관계자를 찾을 수 없습니다.")
