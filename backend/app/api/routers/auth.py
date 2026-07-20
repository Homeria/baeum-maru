"""접속 코드 로그인, 로그아웃과 현재 세션 요청을 받는 router."""

from typing import Annotated, Any

from fastapi import APIRouter, Depends, Request, Response, status

import app.services.auth_service as auth_service
from app.api.dependencies import SESSION_COOKIE_NAME, get_current_operator, get_settings
from app.core.settings import AppSettings
from app.schemas.auth import LoginRequest, OperatorIdentity

router = APIRouter(tags=["auth"])


@router.post("/auth/login", response_model=OperatorIdentity, summary="접속 코드로 로그인")
def login(
    payload: LoginRequest,
    response: Response,
    settings: Annotated[AppSettings, Depends(get_settings)],
) -> OperatorIdentity:
    result = auth_service.login(payload.code, session_ttl_minutes=settings.auth.session_ttl_minutes)
    response.set_cookie(
        SESSION_COOKIE_NAME,
        result["token"],
        max_age=settings.auth.session_ttl_minutes * 60,
        httponly=True,
        samesite="lax",
    )
    return OperatorIdentity.model_validate(result["operator"])


@router.post("/auth/logout", status_code=status.HTTP_204_NO_CONTENT, summary="로그아웃")
def logout(request: Request, response: Response) -> None:
    token = request.cookies.get(SESSION_COOKIE_NAME)
    if token:
        auth_service.logout(token)
    response.delete_cookie(SESSION_COOKIE_NAME)


@router.get("/auth/me", response_model=OperatorIdentity, summary="현재 로그인한 관계자")
def me(operator: Annotated[dict[str, Any], Depends(get_current_operator)]) -> OperatorIdentity:
    return OperatorIdentity.model_validate(operator)
