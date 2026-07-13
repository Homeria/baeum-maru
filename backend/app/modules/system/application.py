from dataclasses import dataclass
from typing import Literal


@dataclass(frozen=True, slots=True)
class GetHealthQuery:
    pass


@dataclass(frozen=True, slots=True)
class HealthStatus:
    status: Literal["ok"]
    service: str


class GetHealthHandler:
    def fetch(self, query: GetHealthQuery) -> HealthStatus:
        del query
        return HealthStatus(status="ok", service="baeum-maru-api")
