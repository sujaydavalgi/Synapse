#!/usr/bin/env python3
"""Add plain-English block comments before control-flow and steps inside functions."""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_PATH_PARTS = {"target", "node_modules", ".git", "dist", "__pycache__"}

RUST_LOGIC = re.compile(
    r"^\s*(?:"
    r"(?:else\s+)?if(?:\s+let|\s+\(|(?=\s))"
    r"|else\b"
    r"|match\b"
    r"|for\b"
    r"|while\b"
    r"|loop\b"
    r")"
)
TS_LOGIC = re.compile(
    r"^\s*(?:"
    r"\}?\s*else\s+if\s*\("
    r"|\}?\s*else\s*\{"
    r"|if\s*\("
    r"|switch\s*\("
    r"|for\s*\("
    r"|while\s*\("
    r"|do\s*\{"
    r"|try\s*\{"
    r"|catch\s*\("
    r")"
)

API_MARKERS = (
    "Parameters:",
    "Returns:",
    "Options:",
    "Example:",
    "Logic:",
    "None.",
)
OLD_BLOCK_COMMENT = re.compile(
    r"^\s*//\s*(?:Logic:\s*)?(?:Execute:|Bind:|Return |Propagate|When |Otherwise,|Iterate:|Repeat |Dispatch |Attempt |Handle failure)"
)


def load_docs_module():


    """








    Description:








    Load docs module.

















    Inputs:








    None.

















    Outputs:








    None.

















    Example:








    result = load_docs_module()


    """
    import sys

    path = Path(__file__).with_name("add_inline_docs.py")
    name = "spanda_add_inline_docs"
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    assert spec.loader is not None
    spec.loader.exec_module(mod)
    return mod


def is_test_function(text: str, start: int) -> bool:


    """








    Description:








    Is test function.

















    Inputs:








    text: str








    Caller-supplied text.








    start: int








    Caller-supplied start.

















    Outputs:








    result: bool








    Return value from `is_test_function`.

















    Example:








    result = is_test_function(text, start)


    """
    prefix = text[max(0, start - 600) : start]
    if "#[test]" in prefix or "#[tokio::test]" in prefix:
        return True
    mod_idx = prefix.rfind("mod ")
    if mod_idx != -1 and "#[cfg(test)]" in prefix[mod_idx:]:
        return True
    return False


def body_has_nested_function(body_lines: list[str], base_indent: str) -> bool:


    """








    Description:








    Body has nested function.

















    Inputs:








    body_lines: list[str]








    Caller-supplied body lines.








    base_indent: str








    Caller-supplied base indent.

















    Outputs:








    result: bool








    Return value from `body_has_nested_function`.

















    Example:








    result = body_has_nested_function(body_lines, base_indent)


    """
    nested = re.compile(rf"^{re.escape(base_indent)}(?:pub\s+)?fn\s+")
    return any(nested.match(line) for line in body_lines)


def is_api_marker_line(line: str) -> bool:


    """








    Description:








    Is api marker line.

















    Inputs:








    line: str








    Caller-supplied line.

















    Outputs:








    result: bool








    Return value from `is_api_marker_line`.

















    Example:








    result = is_api_marker_line(line)


    """
    stripped = line.strip()
    if stripped in {"//", "// -"}:
        return True
    if stripped.startswith("// - `"):
        return True
    return any(stripped == f"// {m}" or stripped.startswith(f"// {m} ") for m in API_MARKERS)


EXAMPLE_USE_LINE = re.compile(r"^\s*// use (?:crate::|\w+::)")


def is_example_use_line(line: str) -> bool:


    """








    Description:








    Is example use line.

















    Inputs:








    line: str








    Caller-supplied line.

















    Outputs:








    result: bool








    Return value from `is_example_use_line`.

















    Example:








    result = is_example_use_line(line)


    """
    return EXAMPLE_USE_LINE.match(line) is not None


def is_generated_block_comment(line: str) -> bool:


    """








    Description:








    Is generated block comment.

















    Inputs:








    line: str








    Caller-supplied line.

















    Outputs:








    result: bool








    Return value from `is_generated_block_comment`.

















    Example:








    result = is_generated_block_comment(line)


    """
    stripped = line.strip()
    if not stripped.startswith("//"):
        return False
    if is_api_marker_line(line):
        return False
    if stripped.startswith("// let result =") or is_example_use_line(line):
        return False
    return True


def precedes_block_comment(prev_line: str) -> bool:


    """








    Description:








    Precedes block comment.

















    Inputs:








    prev_line: str








    Caller-supplied prev line.

















    Outputs:








    result: bool








    Return value from `precedes_block_comment`.

















    Example:








    result = precedes_block_comment(prev_line)


    """
    stripped = prev_line.strip()
    if not stripped.startswith("//"):
        return True
    if is_generated_block_comment(prev_line):
        return False
    return True


def is_old_block_comment(line: str) -> bool:


    """








    Description:








    Is old block comment.

















    Inputs:








    line: str








    Caller-supplied line.

















    Outputs:








    result: bool








    Return value from `is_old_block_comment`.

















    Example:








    result = is_old_block_comment(line)


    """
    stripped = line.strip()
    if not stripped.startswith("//"):
        return False
    if stripped.startswith("// Logic:"):
        return True
    return OLD_BLOCK_COMMENT.match(line) is not None


def strip_logic_header(lines: list[str]) -> None:


    """








    Description:








    Strip logic header.

















    Inputs:








    lines: list[str]








    Caller-supplied lines.

















    Outputs:








    None.

















    Example:








    result = strip_logic_header(lines)


    """
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped == "// Logic:":
            del lines[i]
            if i < len(lines) and "Explained inline" in lines[i]:
                del lines[i]
            if i < len(lines) and lines[i].strip() == "//":
                del lines[i]
            continue
        if stripped.startswith("// Explained inline with"):
            del lines[i]
            continue
        i += 1


GENERIC_BLOCK_COMMENTS = {
    "Process each item in turn.",
    "Repeat for each iteration of the loop.",
    "Take this branch when the condition holds.",
    "Otherwise, evaluate the next condition.",
    "Hold an intermediate value for the next steps.",
    "Initialize mutable state used by the following steps.",
    "Carry out the next step of this operation.",
    "Dispatch to the arm that matches the value.",
    "Choose behavior based on the variant of this value.",
    "Continue only when the optional value is present.",
    "Continue only when the operation succeeded.",
    "Handle the failure path.",
    "Handle the remaining cases.",
    "Reject or skip when the collection or string is empty.",
    "Proceed only when the value is available.",
    "Keep entries whose kind matches the filter.",
    "Store the value in the collection.",
    "Return success to the caller.",
    "Return a failure to the caller.",
    "Return the final result to the caller.",
    "Build the result using the type constructor.",
    "Construct and return the configured value.",
    "Repeat until the loop condition no longer holds.",
    "Repeat until the loop exits explicitly.",
    "Stop and propagate any error from this step.",
    "Walk the stored entries.",
    "Collect the filtered results into a list.",
    "Keep only the entries that pass the filter.",
    "Use the receiver to complete the operation.",
    "Verify the expected outcome.",
    "Normalize fields to their canonical form.",
    "Resolve the lookup and extract the needed data.",
    "Dispatch based on the switched value.",
    "Take this branch when the values match.",
    "Take this branch when the condition is false.",
    "Return the result to the caller.",
    "Signal a failure to the caller.",
    "Wait for the asynchronous operation to finish.",
    "Use the instance state to continue the operation.",
    "Store the value for later use.",
    "Report a validation or runtime error.",
    "Try the operation and handle failures below.",
    "Handle errors raised by the try block.",
    "Otherwise, continue only when the optional value is present.",
    "Otherwise, handle the successful result branch.",
    "Otherwise, handle the error branch.",
    "Parse input and stop on failure.",
    "Read from the filesystem and stop on I/O errors.",
    "Acquire the lock and stop if that fails.",
    "Register the handler through the shared registration path.",
}


def is_generic_block_comment(text: str) -> bool:


    """








    Description:








    Is generic block comment.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: bool








    Return value from `is_generic_block_comment`.

















    Example:








    result = is_generic_block_comment(text)


    """
    text = text.strip()
    if not text.endswith("."):
        text += "."
    return text in GENERIC_BLOCK_COMMENTS


def humanize(name: str) -> str:


    """








    Description:








    Humanize.

















    Inputs:








    name: str








    Caller-supplied name.

















    Outputs:








    result: str








    Return value from `humanize`.

















    Example:








    result = humanize(name)


    """
    name = name.strip("_").strip()
    if not name:
        return "value"
    return name.replace("_", " ")


def rust_tail(expr: str) -> str:


    """








    Description:








    Rust tail.

















    Inputs:








    expr: str








    Caller-supplied expr.

















    Outputs:








    result: str








    Return value from `rust_tail`.

















    Example:








    result = rust_tail(expr)


    """
    expr = expr.rstrip(" {;,").strip()
    expr = re.sub(r"^&mut\s+", "", expr)
    expr = re.sub(r"^&", "", expr)
    expr = re.sub(r"^self\.", "", expr)
    if "::" in expr:
        expr = expr.rsplit("::", 1)[-1]
    if "." in expr:
        expr = expr.rsplit(".", 1)[-1]
    expr = expr.split("(")[0].strip()
    return humanize(expr)


def describe_for_rust(s: str) -> str:


    """








    Description:








    Describe for rust.

















    Inputs:








    s: str








    Caller-supplied s.

















    Outputs:








    result: str








    Return value from `describe_for_rust`.

















    Example:








    result = describe_for_rust(s)


    """
    m = re.match(r"for\s+(.+?)\s+in\s+(.+)", s)
    if not m:
        return "Iterate over the collection."
    binding, source = m.group(1).strip(), m.group(2).rstrip(" {").strip()
    src = rust_tail(source)
    if binding.startswith("("):
        inner = binding.strip("()")
        if "param" in inner:
            return "Bind each formal parameter to its call argument."
        if "key" in inner and "value" in inner:
            return f"Walk each key/value pair in {src}."
        return f"Iterate over {src} with destructured elements."
    var = binding.strip("_")
    hints = {
        "import": "Emit codegen metadata for each import.",
        "ext": "Declare each extern function in the generated output.",
        "func": "Generate code for each module function.",
        "robot": "Handle each robot declared in the program.",
        "test": "Run each test block in program order.",
        "fault": "Inject each configured hardware fault.",
        "param": "Bind each parameter before executing the body.",
        "entry": "Process each registry entry.",
        "cap": "Validate each requested capability.",
        "dep": "Resolve each dependency specification.",
        "line": "Handle each input line.",
        "arg": "Apply each command-line argument.",
        "trigger": "Evaluate each trigger definition.",
        "handler": "Invoke each registered handler.",
        "token": "Process each lexer token.",
        "stmt": "Execute each statement in sequence.",
        "expr": "Evaluate each expression in the list.",
        "name": f"Iterate over each name in {src}.",
        "item": f"Handle each entry in {src}.",
        "ch": "Scan each character in the input.",
        "byte": "Scan each byte in the buffer.",
        "path": "Process each filesystem path.",
        "file": "Handle each file in the listing.",
        "dir": "Walk each directory entry.",
        "frame": "Replay each recorded trace frame.",
        "step": "Execute each pipeline step.",
        "node": "Visit each AST node.",
        "field": "Check each struct field.",
        "variant": "Handle each enum variant arm.",
    }
    if var in hints:
        return hints[var]
    if src.endswith("s") and len(src) > 3:
        return f"Process each {src[:-1]}."
    return f"Iterate over {src}."


def sentence(text: str) -> str:


    """








    Description:








    Sentence.

















    Inputs:








    text: str








    Caller-supplied text.

















    Outputs:








    result: str








    Return value from `sentence`.

















    Example:








    result = sentence(text)


    """
    text = text.strip()
    if not text:
        return text
    return text[0].upper() + text[1:]


def describe_if_rust(s: str, *, otherwise: bool = False) -> str:


    """








    Description:








    Describe if rust.

















    Inputs:








    s: str








    Caller-supplied s.








    *: input value








    Caller-supplied *.








    otherwise: bool








    Caller-supplied otherwise.

















    Outputs:








    result: str








    Return value from `describe_if_rust`.

















    Example:








    result = describe_if_rust(s, *, otherwise)


    """
    prefix = "Otherwise, " if otherwise else ""
    body = re.sub(r"^else\s+", "", s).strip()
    body = re.sub(r"^if\s+", "", body).rstrip(" {").strip()

    for pat in (
        r"let Some\((\w+)\)\s*=\s*(.+)",
        r"let Ok\((\w+)\)\s*=\s*(.+)",
        r"let Err\((\w+)\)\s*=\s*(.+)",
    ):
        m = re.match(pat, body)
        if m:
            var, src = m.group(1), rust_tail(m.group(2))
            if "Some" in pat:
                return sentence(f"{prefix}emit output when {src} provides a {humanize(var)}.")
            if "Ok" in pat:
                return sentence(f"{prefix}handle the success value from {src}.")
            return sentence(f"{prefix}handle the error returned from {src}.")

    if ".is_empty()" in body:
        subj = rust_tail(re.sub(r"\.is_empty\(\).*", "", body))
        return sentence(f"{prefix}skip further work when {subj} is empty.")

    if ".is_some()" in body or ".is_ok()" in body:
        subj = rust_tail(body)
        return sentence(f"{prefix}proceed only when {subj} is available.")

    if "matches!" in body:
        return sentence(f"{prefix}keep entries that match the expected pattern.")

    for key, msg in {
        "trace_scheduler": "log scheduler decisions when scheduler tracing is enabled.",
        "trace_tasks": "log task lifecycle events when task tracing is enabled.",
        "trace_triggers": "log trigger evaluation when trigger tracing is enabled.",
        "trace_events": "log event dispatch when event tracing is enabled.",
        "replay_trace": "record replay output when trace replay mode is active.",
        "is_async": "skip synchronous handling for async functions.",
        "is_dir": "treat the path as a directory and scan its contents.",
        "is_file": "continue only when the path is a regular file.",
        "success()": "handle output when the subprocess succeeds.",
        "exists()": "act only when the target path already exists.",
        "contains(": "check membership before continuing.",
    }.items():
        if key in body:
            return sentence(prefix + msg)

    if "==" in body:
        left, right = body.split("==", 1)
        return sentence(f"{prefix}take the branch when {rust_tail(left)} equals {rust_tail(right)}.")
    if "!=" in body:
        left, right = body.split("!=", 1)
        return sentence(f"{prefix}take the branch when {rust_tail(left)} differs from {rust_tail(right)}.")
    if body.startswith("!"):
        return sentence(f"{prefix}take the branch when {rust_tail(body[1:])} is false.")

    cond = body[:70].strip()
    return sentence(f"{prefix}take this path when {humanize(cond)}.")


def describe_let_rust(s: str) -> str:


    """








    Description:








    Describe let rust.

















    Inputs:








    s: str








    Caller-supplied s.

















    Outputs:








    result: str








    Return value from `describe_let_rust`.

















    Example:








    result = describe_let_rust(s)


    """
    if "Program::Program" in s or "Program {" in s:
        if "tests" in s:
            return "Extract test blocks from the parsed program."
        if "robots" in s:
            return "Extract robot declarations from the parsed program."
        return "Destructure the program into its top-level sections."
    if "clone_bindings" in s:
        return "Save current variable bindings before the call."
    if "SirProgram" in s or "sir" in s.lower():
        return "Hold the lowered SIR program for codegen."
    m = re.match(r"let mut (\w+)", s)
    if m:
        name = humanize(m.group(1))
        if m.group(1) == "out":
            return "Start the generated output buffer."
        return f"Create mutable {name} for accumulating results."
    m = re.match(r"let (\w+)", s)
    if m:
        name = m.group(1)
        hints = {
            "out": "Start building the generated output buffer.",
            "output": "Start building the generated output buffer.",
            "saved": "Preserve state that must be restored after the call.",
            "tokens": "Tokenize the source before parsing.",
            "program": "Parse and type-check the source program.",
            "manifest": "Load the package manifest from disk.",
            "child": "Spawn the subprocess for the external command.",
            "url": "Resolve the download URL for the package.",
            "path": "Resolve the filesystem path for the next step.",
            "name": "Resolve the symbol name used below.",
            "value": "Compute the value consumed by the next step.",
            "result": "Store the outcome of the preceding operation.",
            "err": "Capture the error for reporting.",
            "line": "Read the next input line.",
            "body": "Hold the function body for execution.",
        }
        if name in hints:
            return hints[name]
        return f"Compute {humanize(name)} for the following logic."
    return "Bind a local value for the next steps."


def describe_block_rust(line: str) -> str:


    """








    Description:








    Describe block rust.

















    Inputs:








    line: str








    Caller-supplied line.

















    Outputs:








    result: str








    Return value from `describe_block_rust`.

















    Example:








    result = describe_block_rust(line)


    """
    s = line.strip()
    if s.startswith("else if let TriggerKind::Event"):
        return "Otherwise, index event triggers by name for fast lookup."
    if s.startswith("if let TriggerKind::Event"):
        return "For event triggers, record the event name in the index."
    if s.startswith("else if "):
        return describe_if_rust(s, otherwise=True)
    if s.startswith("if "):
        return describe_if_rust(s)
    if s.startswith("else"):
        return "Handle any remaining cases."
    if s.startswith("match "):
        subj = s[6:].strip().rstrip("{").strip()
        if subj in ("self", "&self", "*self"):
            return "Dispatch based on the enum variant or current state."
        return f"Match on {rust_tail(subj)} and handle each case."
    if s.startswith("for "):
        return describe_for_rust(s)
    if s.startswith("while "):
        cond = s[6:].strip().rstrip(" {")
        return f"Repeat while {humanize(cond)}."
    if s.startswith("loop"):
        return "Run the loop body until it exits."
    if s.startswith("self.register("):
        arg = s.split("(", 1)[1].split(")", 1)[0].strip()
        return f"Register the {humanize(arg)} handler."
    if s.startswith("Ok("):
        return "Return the success value to the caller."
    if s.startswith("Err("):
        return "Return the error to the caller."
    if m := re.match(r"RuntimeValue::(\w+)", s):
        return f"Build a {humanize(m.group(1))} runtime value."
    if s.startswith("SpandaError::"):
        return "Construct the structured error to propagate."
    if s.startswith("use "):
        return "Import the items needed by the logic below."
    if s.endswith("?"):
        if "from_str" in s or "parse" in s:
            return "Parse input and stop on failure."
        if "read_to_string" in s or "fs::" in s:
            return "Read from the filesystem and stop on I/O errors."
        if "lock()" in s:
            return "Acquire the lock and stop if that fails."
        return f"Propagate any error from {rust_tail(s.rstrip('?'))}."
    if s.startswith("return "):
        return "Assemble the struct fields and return it."
    if s.startswith("Self::"):
        return f"Build the result via {rust_tail(s)}."
    if s.startswith("let mut ") or s.startswith("let "):
        return describe_let_rust(s)
    if s == "None":
        return "Return no value for this path."
    if s.startswith("Vec::new"):
        return "Return an empty list."
    if s.startswith("std::mem::take"):
        return "Move out the stored value and leave a default behind."
    if ".normalize()" in s:
        return f"Normalize {rust_tail(s.split('.normalize')[0])} to canonical form."
    if ".insert(" in s or ".push(" in s:
        return f"Append into {rust_tail(s.split('.')[0])}."
    if ".collect()" in s:
        return "Collect filtered entries into a new list."
    if ".filter(" in s and ".iter()" in s:
        return "Filter the iterable before continuing."
    if ".iter()" in s:
        return f"Iterate over {rust_tail(s.split('.iter()')[0])}."
    if ".and_then(" in s or ".map(" in s:
        return f"Transform {rust_tail(s.split('.')[0])} and continue the chain."
    if s.startswith("&self."):
        return f"Return {rust_tail(s)} from this handle."
    if s.startswith("self."):
        method = s.split("(")[0].split(".")[-1]
        return f"Call {humanize(method)} on the current instance."
    if s.startswith("&["):
        return "Return the static list of known values."
    if "assert!" in s or "assert_eq!" in s:
        return "Assert the expected outcome in tests."
    return f"Produce {rust_tail(s)} as the result."


def ts_tail(expr: str) -> str:


    """








    Description:








    Ts tail.

















    Inputs:








    expr: str








    Caller-supplied expr.

















    Outputs:








    result: str








    Return value from `ts_tail`.

















    Example:








    result = ts_tail(expr)


    """
    expr = expr.rstrip(" {;,").strip()
    expr = re.sub(r"^this\.", "", expr)
    if "." in expr:
        expr = expr.rsplit(".", 1)[-1]
    expr = expr.split("(")[0].strip()
    return humanize(expr)


def describe_for_ts(s: str) -> str:


    """








    Description:








    Describe for ts.

















    Inputs:








    s: str








    Caller-supplied s.

















    Outputs:








    result: str








    Return value from `describe_for_ts`.

















    Example:








    result = describe_for_ts(s)


    """
    m = re.search(r"for\s*\(\s*(?:const|let)\s+(\w+)\s+of\s+(.+?)\)", s)
    if m:
        var, src = m.group(1), ts_tail(m.group(2))
        hints = {
            "import": "Process each import declaration.",
            "imp": "Process each import declaration.",
            "stmt": "Execute each statement in sequence.",
            "token": "Process each lexer token.",
            "node": "Visit each AST node.",
            "entry": "Handle each registry entry.",
            "item": f"Handle each entry in {src}.",
            "test": "Run each test case.",
            "arg": "Apply each command-line argument.",
            "line": "Handle each input line.",
            "robot": "Set up each robot declaration.",
            "param": "Bind each parameter before continuing.",
            "handler": "Invoke each registered handler.",
        }
        if var in hints:
            return hints[var]
        if src.endswith("s") and len(src) > 3:
            return f"Process each {src[:-1]}."
        return f"Iterate over {src}."
    m = re.search(r"for\s*\(\s*(?:const|let)\s+(\w+)", s)
    if m:
        return f"Loop with index variable {humanize(m.group(1))}."
    return "Iterate over the collection."


def describe_if_ts(s: str, *, otherwise: bool = False) -> str:


    """








    Description:








    Describe if ts.

















    Inputs:








    s: str








    Caller-supplied s.








    *: input value








    Caller-supplied *.








    otherwise: bool








    Caller-supplied otherwise.

















    Outputs:








    result: str








    Return value from `describe_if_ts`.

















    Example:








    result = describe_if_ts(s, *, otherwise)


    """
    prefix = "Otherwise, " if otherwise else ""
    body = s.strip()
    if body.startswith("} else if ("):
        body = body.split("(", 1)[1]
    elif body.startswith("if ("):
        body = body.split("(", 1)[1]
    body = body.rsplit(")", 1)[0].strip()

    if "===" in body:
        left, right = body.split("===", 1)
        return f"{prefix}continue when {ts_tail(left)} equals {ts_tail(right)}."
    if "!==" in body:
        left, right = body.split("!==", 1)
        return f"{prefix}continue when {ts_tail(left)} differs from {ts_tail(right)}."
    if body.strip().startswith("!"):
        return f"{prefix}continue when {ts_tail(body.strip()[1:])} is falsy."
    if ".length === 0" in body or ".length == 0" in body:
        return f"{prefix}skip when {ts_tail(body.split('.')[0])} is empty."
    if ".includes(" in body:
        return f"{prefix}check membership before continuing."
    return f"{prefix}continue when {humanize(body[:70])}."


def describe_let_ts(s: str) -> str:


    """








    Description:








    Describe let ts.

















    Inputs:








    s: str








    Caller-supplied s.

















    Outputs:








    result: str








    Return value from `describe_let_ts`.

















    Example:








    result = describe_let_ts(s)


    """
    m = re.match(r"(?:const|let)\s+(\w+)", s)
    if m:
        name = m.group(1)
        hints = {
            "tokens": "Tokenize the source before parsing.",
            "program": "Parse the program AST for checking.",
            "result": "Store the outcome of the preceding operation.",
            "value": "Compute the value consumed by the next step.",
            "path": "Resolve the filesystem path for the next step.",
            "output": "Start building the output buffer.",
            "imported": "Track imported module names.",
            "registryExport": "Look up exports from the module registry.",
        }
        if name in hints:
            return hints[name]
        return f"Compute {humanize(name)} for the following logic."
    return "Bind a local value for the next steps."


def describe_block_ts(line: str) -> str:


    """








    Description:








    Describe block ts.

















    Inputs:








    line: str








    Caller-supplied line.

















    Outputs:








    result: str








    Return value from `describe_block_ts`.

















    Example:








    result = describe_block_ts(line)


    """
    s = line.strip()
    if "} else if (" in s or s.startswith("else if ("):
        return describe_if_ts(s, otherwise=True)
    if s.startswith("} else {") or s == "else {":
        return "Handle any remaining cases."
    if s.startswith("if ("):
        return describe_if_ts(s)
    if s.startswith("switch ("):
        subj = s.split("(", 1)[1].rsplit(")", 1)[0].strip()
        return f"Branch on {ts_tail(subj)}."
    if s.startswith("for ("):
        return describe_for_ts(s)
    if s.startswith("while ("):
        cond = s.split("(", 1)[1].rsplit(")", 1)[0].strip()
        return f"Repeat while {humanize(cond)}."
    if s.startswith("try {"):
        return "Try the operation and handle failures below."
    if s.startswith("catch ("):
        return "Handle errors raised by the try block."
    if s.startswith("return "):
        target = s[7:].strip().rstrip(";")
        return f"Return {ts_tail(target)} to the caller."
    if s.startswith("throw "):
        return f"Signal failure: {humanize(s[6:].strip())}."
    if s.startswith("const ") or s.startswith("let "):
        return describe_let_ts(s)
    if s.startswith("await "):
        return f"Await {ts_tail(s[6:].strip())}."
    if s.startswith("this."):
        method = s.split("(")[0].split(".")[-1]
        return f"Call {humanize(method)} on this instance."
    if ".push(" in s:
        return f"Append into {ts_tail(s.split('.push')[0])}."
    if ".set(" in s:
        return f"Store a value in {ts_tail(s.split('.set')[0])}."
    if "throw new" in s:
        return "Report a validation or runtime error."
    return f"Evaluate {ts_tail(s)}."


def describe_block(line: str, lang: str, is_control: bool) -> str:


    """








    Description:








    Describe block.

















    Inputs:








    line: str








    Caller-supplied line.








    lang: str








    Caller-supplied lang.








    is_control: bool








    Caller-supplied is control.

















    Outputs:








    result: str








    Return value from `describe_block`.

















    Example:








    result = describe_block(line, lang, is_control)


    """
    if is_control:
        return describe_block_rust(line) if lang == "rust" else describe_block_ts(line)
    return describe_block_rust(line) if lang == "rust" else describe_block_ts(line)


def annotate_body_lines(body_lines: list[str], base_indent: str, lang: str) -> int:


    """








    Description:








    Annotate body lines.

















    Inputs:








    body_lines: list[str]








    Caller-supplied body lines.








    base_indent: str








    Caller-supplied base indent.








    lang: str








    Caller-supplied lang.

















    Outputs:








    result: int








    Return value from `annotate_body_lines`.

















    Example:








    result = annotate_body_lines(body_lines, base_indent, lang)


    """
    strip_logic_header(body_lines)

    cleaned: list[str] = []
    past_api = False
    for line in body_lines:
        stripped = line.strip()
        if not past_api:
            if stripped in ("// Logic:",) or stripped.startswith("// Explained inline"):
                continue
            if is_old_block_comment(line):
                continue
            if stripped.startswith("//") or stripped == "":
                cleaned.append(line)
                continue
            past_api = True
        if is_old_block_comment(line):
            continue
        if past_api and stripped == "":
            continue
        if past_api and stripped.startswith("//") and not is_api_marker_line(line):
            if stripped.startswith("// let result =") or is_example_use_line(line):
                cleaned.append(line)
                continue
            if is_generic_block_comment(stripped[2:].strip()):
                continue
            continue
        cleaned.append(line)

    logic_re = RUST_LOGIC if lang == "rust" else TS_LOGIC
    out: list[str] = []
    fixes = 0
    past_api = False
    executable_seen = 0

    for line in cleaned:
        stripped = line.strip()
        if not past_api:
            if stripped.startswith("//") or stripped == "":
                out.append(line)
                continue
            past_api = True
        if stripped == "":
            continue
        if stripped.startswith("//"):
            out.append(line)
            continue

        is_control = logic_re.match(line) is not None
        needs_comment = is_control or executable_seen == 0
        if needs_comment:
            while out and out[-1].strip() == "":
                out.pop()
            j = len(out) - 1
            has_comment = j >= 0 and is_generated_block_comment(out[j])
            desc = describe_block(line, lang, is_control)
            line_indent = line[: len(line) - len(line.lstrip(" \t"))]
            if has_comment:
                prev_text = out[j].strip()[2:].strip()
                if is_generic_block_comment(prev_text) and prev_text != desc:
                    out[j] = f"{line_indent}// {desc}\n"
                    fixes += 1
            else:
                if out:
                    prev = out[-1]
                    while prev.strip() == "" and len(out) > 1:
                        prev = out[-2]
                    if precedes_block_comment(prev):
                        if not (out and out[-1].strip() == ""):
                            out.append("\n")
                out.append(f"{line_indent}// {desc}\n")
                fixes += 1
        out.append(line)
        executable_seen += 1

    body_lines[:] = out
    return fixes


def find_body_end(text: str, body_start: int) -> int | None:


    """








    Description:








    Find body end.

















    Inputs:








    text: str








    Caller-supplied text.








    body_start: int








    Caller-supplied body start.

















    Outputs:








    result: int | None








    Return value from `find_body_end`.

















    Example:








    result = find_body_end(text, body_start)


    """
    docs = load_docs_module()
    return docs.scan_balanced(text, body_start - 1, "{", "}")


def collect_functions(text: str, docs_mod, lang: str):


    """








    Description:








    Collect functions.

















    Inputs:








    text: str








    Caller-supplied text.








    docs_mod: input value








    Caller-supplied docs mod.








    lang: str








    Caller-supplied lang.

















    Outputs:








    None.

















    Example:








    result = collect_functions(text, docs_mod, lang)


    """
    if lang == "rust":
        fns = docs_mod.find_rust_functions(text)
    else:
        fns = docs_mod.find_ts_callables(text, False) + docs_mod.find_ts_callables(text, True)
        seen: set[int] = set()
        unique = []
        for fm in fns:
            if fm.body_start not in seen:
                seen.add(fm.body_start)
                unique.append(fm)
        fns = unique
    spans = []
    for fm in fns:
        if is_test_function(text, fm.start):
            continue
        end = find_body_end(text, fm.body_start)
        if end is None:
            continue
        body = text[fm.body_start:end]
        base_indent = fm.indent + ("    " if lang == "rust" else "  ")
        if body_has_nested_function(body.splitlines(keepends=True), base_indent):
            continue
        spans.append((fm.body_start, end, base_indent, len(body)))
    spans.sort(key=lambda s: s[3])
    accepted: list[tuple[int, int, str, int]] = []
    for span in spans:
        start, end, base_indent, length = span
        if any(not (end <= a[0] or start >= a[1]) for a in accepted):
            continue
        accepted.append(span)
    return accepted


def process_file(path: Path, docs_mod, lang: str) -> tuple[str, int]:


    """








    Description:








    Process file.

















    Inputs:








    path: Path








    Caller-supplied path.








    docs_mod: input value








    Caller-supplied docs mod.








    lang: str








    Caller-supplied lang.

















    Outputs:








    result: tuple[str, int]








    Return value from `process_file`.

















    Example:








    result = process_file(path, docs_mod, lang)


    """
    text = path.read_text(encoding="utf-8")
    spans = collect_functions(text, docs_mod, lang)
    updates: list[tuple[int, int, str]] = []
    total = 0
    for start, end, base_indent, _ in spans:
        body_lines = text[start:end].splitlines(keepends=True)
        total += annotate_body_lines(body_lines, base_indent, lang)
        updates.append((start, end, "".join(body_lines)))
    for start, end, new_body in sorted(updates, key=lambda u: u[0], reverse=True):
        text = text[:start] + new_body + text[end:]
    return text, total


def should_process(path: Path) -> str | None:


    """








    Description:








    Should process.

















    Inputs:








    path: Path








    Caller-supplied path.

















    Outputs:








    result: str | None








    Return value from `should_process`.

















    Example:








    result = should_process(path)


    """
    if any(p in path.parts for p in SKIP_PATH_PARTS):
        return None
    if "editor" in path.parts:
        return None
    if path.suffix == ".rs" and "crates" in path.parts:
        return "rust"
    if path.suffix == ".ts" and ("src" in path.parts or "packages" in path.parts):
        if path.name.endswith(".test.ts"):
            return None
        return "ts"
    return None


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
    docs_mod = load_docs_module()
    changed = 0
    total_fixes = 0
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file():
            continue
        lang = should_process(path)
        if lang is None:
            continue
        original = path.read_text(encoding="utf-8")
        updated, fixes = process_file(path, docs_mod, lang)
        if updated != original:
            path.write_text(updated, encoding="utf-8")
            changed += 1
            total_fixes += fixes
            print(f"updated {path.relative_to(ROOT)} ({fixes} block comments)")
    print(f"\nDone. Updated {changed} files, {total_fixes} block comments.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
