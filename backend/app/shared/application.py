from typing import Protocol, TypeVar

CommandT = TypeVar("CommandT", contravariant=True)
QueryT = TypeVar("QueryT", contravariant=True)
ResultT = TypeVar("ResultT", covariant=True)


class CommandHandler(Protocol[CommandT, ResultT]):
    """Application boundary for use cases that change persistent state."""

    def execute(self, command: CommandT) -> ResultT: ...


class QueryHandler(Protocol[QueryT, ResultT]):
    """Application boundary for side-effect-free reads."""

    def fetch(self, query: QueryT) -> ResultT: ...
