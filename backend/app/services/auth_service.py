"""접속 코드 발급과 로그인 세션 수명주기를 담는 service."""

from typing import Any

from app.core.exceptions import AuthenticationError, ConflictError, ResourceNotFoundError
from app.core.security import generate_access_code, generate_session_token, hash_secret
from app.repositories import auth_repository as auth_repo
from app.repositories import operator_repository as operator_repo

_MAX_CODE_ATTEMPTS = 5


def _require_operator(operator_id: int) -> dict[str, Any]:
    operator = operator_repo.get_operator(operator_id)
    if operator is None:
        raise ResourceNotFoundError("operator_not_found", "관계자를 찾을 수 없습니다.")
    return operator


# --- 접속 코드 관리 ---


def issue_access_code(operator_id: int, *, ttl_minutes: int) -> dict[str, Any]:
    """관계자에게 접속 코드를 발급하고 평문 코드를 한 번만 반환한다."""
    operator = _require_operator(operator_id)
    if not operator["is_active"]:
        raise ConflictError("operator_inactive", "비활성 관계자에게는 코드를 발급할 수 없습니다.")
    for _ in range(_MAX_CODE_ATTEMPTS):
        code = generate_access_code()
        try:
            record = auth_repo.create_access_code(
                operator_id=operator_id, code_hash=hash_secret(code), ttl_minutes=ttl_minutes
            )
        except ConflictError:
            continue  # 코드 hash 충돌은 새 코드로 재시도
        return {"code": code, **record}
    raise ConflictError("access_code_collision", "접속 코드 발급에 실패했습니다. 다시 시도하세요.")


def list_access_codes(operator_id: int) -> list[dict[str, Any]]:
    _require_operator(operator_id)
    return auth_repo.list_access_codes(operator_id)


def revoke_access_code(operator_id: int, code_id: int) -> None:
    _require_operator(operator_id)
    if not auth_repo.revoke_access_code(code_id):
        raise ResourceNotFoundError("access_code_not_found", "폐기할 접속 코드를 찾을 수 없습니다.")


# --- 로그인 세션 ---


def login(code: str, *, session_ttl_minutes: int) -> dict[str, Any]:
    """접속 코드로 세션을 열고 평문 세션 token과 관계자 정보를 반환한다."""
    active = auth_repo.get_active_access_code_by_hash(hash_secret(code))
    if active is None:
        raise AuthenticationError(
            "invalid_access_code", "접속 코드가 올바르지 않거나 만료되었습니다."
        )
    if not active["is_active"]:
        raise AuthenticationError(
            "operator_inactive", "비활성 관계자입니다. 관리자에게 문의하세요."
        )
    token = generate_session_token()
    auth_repo.open_session(
        operator_id=active["operator_id"],
        access_code_id=active["access_code_id"],
        token_hash=hash_secret(token),
        ttl_minutes=session_ttl_minutes,
    )
    return {
        "token": token,
        "operator": {
            "id": active["operator_id"],
            "display_name": active["display_name"],
            "role": active["role"],
        },
    }


def verify_session(token: str) -> dict[str, Any] | None:
    """세션 token을 검증하고 유효하면 마지막 활동 시각을 갱신해 관계자를 반환한다."""
    session = auth_repo.get_active_session_by_hash(hash_secret(token))
    if session is None:
        return None
    auth_repo.touch_session(session["session_id"])
    return {
        "session_id": session["session_id"],
        "id": session["operator_id"],
        "display_name": session["display_name"],
        "role": session["role"],
    }


def logout(token: str) -> None:
    auth_repo.revoke_session_by_hash(hash_secret(token))


def bootstrap_admin(*, ttl_minutes: int) -> str | None:
    """관계자가 하나도 없으면 최초 관리자와 접속 코드를 만들고 평문 코드를 반환한다."""
    if operator_repo.list_operators(include_inactive=True):
        return None
    operator = operator_repo.create_operator(display_name="최초 관리자", role="staff")
    return str(issue_access_code(operator["id"], ttl_minutes=ttl_minutes)["code"])
