"""접속 코드·세션 token 생성과 단방향 hash를 담당하는 보안 도구 모듈."""

import hashlib
import secrets

# 손으로 옮겨 적기 쉽도록 0/O/1/I/L 처럼 헷갈리는 문자를 뺀 알파벳.
_CODE_ALPHABET = "ABCDEFGHJKMNPQRSTUVWXYZ23456789"


def generate_access_code() -> str:
    """관계자에게 공유할 사람이 읽기 쉬운 접속 코드(XXXX-XXXX)를 만든다."""
    chars = [secrets.choice(_CODE_ALPHABET) for _ in range(8)]
    return f"{''.join(chars[:4])}-{''.join(chars[4:])}"


def generate_session_token() -> str:
    """추측 불가능한 세션 token을 만든다(쿠키에 담고 hash만 저장)."""
    return secrets.token_urlsafe(32)


def hash_secret(secret: str) -> str:
    """접속 코드·세션 token을 DB 조회·비교용으로 단방향 hash한다."""
    return hashlib.sha256(secret.encode("utf-8")).hexdigest()
