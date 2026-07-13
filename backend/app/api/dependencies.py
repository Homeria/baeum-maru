"""FastAPI 공통 의존성 제공 모듈.

DB session, 현재 로그인 사용자와 service 생성처럼 여러 router가 공유하는
Depends 의존성을 정의한다. 업무 규칙은 이 모듈에서 처리하지 않는다.
"""
