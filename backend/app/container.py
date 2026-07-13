from dataclasses import dataclass

from app.core.bootstrap import RuntimeContext
from app.modules.system.public import GetHealthQuery, HealthStatus
from app.shared.application import QueryHandler


@dataclass(frozen=True, slots=True)
class SystemModule:
    get_health: QueryHandler[GetHealthQuery, HealthStatus]


@dataclass(frozen=True, slots=True)
class ApplicationContainer:
    runtime: RuntimeContext
    system: SystemModule
