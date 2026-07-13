import ast
from collections.abc import Iterable
from pathlib import Path
from typing import TypedDict

APP_ROOT = Path(__file__).resolve().parents[2] / "app"
MODULES_ROOT = APP_ROOT / "modules"

FRAMEWORK_IMPORTS = (
    "fastapi",
    "pydantic",
    "pydantic_core",
    "pydantic_settings",
    "sqlalchemy",
    "starlette",
)


class LayerRules(TypedDict):
    blocked_prefixes: tuple[str, ...]
    blocked_layers: set[str]


INNER_LAYER_RULES: dict[str, LayerRules] = {
    "application": {
        "blocked_prefixes": (*FRAMEWORK_IMPORTS, "app.api", "app.container", "app.core"),
        "blocked_layers": {"api", "schemas", "repository", "models", "infrastructure"},
    },
    "domain": {
        "blocked_prefixes": (*FRAMEWORK_IMPORTS, "app.api", "app.container", "app.core"),
        "blocked_layers": {
            "api",
            "schemas",
            "application",
            "repository",
            "models",
            "infrastructure",
        },
    },
    "ports": {
        "blocked_prefixes": (*FRAMEWORK_IMPORTS, "app.api", "app.container", "app.core"),
        "blocked_layers": {
            "api",
            "schemas",
            "application",
            "repository",
            "models",
            "infrastructure",
        },
    },
    "public": {
        "blocked_prefixes": (*FRAMEWORK_IMPORTS, "app.api", "app.container", "app.core"),
        "blocked_layers": {"api", "schemas", "repository", "models", "infrastructure"},
    },
    "api": {
        "blocked_prefixes": ("sqlalchemy",),
        "blocked_layers": {"repository", "models", "infrastructure"},
    },
    "schemas": {
        "blocked_prefixes": ("sqlalchemy",),
        "blocked_layers": {"repository", "models", "infrastructure"},
    },
    "repository": {
        "blocked_prefixes": ("fastapi", "app.api"),
        "blocked_layers": {"api", "schemas"},
    },
    "models": {
        "blocked_prefixes": ("fastapi", "app.api"),
        "blocked_layers": {"api", "schemas"},
    },
    "infrastructure": {
        "blocked_prefixes": ("fastapi", "app.api"),
        "blocked_layers": {"api", "schemas"},
    },
}


def test_feature_layer_imports_follow_dependency_direction() -> None:
    violations: list[str] = []

    for path in MODULES_ROOT.rglob("*.py"):
        rules = INNER_LAYER_RULES.get(_layer_for(path))
        if rules is None:
            continue

        for imported in _imported_modules(path):
            blocked_prefixes = rules["blocked_prefixes"]
            blocked_layers = rules["blocked_layers"]
            if _starts_with_any(imported, blocked_prefixes) or _targets_layer(
                imported, blocked_layers
            ):
                violations.append(f"{path.relative_to(APP_ROOT)} -> {imported}")

    assert not violations, "Invalid layer imports:\n" + "\n".join(violations)


def test_feature_modules_use_absolute_imports() -> None:
    violations: list[str] = []

    for path in MODULES_ROOT.rglob("*.py"):
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        if any(isinstance(node, ast.ImportFrom) and node.level for node in ast.walk(tree)):
            violations.append(str(path.relative_to(APP_ROOT)))

    assert not violations, "Feature modules must use absolute imports:\n" + "\n".join(violations)


def test_cross_module_imports_use_public_boundary() -> None:
    violations: list[str] = []

    for path in MODULES_ROOT.rglob("*.py"):
        relative = path.relative_to(MODULES_ROOT)
        if len(relative.parts) < 2:
            continue
        source_module = relative.parts[0]
        source_layer = _layer_for(path)

        for imported in _imported_modules(path):
            parts = imported.split(".")
            if len(parts) < 3 or parts[:2] != ["app", "modules"]:
                continue
            target_module = parts[2]
            if target_module == source_module:
                continue
            if source_layer != "application" or len(parts) < 4 or parts[3] != "public":
                violations.append(f"{relative} -> {imported}")

    assert not violations, "Cross-module imports must target public.py:\n" + "\n".join(violations)


def test_every_feature_module_has_public_boundary() -> None:
    feature_modules = [
        path for path in MODULES_ROOT.iterdir() if path.is_dir() and not path.name.startswith("__")
    ]
    missing = [path.name for path in feature_modules if not (path / "public.py").is_file()]

    assert not missing, f"Feature modules without public.py: {missing}"


def test_main_imports_application_only_through_composition_root() -> None:
    internal_imports = {
        imported
        for imported in _imported_modules(APP_ROOT / "main.py")
        if imported.startswith("app.")
    }

    assert internal_imports == {"app.composition"}


def _imported_modules(path: Path) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    modules: set[str] = set()

    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            modules.update(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            modules.add(node.module)

    return modules


def _starts_with_any(module: str, prefixes: Iterable[str]) -> bool:
    return any(module == prefix or module.startswith(f"{prefix}.") for prefix in prefixes)


def _targets_layer(module: str, blocked_layers: Iterable[str]) -> bool:
    parts = module.split(".")
    return (
        len(parts) >= 4
        and parts[:2] == ["app", "modules"]
        and any(part in blocked_layers for part in parts[3:])
    )


def _layer_for(path: Path) -> str:
    relative = path.relative_to(MODULES_ROOT)
    module_path = relative.parts[1:]
    if len(module_path) > 1 and module_path[0] in INNER_LAYER_RULES:
        return module_path[0]
    return path.stem
