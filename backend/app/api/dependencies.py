from typing import Annotated

from fastapi import Depends, Request

from app.container import ApplicationContainer


def get_application_container(request: Request) -> ApplicationContainer:
    container = getattr(request.app.state, "container", None)
    if not isinstance(container, ApplicationContainer):
        raise RuntimeError("Application container is not initialized")
    return container


ContainerDependency = Annotated[
    ApplicationContainer,
    Depends(get_application_container),
]
