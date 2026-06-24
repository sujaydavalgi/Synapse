#!/usr/bin/env python3
"""Add inline documentation blocks inside Rust and TypeScript functions."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SKIP_PATH_PARTS = {"target", "node_modules", ".git"}
SKIP_FILES: set[str] = set()

RUST_FN_HEAD = re.compile(
    r"(?m)^(?P<indent>\s*)"
    r"(?:(?:pub\s*\([^)]*\)\s+|pub\s+|async\s+|unsafe\s+|const\s+)*)"
    r"fn\s+(?P<name>\w+)\s*"
)


def skip_string_literal(text: str, i: int) -> int:


    """








    Description:








    Skip string literal.

















    Inputs:








    text: str








    Caller-supplied text.








    i: int








    Caller-supplied i.

















    Outputs:








    result: int








    Return value from `skip_string_literal`.

















    Example:








    result = skip_string_literal(text, i)


    """
    if i >= len(text):
        return i
    if text.startswith("r#", i):
        hash_count = 0
        j = i + 1
        while j < len(text) and text[j] == "#":
            hash_count += 1
            j += 1
        if j < len(text) and text[j] in "\"'":
            quote = text[j]
            j += 1
            while j < len(text):
                if text[j] == quote:
                    if hash_count == 0:
                        break
                    if text[j + 1 : j + 1 + hash_count] == "#" * hash_count:
                        j += hash_count
                        break
                j += 1
            return j
    if text[i] in "\"'":
        quote = text[i]
        j = i + 1
        while j < len(text):
            if text[j] == "\\":
                j += 2
                continue
            if text[j] == quote:
                return j
            j += 1
        return j
    return i


def scan_balanced(text: str, start: int, open_ch: str, close_ch: str) -> int | None:


    """








    Description:








    Scan balanced.

















    Inputs:








    text: str








    Caller-supplied text.








    start: int








    Caller-supplied start.








    open_ch: str








    Caller-supplied open ch.








    close_ch: str








    Caller-supplied close ch.

















    Outputs:








    result: int | None








    Return value from `scan_balanced`.

















    Example:








    result = scan_balanced(text, start, open_ch, close_ch)


    """
    if start >= len(text) or text[start] != open_ch:
        return None
    depth = 0
    i = start
    while i < len(text):
        if text.startswith("//", i):
            i = text.find("\n", i)
            if i == -1:
                return None
            continue
        if text.startswith("/*", i):
            end = text.find("*/", i + 2)
            if end == -1:
                return None
            i = end + 2
            continue
        ch = text[i]
        if ch == open_ch:
            depth += 1
        elif ch == close_ch:
            depth -= 1
            if depth == 0:
                return i
        elif ch in "\"'" or (
            text.startswith("r", i) and i + 1 < len(text) and text[i + 1] in "'\""
        ):
            i = skip_string_literal(text, i)
        i += 1
    return None


def find_rust_functions(text: str) -> list[FnMatch]:


    """








    Description:








    Find rust functions.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: list[FnMatch]








    Return value from `find_rust_functions`.

















    Example:








    result = find_rust_functions(text)


    """
    matches: list[FnMatch] = []
    for m in RUST_FN_HEAD.finditer(text):
        pos = m.end()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos < len(text) and text[pos] == "<":
            end = scan_balanced(text, pos, "<", ">")
            if end is None:
                continue
            pos = end + 1
            while pos < len(text) and text[pos].isspace():
                pos += 1
        if pos >= len(text) or text[pos] != "(":
            continue
        params_end = scan_balanced(text, pos, "(", ")")
        if params_end is None:
            continue
        params = text[pos + 1 : params_end]
        pos = params_end + 1
        while pos < len(text) and text[pos].isspace():
            pos += 1
        ret: str | None = None
        if text.startswith("->", pos):
            pos += 2
            ret_start = pos
            while pos < len(text) and text[pos] not in "{;":
                pos += 1
            ret = text[ret_start:pos].strip()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text):
            continue
        if text[pos] == ";":
            continue
        if text[pos] != "{":
            continue
        matches.append(
            FnMatch(
                start=m.start(),
                body_start=pos + 1,
                indent=m.group("indent"),
                name=m.group("name"),
                params=params,
                ret=ret,
            )
        )
    return matches


@dataclass
class FnMatch:
    start: int
    body_start: int
    indent: str
    name: str
    params: str
    ret: str | None


def snake_to_words(name: str) -> str:


    """








    Description:








    Snake to words.

















    Inputs:








    name: str








    Caller-supplied name.

















    Outputs:








    result: str








    Return value from `snake_to_words`.

















    Example:








    result = snake_to_words(name)


    """
    parts = name.strip("_").split("_")
    return " ".join(parts) if parts else name


def describe_name(name: str) -> str:


    """








    Description:








    Describe name.

















    Inputs:








    name: str








    Caller-supplied name.

















    Outputs:








    result: str








    Return value from `describe_name`.

















    Example:








    result = describe_name(name)


    """
    lower = name.lower()
    if name == "new":
        return "Create a new instance"
    if name == "default":
        return "Return the default value"
    if lower.startswith("is_"):
        return f"Return whether {snake_to_words(name[3:])}"
    if lower.startswith("has_"):
        return f"Return whether this value has {snake_to_words(name[4:])}"
    if lower.startswith("from_"):
        return f"Construct from {snake_to_words(name[5:])}"
    if lower.startswith("to_"):
        return f"Convert to {snake_to_words(name[3:])}"
    if lower.startswith("into_"):
        return f"Convert into {snake_to_words(name[5:])}"
    if lower.startswith("as_"):
        return f"Return as {snake_to_words(name[3:])}"
    if lower.startswith("get_"):
        return f"Return {snake_to_words(name[4:])}"
    if lower.startswith("set_"):
        return f"Set {snake_to_words(name[4:])}"
    if lower.startswith("with_"):
        return f"Return a copy with {snake_to_words(name[5:])} updated"
    if lower.startswith("parse_") or name == "parse":
        suffix = name[6:] if lower.startswith("parse_") else "input"
        return f"Parse {snake_to_words(suffix)}"
    if name in {"load", "save", "register", "unregister", "reset", "clear", "normalize"}:
        return f"{name.capitalize()} the value"
    if lower.endswith("_count"):
        return f"Return the number of {snake_to_words(name[:-6])}"
    if lower.startswith("register_"):
        return f"Register {snake_to_words(name[9:])}"
    if lower.startswith("find_"):
        return f"Find {snake_to_words(name[5:])}"
    if lower.startswith("render_"):
        return f"Render {snake_to_words(name[7:])}"
    if lower.startswith("check_") or name == "check":
        return f"Check {snake_to_words(name[6:] if lower.startswith('check_') else 'input')}"
    if lower.startswith("validate_") or name == "validate":
        return f"Validate {snake_to_words(name[9:] if lower.startswith('validate_') else 'input')}"
    if lower.startswith("run_") or name == "run":
        return f"Run {snake_to_words(name[4:] if lower.startswith('run_') else 'the operation')}"
    if lower.startswith("emit_"):
        return f"Emit {snake_to_words(name[5:])}"
    if lower.startswith("handle_"):
        return f"Handle {snake_to_words(name[7:])}"
    if lower.startswith("dispatch_"):
        return f"Dispatch {snake_to_words(name[9:])}"
    words = snake_to_words(name)
    return words[0].upper() + words[1:] if words else name


def parse_rust_params(raw: str) -> list[tuple[str, str]]:


    """








    Description:








    Parse rust params.

















    Inputs:








    raw: str








    Caller-supplied raw.

















    Outputs:








    result: list[tuple[str, str]]








    Return value from `parse_rust_params`.

















    Example:








    result = parse_rust_params(raw)


    """
    raw = raw.strip()
    if not raw:
        return []
    params: list[tuple[str, str]] = []
    depth = 0
    current: list[str] = []
    for ch in raw + ",":
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}":
            depth -= 1
        if ch == "," and depth == 0:
            piece = "".join(current).strip()
            current = []
            if not piece:
                continue
            if piece in {"&mut self", "&self", "self"}:
                params.append(("self", "method receiver"))
                continue
            if piece.startswith("&mut "):
                name = piece[5:].split(":")[0].strip()
                params.append((name, "mutable borrow"))
                continue
            if piece.startswith("&"):
                name = piece[1:].split(":")[0].strip()
                params.append((name, "shared borrow"))
                continue
            name = piece.split(":")[0].strip()
            if name:
                params.append((name, "input value"))
        else:
            current.append(ch)
    return params


def parse_ts_params(raw: str) -> list[tuple[str, str]]:


    """








    Description:








    Parse ts params.

















    Inputs:








    raw: str








    Caller-supplied raw.

















    Outputs:








    result: list[tuple[str, str]]








    Return value from `parse_ts_params`.

















    Example:








    result = parse_ts_params(raw)


    """
    raw = raw.strip()
    if not raw:
        return []
    params: list[tuple[str, str]] = []
    depth = 0
    current: list[str] = []
    for ch in raw + ",":
        if ch in "<([{":
            depth += 1
        elif ch in ">)]}":
            depth -= 1
        if ch == "," and depth == 0:
            piece = "".join(current).strip()
            current = []
            if not piece:
                continue
            name = piece.split(":")[0].split("=")[0].strip()
            if name.startswith("..."):
                params.append((name, "rest arguments"))
            elif name:
                optional = "?" in piece or "=" in piece
                params.append((name, "optional input" if optional else "input value"))
        else:
            current.append(ch)
    return params


def describe_return(ret: str | None, lang: str) -> str:


    """








    Description:








    Describe return.

















    Inputs:








    ret: str | None








    Caller-supplied ret.








    lang: str








    Caller-supplied lang.

















    Outputs:








    result: str








    Return value from `describe_return`.

















    Example:








    result = describe_return(ret, lang)


    """
    if not ret:
        return "Nothing."
    ret = ret.strip()
    if ret in {"()", "void"}:
        return "Nothing."
    if ret == "Self":
        return "A new instance of this type."
    if ret.startswith("Option<") or ret.endswith("| null") or ret.endswith("| undefined"):
        return "Some value on success, otherwise none."
    if ret.startswith("Result<") or "Promise<" in ret:
        return "Success value on completion, or an error."
    if ret in {"bool", "boolean"}:
        return "true or false."
    if ret in {"usize", "u32", "u64", "i32", "i64", "f32", "f64", "number"}:
        return "Numeric result."
    if ret in {"String", "string", "&str", "&'static str"}:
        return "Text result."
    if ret in {"()", "void", "undefined"}:
        return "Nothing."
    return f"{ret}."


def has_inline_doc(text: str, body_start: int) -> bool:


    """








    Description:








    Has inline doc.

















    Inputs:








    text: str








    Caller-supplied text.








    body_start: int








    Caller-supplied body start.

















    Outputs:








    result: bool








    Return value from `has_inline_doc`.

















    Example:








    result = has_inline_doc(text, body_start)


    """
    snippet = text[body_start : body_start + 800]
    return bool(
        re.search(r"^\s*//\s*Parameters:", snippet, re.M)
        or re.search(r"^\s*//\s*Returns:", snippet, re.M)
    )


def is_test_context(text: str, start: int, name: str) -> bool:


    """








    Description:








    Is test context.

















    Inputs:








    text: str








    Caller-supplied text.








    start: int








    Caller-supplied start.








    name: str








    Caller-supplied name.

















    Outputs:








    result: bool








    Return value from `is_test_context`.

















    Example:








    result = is_test_context(text, start, name)


    """
    # Document all functions including tests; only skip nested test helper decls in mod tests
    # when they are clearly private fixtures duplicated elsewhere.
    return False


def example_lines(name: str, params: list[tuple[str, str]], module_hint: str, lang: str) -> list[str]:


    """








    Description:








    Example lines.

















    Inputs:








    name: str








    Caller-supplied name.








    params: list[tuple[str, str]]








    Caller-supplied params.








    module_hint: str








    Caller-supplied module hint.








    lang: str








    Caller-supplied lang.

















    Outputs:








    result: list[str]








    Return value from `example_lines`.

















    Example:








    result = example_lines(name, params, module_hint, lang)


    """
    call_params = ", ".join(p[0] for p in params if p[0] != "self")
    if lang == "rust":
        if name == "new":
            return [f"let value = {module_hint}::new({call_params});"]
        if name == "default":
            return [f"let value = {module_hint}::default();"]
        if params and params[0][0] == "self":
            return [f"let result = instance.{name}({call_params});"]
        return [f"let result = {module_hint}::{name}({call_params});"]
    if params and params[0][0] == "self":
        return [f"const result = instance.{name}({call_params});"]
    return [f"const result = {name}({call_params});"]


def module_hint_from_path(path: Path) -> str:


    """








    Description:








    Module hint from path.

















    Inputs:








    path: Path








    Caller-supplied path.

















    Outputs:








    result: str








    Return value from `module_hint_from_path`.

















    Example:








    result = module_hint_from_path(path)


    """
    parts = list(path.parts)
    if "crates" in parts:
        idx = parts.index("crates")
        crate = parts[idx + 1].replace("-", "_")
        if path.name == "lib.rs":
            return crate
        stem = path.stem.replace("-", "_")
        return f"{crate}::{stem}"
    if path.suffix == ".ts":
        return path.stem
    return path.stem


def module_purpose(path: Path, lang: str) -> str | None:


    """








    Description:








    Module purpose.

















    Inputs:








    path: Path








    Caller-supplied path.








    lang: str








    Caller-supplied lang.

















    Outputs:








    result: str | None








    Return value from `module_purpose`.

















    Example:








    result = module_purpose(path, lang)


    """
    text = path.read_text(encoding="utf-8")
    if lang == "rust":
        if text.lstrip().startswith("//!"):
            return None
        stem = path.stem.replace("_", " ")
        if path.name == "lib.rs":
            crate = path.parent.name.replace("-", " ")
            return f"//! {crate} crate public API and re-exports.\n//!\n"
        if path.name == "mod.rs":
            mod_name = path.parent.name.replace("_", " ")
            return f"//! {mod_name} module for Spanda.\n//!\n"
        if path.name == "manifest.rs":
            return "//! `spanda.toml` manifest parsing, validation, and project root discovery.\n//!\n"
        return f"//! {stem} support for Spanda.\n//!\n"
    if lang == "ts":
        if re.search(r"@module", text[:800]):
            return None
        if re.match(r"\s*/\*\*", text):
            return None
        stem = path.stem.replace("-", " ")
        rel = path.relative_to(ROOT / "src") if (ROOT / "src") in path.parents else path.name
        return f"/**\n * {stem} module ({rel}).\n * @module\n */\n\n"
    return None


def build_doc_block(
    indent: str,
    name: str,
    params: list[tuple[str, str]],
    ret: str | None,
    module_hint: str,
    lang: str,
) -> str:


    """








    Description:








    Build doc block.

















    Inputs:








    indent: str








    Caller-supplied indent.








    name: str








    Caller-supplied name.








    params: list[tuple[str, str]]








    Caller-supplied params.








    ret: str | None








    Caller-supplied ret.








    module_hint: str








    Caller-supplied module hint.








    lang: str








    Caller-supplied lang.

















    Outputs:








    result: str








    Return value from `build_doc_block`.

















    Example:








    result = build_doc_block(indent, name, params, ret, module_hint, lang)


    """
    desc = describe_name(name)
    lines = [f"{indent}// {desc}."]
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Parameters:")
    if params:
        for pname, phint in params:
            lines.append(f"{indent}// - `{pname}` — {phint}")
    else:
        lines.append(f"{indent}// None.")
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Returns:")
    lines.append(f"{indent}// {describe_return(ret, lang)}")
    optional = [p for p in params if "optional" in p[1]]
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Options:")
    if optional:
        for pname, _ in optional:
            lines.append(f"{indent}// - `{pname}` — optional parameter")
    else:
        lines.append(f"{indent}// None.")
    lines.append(f"{indent}//")
    lines.append(f"{indent}// Example:")
    for ex in example_lines(name, params, module_hint, lang):
        lines.append(f"{indent}// {ex}")
    return "\n".join(lines) + "\n"


def process_rust(path: Path) -> bool:


    """








    Description:








    Process rust.

















    Inputs:








    path: Path








    Caller-supplied path.

















    Outputs:








    result: bool








    Return value from `process_rust`.

















    Example:








    result = process_rust(path)


    """
    text = path.read_text(encoding="utf-8")
    original = text
    module_doc = module_purpose(path, "rust")
    if module_doc:
        text = module_doc + text

    module_hint = module_hint_from_path(path)
    inserts: list[tuple[int, str]] = []

    for fm in find_rust_functions(text):
        name = fm.name
        if is_test_context(text, fm.start, name):
            continue
        if has_inline_doc(text, fm.body_start):
            continue
        params = parse_rust_params(fm.params)
        doc = build_doc_block(fm.indent + "    ", name, params, fm.ret, module_hint, "rust")
        inserts.append((fm.body_start, doc))

    for pos, doc in reversed(inserts):
        text = text[:pos] + "\n" + doc + text[pos:]

    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def find_ts_callables(text: str, is_method: bool) -> list[FnMatch]:


    """








    Description:








    Find ts callables.

















    Inputs:








    text: str








    Caller-supplied text.








    is_method: bool








    Caller-supplied is method.

















    Outputs:








    result: list[FnMatch]








    Return value from `find_ts_callables`.

















    Example:








    result = find_ts_callables(text, is_method)


    """
    head = re.compile(
        r"(?m)^(?P<indent>\s+)"
        r"(?:(?:public|private|protected|static|async|readonly)\s+)+"
        r"(?P<name>[a-zA-Z_]\w*)\s*"
    ) if is_method else re.compile(
        r"(?m)^(?P<indent>\s*)"
        r"(?:(?:export\s+|async\s+|static\s+)*)"
        r"function\s+(?P<name>\w+)\s*"
    )
    matches: list[FnMatch] = []
    for m in head.finditer(text):
        name = m.group("name")
        if name in {"if", "for", "while", "switch", "catch", "constructor", "super"}:
            continue
        pos = m.end()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text) or text[pos] != "(":
            continue
        params_end = scan_balanced(text, pos, "(", ")")
        if params_end is None:
            continue
        params = text[pos + 1 : params_end]
        pos = params_end + 1
        while pos < len(text) and text[pos].isspace():
            pos += 1
        ret: str | None = None
        if text[pos:].startswith(":"):
            pos += 1
            ret_start = pos
            while pos < len(text) and text[pos] not in "{=":
                pos += 1
            ret = text[ret_start:pos].strip()
        while pos < len(text) and text[pos].isspace():
            pos += 1
        if pos >= len(text):
            continue
        if text.startswith("=>", pos):
            pos += 2
            while pos < len(text) and text[pos].isspace():
                pos += 1
        if pos >= len(text) or text[pos] != "{":
            continue
        matches.append(
            FnMatch(
                start=m.start(),
                body_start=pos + 1,
                indent=m.group("indent"),
                name=name,
                params=params,
                ret=ret,
            )
        )
    return matches


TS_ARROW = re.compile(
    r"(?m)^(?P<indent>\s*)"
    r"(?:(?:export\s+|async\s+)*)"
    r"(?:const|let)\s+(?P<name>\w+)\s*=\s*"
    r"(?:async\s+)?"
    r"\((?P<params>[^)]*)\)"
    r"(?:\s*:\s*(?P<ret>[^=]+))?"
    r"\s*=>\s*\{"
)


def process_ts(path: Path) -> bool:


    """








    Description:








    Process ts.

















    Inputs:








    path: Path








    Caller-supplied path.

















    Outputs:








    result: bool








    Return value from `process_ts`.

















    Example:








    result = process_ts(path)


    """
    text = path.read_text(encoding="utf-8")
    original = text
    module_doc = module_purpose(path, "ts")
    if module_doc:
        if text.startswith("#!"):
            first_nl = text.find("\n") + 1
            text = text[:first_nl] + module_doc + text[first_nl:]
        else:
            text = module_doc + text

    module_hint = module_hint_from_path(path)
    inserts: list[tuple[int, str]] = []

    for fm in find_ts_callables(text, is_method=False):
        if has_inline_doc(text, fm.body_start):
            continue
        params = parse_ts_params(fm.params)
        doc = build_doc_block(fm.indent + "  ", fm.name, params, fm.ret, module_hint, "ts")
        inserts.append((fm.body_start, doc))

    for fm in find_ts_callables(text, is_method=True):
        if has_inline_doc(text, fm.body_start):
            continue
        params = parse_ts_params(fm.params)
        doc = build_doc_block(fm.indent + "  ", fm.name, params, fm.ret, module_hint, "ts")
        inserts.append((fm.body_start, doc))

    for m in TS_ARROW.finditer(text):
        name = m.group("name")
        body_start = m.end()
        if has_inline_doc(text, body_start):
            continue
        params = parse_ts_params(m.group("params") or "")
        ret = m.group("ret")
        doc = build_doc_block(m.group("indent") + "  ", name, params, ret, module_hint, "ts")
        inserts.append((body_start, doc))

    for pos, doc in reversed(inserts):
        text = text[:pos] + "\n" + doc + text[pos:]

    if text != original:
        path.write_text(text, encoding="utf-8")
        return True
    return False


def should_process(path: Path) -> tuple[str | None, bool]:


    """








    Description:








    Should process.

















    Inputs:








    path: Path








    Caller-supplied path.

















    Outputs:








    result: tuple[str | None, bool]








    Return value from `should_process`.

















    Example:








    result = should_process(path)


    """
    if any(part in SKIP_PATH_PARTS for part in path.parts):
        return None, False
    if path.name in SKIP_FILES:
        return None, False
    if path.suffix == ".rs" and "crates" in path.parts:
        return "rust", True
    if path.suffix == ".ts" and (
        "src" in path.parts or "packages" in path.parts
    ) and "test" not in path.name and not path.name.endswith(".test.ts"):
        return "ts", True
    return None, False


def main() -> int:


    """








    Description:








    Main.

















    Inputs:








    None.

















    Outputs:








    result: int








    Return value from `main`.

















    Example:








    result = main()


    """
    changed = 0
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file():
            continue
        lang, ok = should_process(path)
        if not ok or lang is None:
            continue
        if lang == "rust" and process_rust(path):
            changed += 1
            print(f"updated rust: {path.relative_to(ROOT)}")
        elif lang == "ts" and process_ts(path):
            changed += 1
            print(f"updated ts: {path.relative_to(ROOT)}")
    print(f"\nDone. Updated {changed} files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
