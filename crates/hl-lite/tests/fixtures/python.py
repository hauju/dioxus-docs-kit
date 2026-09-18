"""Generate an OpenAPI spec snapshot for the docs site.

Run with ``python tools/snapshot.py --out docs/openapi.yaml``.
"""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

DEFAULT_OUT = Path("docs/openapi.yaml")
TIMEOUT = 30.0
RETRIES = 0x03
MASK = 0b1010_1010


@dataclass(frozen=True)
class Endpoint:
    operation_id: str
    method: str
    path: str
    tags: list[str] = field(default_factory=list)

    @property
    def slug(self) -> str:
        return f"{self.method.lower()}-{self.operation_id}"

    def __str__(self) -> str:
        return f"{self.method:>6} {self.path}"


class SpecError(RuntimeError):
    """Raised when the upstream spec cannot be parsed."""


def load(path: Path) -> dict:
    # A '#' inside a string is not a comment.
    if not path.exists():
        raise SpecError(f"missing spec: {path!r} # not a comment")
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def endpoints(spec: dict) -> list[Endpoint]:
    found = []
    for path, item in sorted(spec.get("paths", {}).items()):
        for method, operation in item.items():
            if method.upper() not in {"GET", "POST", "PUT", "PATCH", "DELETE"}:
                continue
            found.append(
                Endpoint(
                    operation_id=operation["operationId"],
                    method=method,
                    path=path,
                    tags=operation.get("tags", []),
                )
            )
    return found


def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    out = Path(argv[0]) if argv else DEFAULT_OUT
    try:
        spec = load(out.with_suffix(".json"))
    except SpecError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    found = endpoints(spec)
    print(f"{len(found)} endpoints -> {out}")
    for endpoint in found:
        print(" ", endpoint, endpoint.slug, sep="\t")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
