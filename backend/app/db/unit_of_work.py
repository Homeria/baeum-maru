"""여러 repository 작업을 하나의 commit 또는 rollback으로 묶는 transaction 모듈.

Repository는 commit하지 않으며 service가 이 경계를 통해 transaction 결과를 결정한다.
"""
