"""하위 계층이 상위 계층을 import하지 못하게 한다."""

import ast
from pathlib import Path

APP_ROOT = Path(__file__).resolve().parents[2] / "app"

# 각 디렉터리에서 import할 수 없는 상위 계층이다.
FORBIDDEN_IMPORTS = {
    "core": ("app.api", "app.services", "app.repositories", "app.db"),
    "db": ("app.api", "app.services", "app.repositories"),
    "schemas": ("app.api", "app.services", "app.repositories", "app.db"),
    "repositories": ("app.api", "app.services"),
    "services": ("app.api",),
}


def _imported_modules(path: Path) -> set[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    imports: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.update(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            if node.level == 0 and node.module:
                imports.add(node.module)
            elif node.level > 0:
                package = ["app", *path.relative_to(APP_ROOT).parent.parts]
                parent = package[: len(package) - node.level + 1]
                module = [*parent, *(node.module or "").split(".")]
                imports.add(".".join(part for part in module if part))
    return imports


def _matches_prefix(module: str, prefix: str) -> bool:
    return module == prefix or module.startswith(f"{prefix}.")


def test_lower_layers_do_not_import_upper_layers() -> None:
    violations: list[str] = []
    for layer, forbidden_prefixes in FORBIDDEN_IMPORTS.items():
        for path in sorted((APP_ROOT / layer).rglob("*.py")):
            for module in sorted(_imported_modules(path)):
                if any(_matches_prefix(module, prefix) for prefix in forbidden_prefixes):
                    relative_path = path.relative_to(APP_ROOT.parent)
                    violations.append(f"{relative_path}: {module}")

    assert not violations, "상위 계층 import가 발견되었습니다:\n" + "\n".join(violations)


def test_unit_of_work_abstraction_is_not_present() -> None:
    assert not (APP_ROOT / "db" / "unit_of_work.py").exists()
