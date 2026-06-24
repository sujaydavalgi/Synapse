#!/usr/bin/env python3
"""Generate docs/api-reference.md — hierarchical index of modules and public APIs."""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "docs" / "api-reference.md"

SKIP_DIRS = {
    "target",
    "node_modules",
    "dist",
    ".git",
    "tests",
    "golden",
    "__pycache__",
}

RUST_GROUPS: list[tuple[str, list[tuple[str, str]]]] = [
    (
        "Public facade",
        [
            ("crates/spanda-core", "spanda-core"),
        ],
    ),
    (
        "Apps and bindings",
        [
            ("crates/spanda-cli", "spanda-cli"),
            ("crates/spanda-node", "spanda-node"),
            ("crates/spanda-wasm", "spanda-wasm"),
            ("crates/spanda-dap", "spanda-dap"),
        ],
    ),
    (
        "Compile and run pipeline",
        [
            ("crates/spanda-driver", "spanda-driver"),
            ("crates/spanda-lexer", "spanda-lexer"),
            ("crates/spanda-parser", "spanda-parser"),
            ("crates/spanda-typecheck", "spanda-typecheck"),
            ("crates/spanda-sir", "spanda-sir"),
            ("crates/spanda-error", "spanda-error"),
        ],
    ),
    (
        "Front-end AST",
        [
            ("crates/spanda-ast", "spanda-ast"),
            ("crates/spanda-regex-lang", "spanda-regex-lang"),
        ],
    ),
    (
        "Interpreter and runtime",
        [
            ("crates/spanda-interpreter", "spanda-interpreter"),
            ("crates/spanda-runtime", "spanda-runtime"),
            ("crates/spanda-runtime-host", "spanda-runtime-host"),
            ("crates/spanda-comm", "spanda-comm"),
            ("crates/spanda-safety", "spanda-safety"),
            ("crates/spanda-hal", "spanda-hal"),
            ("crates/spanda-concurrency", "spanda-concurrency"),
            ("crates/spanda-debug", "spanda-debug"),
            ("crates/spanda-ai", "spanda-ai"),
        ],
    ),
    (
        "Hardware, certify, fleet, OTA",
        [
            ("crates/spanda-hardware", "spanda-hardware"),
            ("crates/spanda-certify", "spanda-certify"),
            ("crates/spanda-fleet", "spanda-fleet"),
            ("crates/spanda-ota", "spanda-ota"),
            ("crates/spanda-deploy-http", "spanda-deploy-http"),
        ],
    ),
    (
        "Transport and connectivity",
        [
            ("crates/spanda-transport", "spanda-transport"),
            ("crates/spanda-transport-routing", "spanda-transport-routing"),
            ("crates/spanda-transport-ros2", "spanda-transport-ros2"),
            ("crates/spanda-transport-mqtt", "spanda-transport-mqtt"),
            ("crates/spanda-transport-dds", "spanda-transport-dds"),
            ("crates/spanda-transport-websocket", "spanda-transport-websocket"),
            ("crates/spanda-connectivity", "spanda-connectivity"),
            ("crates/spanda-connectivity-runtime", "spanda-connectivity-runtime"),
        ],
    ),
    (
        "Tooling and codegen",
        [
            ("crates/spanda-format", "spanda-format"),
            ("crates/spanda-lint", "spanda-lint"),
            ("crates/spanda-codegen", "spanda-codegen"),
            ("crates/spanda-docs", "spanda-docs"),
            ("crates/spanda-modules", "spanda-modules"),
            ("crates/spanda-llvm", "spanda-llvm"),
            ("crates/spanda-rt", "spanda-rt"),
        ],
    ),
    (
        "FFI, bridge, providers, packages",
        [
            ("crates/spanda-bridge", "spanda-bridge"),
            ("crates/spanda-ffi", "spanda-ffi"),
            ("crates/spanda-lib-registry", "spanda-lib-registry"),
            ("crates/spanda-providers", "spanda-providers"),
            ("crates/spanda-package", "spanda-package"),
        ],
    ),
    (
        "Security and audit",
        [
            ("crates/spanda-security", "spanda-security"),
            ("crates/spanda-audit", "spanda-audit"),
        ],
    ),
]

# Optional crate (excluded from default workspace build)
RUST_OPTIONAL = [
    ("crates/spanda-ros2-rclrs-native", "spanda-ros2-rclrs-native"),
]

# `spanda_core::` path → canonical workspace crate (for embedders migrating off the facade)
FACADE_MAP: list[tuple[str, str, str]] = [
    ("check / compile / run", "spanda_core::", "spanda_driver::"),
    ("AST types", "spanda_core::ast", "spanda_ast::nodes"),
    ("Parser", "spanda_core::parser", "spanda_parser"),
    ("Lexer tokenize", "spanda_core::lexer", "spanda_driver::tokenize"),
    ("Type check", "spanda_core::types", "spanda_driver::type_check"),
    ("SIR", "spanda_core::sir", "spanda_sir"),
    ("Errors / diagnostics", "spanda_core::SpandaError", "spanda_error"),
    ("Hardware verify", "spanda_core::verify_compatibility", "spanda_driver / spanda_hardware"),
    ("Interpreter runtime", "spanda_core::runtime", "spanda_interpreter::runtime"),
    ("RoutingCommBus", "spanda_core::transport", "spanda_transport_routing"),
    ("Live transport hooks", "spanda_core::transport_live (removed)", "spanda_transport_routing::transport_live"),
    ("MQTT/DDS/WebSocket bridges", "spanda_core::transport_* (removed)", "spanda_transport_routing::live_bridges"),
    ("Providers", "spanda_core::providers", "spanda_providers"),
    ("Fleet / OTA", "spanda_core::fleet_* / deploy_*", "spanda_fleet / spanda_ota"),
    ("Format / lint / codegen", "spanda_core::format / lint / codegen", "spanda_format / spanda_lint / spanda_codegen"),
]

TS_ROOTS = [
    ("src", "TypeScript core"),
    ("packages/native", "@spanda/native"),
    ("packages/lsp", "@spanda/lsp"),
    ("packages/web/src", "@spanda/web"),
    ("editor/vscode/src", "VS Code extension"),
]

RE_PUB_MOD = re.compile(r"^pub mod (\w+)")
RE_MOD = re.compile(r"^(?:pub )?mod (\w+)")
RE_PUB_FN = re.compile(r"^pub (?:async )?fn (\w+)")
RE_PUB_STRUCT = re.compile(r"^pub struct (\w+)")
RE_PUB_ENUM = re.compile(r"^pub enum (\w+)")
RE_PUB_TRAIT = re.compile(r"^pub trait (\w+)")
RE_PUB_TYPE = re.compile(r"^pub type (\w+)")
RE_PUB_CONST = re.compile(r"^pub const (\w+)")
RE_IMPL = re.compile(r"^impl(?:<[^>]+>)?\s+(?:\w+\s+for\s+)?(\w+)")
RE_PUB_USE = re.compile(r"^pub use ([\w:]+(?:::\*)?);")

RE_TS_EXPORT_FN = re.compile(r"^export (?:async )?function (\w+)")
RE_TS_EXPORT_CLASS = re.compile(r"^export (?:default )?class (\w+)")
RE_TS_EXPORT_CONST = re.compile(r"^export const (\w+)")
RE_TS_EXPORT_TYPE = re.compile(r"^export type (\w+)")
RE_TS_EXPORT_INTERFACE = re.compile(r"^export interface (\w+)")
RE_TS_EXPORT_ENUM = re.compile(r"^export enum (\w+)")
RE_TS_REEXPORT = re.compile(r"^export \{([^}]+)\}")
RE_TS_MODULE = re.compile(r"@module\s+(\S+)")


@dataclass
class Item:
    kind: str
    name: str
    line: int


@dataclass
class ModuleDoc:
    path: Path
    rel: str
    module_name: str
    items: list[Item] = field(default_factory=list)
    submodule: str | None = None


def anchor(text: str) -> str:


    """








    Description:








    Anchor.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: str








    Return value from `anchor`.

















    Example:








    result = anchor(text)


    """
    return re.sub(r"[^\w\-]", "", text.lower().replace("_", "-").replace("/", "-"))


def link(path: Path, line: int, label: str) -> str:


    """








    Description:








    Link.

















    Inputs:








    path: Path








    Caller-supplied path.








    line: int








    Caller-supplied line.








    label: str








    Caller-supplied label.

















    Outputs:








    result: str








    Return value from `link`.

















    Example:








    result = link(path, line, label)


    """
    rel = path.relative_to(ROOT).as_posix()
    return f"[{label}](../{rel}#L{line})"


def module_label_from_rust(path: Path, crate_root: Path) -> str:


    """








    Description:








    Module label from rust.

















    Inputs:








    path: Path








    Caller-supplied path.








    crate_root: Path








    Caller-supplied crate root.

















    Outputs:








    result: str








    Return value from `module_label_from_rust`.

















    Example:








    result = module_label_from_rust(path, crate_root)


    """
    rel = path.relative_to(crate_root / "src")
    stem = rel.with_suffix("")
    if stem.name == "lib" or stem.name == "main":
        return stem.parent.as_posix() if stem.parent != Path(".") else "root"
    if stem.name == "mod":
        return stem.parent.as_posix()
    parts = list(stem.parts)
    if parts[-1] == "mod":
        parts = parts[:-1]
    return "/".join(parts) if parts else stem.as_posix()


def module_label_from_ts(path: Path, ts_root: Path) -> str:


    """








    Description:








    Module label from ts.

















    Inputs:








    path: Path








    Caller-supplied path.








    ts_root: Path








    Caller-supplied ts root.

















    Outputs:








    result: str








    Return value from `module_label_from_ts`.

















    Example:








    result = module_label_from_ts(path, ts_root)


    """
    rel = path.relative_to(ts_root)
    if rel.name == "index.ts":
        parent = rel.parent
        return parent.as_posix() if parent != Path(".") else "root"
    return rel.with_suffix("").as_posix()


def parse_rust_file(path: Path, crate_root: Path) -> ModuleDoc:


    """








    Description:








    Parse rust file.

















    Inputs:








    path: Path








    Caller-supplied path.








    crate_root: Path








    Caller-supplied crate root.

















    Outputs:








    result: ModuleDoc








    Return value from `parse_rust_file`.

















    Example:








    result = parse_rust_file(path, crate_root)


    """
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    rel = path.relative_to(ROOT).as_posix()
    mod_name = module_label_from_rust(path, crate_root)
    doc = ModuleDoc(path=path, rel=rel, module_name=mod_name)

    in_impl: str | None = None
    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("//") or not stripped:
            continue
        if m := RE_PUB_MOD.match(stripped):
            doc.items.append(Item("mod", m.group(1), i))
            continue
        if m := RE_PUB_USE.match(stripped):
            doc.items.append(Item("re-export", m.group(1), i))
            continue
        if m := RE_IMPL.match(stripped):
            in_impl = m.group(1)
            doc.items.append(Item("impl", in_impl, i))
            continue
        if stripped == "}" and in_impl:
            in_impl = None
            continue
        prefix = "method" if in_impl else "fn"
        if m := RE_PUB_FN.match(stripped):
            doc.items.append(Item(prefix, f"{in_impl}::{m.group(1)}" if in_impl else m.group(1), i))
        elif m := RE_PUB_STRUCT.match(stripped):
            doc.items.append(Item("struct", m.group(1), i))
        elif m := RE_PUB_ENUM.match(stripped):
            doc.items.append(Item("enum", m.group(1), i))
        elif m := RE_PUB_TRAIT.match(stripped):
            doc.items.append(Item("trait", m.group(1), i))
        elif m := RE_PUB_TYPE.match(stripped):
            doc.items.append(Item("type", m.group(1), i))
        elif m := RE_PUB_CONST.match(stripped):
            doc.items.append(Item("const", m.group(1), i))
    return doc


def parse_ts_file(path: Path, ts_root: Path) -> ModuleDoc:


    """








    Description:








    Parse ts file.

















    Inputs:








    path: Path








    Caller-supplied path.








    ts_root: Path








    Caller-supplied ts root.

















    Outputs:








    result: ModuleDoc








    Return value from `parse_ts_file`.

















    Example:








    result = parse_ts_file(path, ts_root)


    """
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    rel = path.relative_to(ROOT).as_posix()
    mod_name = module_label_from_ts(path, ts_root)
    doc = ModuleDoc(path=path, rel=rel, module_name=mod_name)

    for i, line in enumerate(lines, start=1):
        stripped = line.strip()
        if m := RE_TS_MODULE.search(stripped):
            doc.submodule = m.group(1)
        for pat, kind in (
            (RE_TS_EXPORT_FN, "fn"),
            (RE_TS_EXPORT_CLASS, "class"),
            (RE_TS_EXPORT_CONST, "const"),
            (RE_TS_EXPORT_TYPE, "type"),
            (RE_TS_EXPORT_INTERFACE, "interface"),
            (RE_TS_EXPORT_ENUM, "enum"),
        ):
            if m := pat.match(stripped):
                doc.items.append(Item(kind, m.group(1), i))
                break
        else:
            if m := RE_TS_REEXPORT.match(stripped):
                names = [n.strip().split(" as ")[0].strip() for n in m.group(1).split(",")]
                for name in names:
                    if name:
                        doc.items.append(Item("re-export", name, i))
    return doc


def collect_rust(crate_rel: str, crate_name: str) -> list[ModuleDoc]:


    """








    Description:








    Collect rust.

















    Inputs:








    crate_rel: str








    Caller-supplied crate rel.








    crate_name: str








    Caller-supplied crate name.

















    Outputs:








    result: list[ModuleDoc]








    Return value from `collect_rust`.

















    Example:








    result = collect_rust(crate_rel, crate_name)


    """
    crate_root = ROOT / crate_rel
    src = crate_root / "src"
    if not src.exists():
        return []
    docs: list[ModuleDoc] = []
    for path in sorted(src.rglob("*.rs")):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        docs.append(parse_rust_file(path, crate_root))
    return docs


def collect_ts(ts_rel: str, _label: str) -> list[ModuleDoc]:


    """








    Description:








    Collect ts.

















    Inputs:








    ts_rel: str








    Caller-supplied ts rel.








    _label: str








    Caller-supplied label.

















    Outputs:








    result: list[ModuleDoc]








    Return value from `collect_ts`.

















    Example:








    result = collect_ts(ts_rel, _label)


    """
    ts_root = ROOT / ts_rel
    if not ts_root.exists():
        return []
    docs: list[ModuleDoc] = []
    for path in sorted(ts_root.rglob("*.ts")):
        if path.name.endswith(".d.ts"):
            continue
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        docs.append(parse_ts_file(path, ts_root))
    return docs


def kind_icon(kind: str) -> str:


    """








    Description:








    Kind icon.

















    Inputs:








    kind: str








    Caller-supplied kind.

















    Outputs:








    result: str








    Return value from `kind_icon`.

















    Example:








    result = kind_icon(kind)


    """
    return {
        "fn": "fn",
        "method": "method",
        "struct": "struct",
        "enum": "enum",
        "trait": "trait",
        "type": "type",
        "const": "const",
        "class": "class",
        "interface": "interface",
        "mod": "mod",
        "impl": "impl",
        "re-export": "export",
    }.get(kind, kind)


def render_module_section(
    doc: ModuleDoc, heading_level: int, section_id: str
) -> list[str]:


    """








    Description:








    Render module section.

















    Inputs:








    doc: ModuleDoc








    Caller-supplied doc.








    heading_level: int








    Caller-supplied heading level.








    section_id: str








    Caller-supplied section id.

















    Outputs:








    result: list[str]








    Return value from `render_module_section`.

















    Example:








    result = render_module_section(doc, heading_level, section_id)


    """
    if not doc.items and doc.submodule is None:
        return []
    h = "#" * heading_level
    name = doc.submodule or doc.module_name
    out: list[str] = []
    file_link = link(doc.path, 1, doc.rel)
    out.append(f"{h} `{name}` {{#{section_id}}}")
    out.append("")
    out.append(f"Source: {file_link}")
    out.append("")

    by_kind: dict[str, list[Item]] = {}
    for item in doc.items:
        by_kind.setdefault(item.kind, []).append(item)

    order = [
        "mod",
        "re-export",
        "struct",
        "enum",
        "trait",
        "type",
        "interface",
        "class",
        "const",
        "impl",
        "fn",
        "method",
    ]
    for kind in order:
        items = by_kind.get(kind, [])
        if kind == "impl":
            items = [i for i in items if i.name not in ("crate", "Self")]
        if not items:
            continue
        out.append(f"**{kind_icon(kind)}**")
        out.append("")
        for item in sorted(items, key=lambda x: x.name.lower()):
            out.append(f"- {link(doc.path, item.line, f'`{item.name}`')}")
        out.append("")
    return out


def render_toc(entries: list[tuple[str, str]], level: int = 0) -> list[str]:


    """








    Description:








    Render toc.

















    Inputs:








    entries: list[tuple[str, str]]








    Caller-supplied entries.








    level: int








    Caller-supplied level.

















    Outputs:








    result: list[str]








    Return value from `render_toc`.

















    Example:








    result = render_toc(entries, level)


    """
    indent = "  " * level
    out: list[str] = []
    for label, anchor_id in entries:
        out.append(f"{indent}- [{label}](#{anchor_id})")
    return out


def main() -> None:


    """








    Description:








    Main.

















    Inputs:








    None.

















    Outputs:








    None.

















    Example:








    result = main()


    """
    lines: list[str] = []
    lines.append("# API Reference")
    lines.append("")
    lines.append(
        "Hierarchical index of **Rust crates**, **TypeScript packages**, and their "
        "public modules, types, and functions. Each entry links to its source definition "
        "(file and line)."
    )
    lines.append("")
    lines.append(
        "For the **Spanda language** (`.sd` syntax, `std.*`, triggers, CLI), see "
        "[spanda-reference.md](./spanda-reference.md). For how the API docs fit together, see "
        "[api-documentation.md](./api-documentation.md)."
    )
    lines.append("")
    lines.append(
        "**Hierarchy:** language reference → Rust/TS compiler API (this file) → "
        "JSON wire contract → per-crate READMEs in [crates/README.md](../crates/README.md)."
    )
    lines.append("")
    lines.append(
        "> Generated by `scripts/generate_api_reference.py`. Re-run after large API changes:"
    )
    lines.append(">")
    lines.append("> ```bash")
    lines.append("> python3 scripts/generate_api_reference.py")
    lines.append("> ```")
    lines.append("")

    # Table of contents
    lines.append("## Contents")
    lines.append("")

    ts_toc: list[tuple[str, str]] = []
    for ts_rel, ts_name in TS_ROOTS:
        ts_slug = ts_rel.replace("/", "-").replace("@", "")
        ts_toc.append((ts_name, anchor(f"ts-{ts_slug}")))

    lines.append("- [Documentation map](#documentation-map)")
    lines.append("- [Facade → workspace mapping](#facade--workspace-mapping)")
    lines.append("- [Rust crates](#rust-crates)")
    for group_name, crates in RUST_GROUPS:
        group_id = anchor(f"group-{group_name}")
        lines.append(f"  - [{group_name}](#{group_id})")
        for _crate_rel, crate_name in crates:
            lines.append(f"    - [{crate_name}](#{anchor(f'crate-{crate_name}')})")
    if RUST_OPTIONAL:
        lines.append("  - [Optional](#group-optional)")
        for _crate_rel, crate_name in RUST_OPTIONAL:
            lines.append(f"    - [{crate_name}](#{anchor(f'crate-{crate_name}')})")
    lines.append("- [TypeScript](#typescript)")
    lines.extend(render_toc(ts_toc, 1))
    lines.append("- [Quick lookup](#quick-lookup)")
    lines.append("")

    lines.append("## Documentation map")
    lines.append("")
    lines.append("| Layer | Document | Audience |")
    lines.append("|-------|----------|----------|")
    lines.append("| Spanda language (`.sd`) | [spanda-reference.md](./spanda-reference.md) | Authors, reviewers |")
    lines.append("| Language guide | [spanda-language.md](./spanda-language.md) | Tutorial + semantics |")
    lines.append("| Compiler / tooling API | **this file** | Rust/TS integrators |")
    lines.append("| JSON CLI/LSP contract | [api-contract.json](./api-contract.json) | Playground, LSP, bindings |")
    lines.append("| Crate layout | [crates/README.md](../crates/README.md) | Contributors |")
    lines.append("| Stable embedder facade | `spanda_core::` via [spanda-core](../crates/spanda-core/README.md) | External Rust consumers |")
    lines.append("")

    lines.append("## Facade → workspace mapping")
    lines.append("")
    lines.append(
        "In-repo code and new integrations should use the **canonical crate** column. "
        "`spanda_core::` remains stable for external embedders."
    )
    lines.append("")
    lines.append("| API | `spanda_core::` (facade) | Canonical workspace crate |")
    lines.append("|-----|--------------------------|---------------------------|")
    for label, facade, canonical in FACADE_MAP:
        lines.append(f"| {label} | `{facade}` | `{canonical}` |")
    lines.append("")

    all_symbols: list[tuple[str, str, str]] = []

    # Rust crates (grouped)
    lines.append("## Rust crates")
    lines.append("")
    lines.append(
        "Crates are grouped by lean-core layer. Module trees below list `pub` items "
        "with source links."
    )
    lines.append("")

    def render_crate(crate_rel: str, crate_name: str) -> None:

        """

    

        Description:

    

        Render crate.

    

    

    

        Inputs:

    

        crate_rel: str

    

        Caller-supplied crate rel.

    

        crate_name: str

    

        Caller-supplied crate name.

    

    

    

        Outputs:

    

        None.

    

    

    

        Example:

    

        result = render_crate(crate_rel, crate_name)

        """
        docs = collect_rust(crate_rel, crate_name)
        if not docs:
            return
        crate_id = anchor(f"crate-{crate_name}")
        lines.append(f"#### `{crate_name}` {{#{crate_id}}}")
        lines.append("")
        readme = ROOT / crate_rel / "README.md"
        if readme.exists():
            lines.append(
                f"Crate root: [`{crate_rel}`](../{crate_rel}/) · "
                f"[README](../{crate_rel}/README.md)"
            )
        else:
            lines.append(f"Crate root: [`{crate_rel}`](../{crate_rel}/)")
        lines.append("")

        by_mod: dict[str, list[ModuleDoc]] = {}
        for doc in docs:
            by_mod.setdefault(doc.module_name, []).append(doc)

        mod_toc: list[tuple[str, str]] = []
        for mod_name in sorted(by_mod.keys(), key=lambda s: (s.count("/"), s.lower())):
            mod_toc.append((mod_name, anchor(f"{crate_name}-{mod_name}")))
        lines.append("**Modules**")
        lines.append("")
        lines.extend(render_toc(mod_toc))
        lines.append("")

        for mod_name in sorted(by_mod.keys(), key=lambda s: (s.count("/"), s.lower())):
            for doc in by_mod[mod_name]:
                sid = anchor(f"{crate_name}-{mod_name}")
                section = render_module_section(doc, 5, sid)
                if section:
                    lines.extend(section)
                for item in doc.items:
                    if item.kind in ("fn", "method", "struct", "enum", "trait", "class"):
                        all_symbols.append((item.name, crate_name, doc.rel))

    for group_name, crates in RUST_GROUPS:
        group_id = anchor(f"group-{group_name}")
        lines.append(f"### {group_name} {{#{group_id}}}")
        lines.append("")
        for crate_rel, crate_name in crates:
            render_crate(crate_rel, crate_name)

    if RUST_OPTIONAL:
        lines.append("### Optional {{#group-optional}}")
        lines.append("")
        lines.append("Excluded from default `cargo build --workspace`; built when ROS2 native SDK is enabled.")
        lines.append("")
        for crate_rel, crate_name in RUST_OPTIONAL:
            render_crate(crate_rel, crate_name)

    # TypeScript
    lines.append("## TypeScript")
    lines.append("")

    for ts_rel, ts_name in TS_ROOTS:
        docs = collect_ts(ts_rel, ts_name)
        if not docs:
            continue
        ts_slug = ts_rel.replace("/", "-").replace("@", "")
        ts_id = anchor(f"ts-{ts_slug}")
        lines.append(f"### `{ts_name}` {{#{ts_id}}}")
        lines.append("")
        lines.append(f"Root: [`{ts_rel}`](../{ts_rel}/)")
        lines.append("")

        by_mod: dict[str, list[ModuleDoc]] = {}
        for doc in docs:
            key = doc.submodule or doc.module_name
            by_mod.setdefault(key, []).append(doc)

        mod_toc = [(m, anchor(f"{ts_name}-{m}")) for m in sorted(by_mod.keys(), key=str.lower)]
        lines.append("**Modules**")
        lines.append("")
        lines.extend(render_toc(mod_toc))
        lines.append("")

        for mod_name in sorted(by_mod.keys(), key=str.lower):
            for doc in by_mod[mod_name]:
                sid = anchor(f"{ts_name}-{mod_name}")
                section = render_module_section(doc, 4, sid)
                if section:
                    lines.extend(section)
                for item in doc.items:
                    if item.kind in ("fn", "class", "interface"):
                        all_symbols.append((item.name, ts_name, doc.rel))

    # Alphabetical quick lookup (top symbols only, deduped)
    lines.append("## Quick lookup")
    lines.append("")
    lines.append("Alphabetical index of public functions, methods, structs, enums, traits, and classes.")
    lines.append("")
    seen: set[str] = set()
    for name, package, rel in sorted(all_symbols, key=lambda x: x[0].lower()):
        key = f"{name}:{rel}"
        if key in seen:
            continue
        seen.add(key)
        lines.append(f"- `{name}` — {package} ([`{rel}`](../{rel}))")

    lines.append("")
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {OUT} ({len(lines)} lines, {len(seen)} symbols)")


if __name__ == "__main__":
    main()
