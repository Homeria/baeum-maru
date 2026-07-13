from types import ModuleType

from app.modules.attendance import models as attendance_models
from app.modules.courses import models as course_models
from app.modules.identity import models as identity_models
from app.modules.locations import models as location_models
from app.modules.lottery import models as lottery_models
from app.modules.members import models as member_models
from app.modules.operations import models as operation_models
from app.modules.organization import models as organization_models
from app.modules.registrations import models as registration_models

MODEL_MODULES: tuple[ModuleType, ...] = (
    organization_models,
    identity_models,
    location_models,
    course_models,
    member_models,
    registration_models,
    lottery_models,
    attendance_models,
    operation_models,
)
