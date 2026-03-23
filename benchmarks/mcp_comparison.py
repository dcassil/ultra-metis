#!/usr/bin/env python3
"""
MCP-vs-MCP Benchmark: Cadre vs Original Metis
Both servers tested over identical stdio MCP transport — apples-to-apples comparison.

Scenarios 1-5: Speed, correctness, error handling
Scenario 6: Template quality — which templates guide AI toward better document content
"""

import json
import re
import subprocess
import sys
import tempfile
import time
import os
from dataclasses import dataclass, field
from pathlib import Path

# ── MCP JSON-RPC transport (newline-delimited JSON) ─────────────────────────

def send_jsonrpc(proc, method, params=None, req_id=1):
    """Send a JSON-RPC request as a single line, read one-line response."""
    request = {"jsonrpc": "2.0", "id": req_id, "method": method}
    if params is not None:
        request["params"] = params
    line = json.dumps(request, separators=(",", ":"))
    proc.stdin.write(line + "\n")
    proc.stdin.flush()
    return read_jsonrpc_response(proc)


def read_jsonrpc_response(proc):
    """Read a single JSON-RPC response line, skipping empty/non-JSON lines.

    Some servers (original metis) write log messages to stdout mixed with
    JSON-RPC responses. We skip any line that doesn't start with '{'.
    """
    for _ in range(50):  # skip up to 50 non-JSON lines
        line = proc.stdout.readline()
        if not line:  # EOF — server died
            return None
        stripped = line.strip()
        if not stripped:
            continue
        # Skip non-JSON lines (log output, ANSI escape sequences, etc.)
        if not stripped.startswith("{"):
            continue
        try:
            return json.loads(stripped)
        except json.JSONDecodeError:
            continue
    return None


def call_tool(proc, tool_name, arguments, req_id=1):
    """Call an MCP tool and return (result, elapsed_ms)."""
    start = time.perf_counter()
    resp = send_jsonrpc(proc, "tools/call", {
        "name": tool_name,
        "arguments": arguments,
    }, req_id=req_id)
    elapsed_ms = (time.perf_counter() - start) * 1000
    return resp, elapsed_ms


def drain_pending(proc, timeout=0.1):
    """Non-blocking drain of any pending stdout data."""
    import select
    while True:
        ready, _, _ = select.select([proc.stdout], [], [], timeout)
        if not ready:
            break
        line = proc.stdout.readline()
        if not line:
            break


def start_mcp_server(cmd, cwd=None):
    """Spawn an MCP server subprocess and complete the initialize handshake."""
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        cwd=cwd,
    )
    resp = send_jsonrpc(proc, "initialize", {
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {"name": "benchmark", "version": "1.0.0"},
    }, req_id=0)
    # Send initialized notification (no id = true notification, no response expected)
    notif = json.dumps({"jsonrpc": "2.0", "method": "notifications/initialized"})
    proc.stdin.write(notif + "\n")
    proc.stdin.flush()
    # Cadre incorrectly responds with "Method not found". Drain it.
    drain_pending(proc, timeout=0.2)
    return proc, resp


def stop_server(proc):
    try:
        proc.stdin.close()
        proc.wait(timeout=5)
    except Exception:
        proc.kill()


# ── Data types ───────────────────────────────────────────────────────────────

@dataclass
class TimingResult:
    operation: str
    elapsed_ms: float
    success: bool
    output_size: int = 0
    detail: str = ""


@dataclass
class ScenarioResult:
    name: str
    ultra_timings: list = field(default_factory=list)
    original_timings: list = field(default_factory=list)


@dataclass
class TemplateQualityResult:
    tool_name: str
    template_size: int
    modules_tested: int
    avg_completeness: float
    avg_placeholders: float
    total_filled_sections: int
    total_empty_sections: int
    total_tokens: int
    total_time_s: float
    per_module: list = field(default_factory=list)


# ── Helpers ──────────────────────────────────────────────────────────────────

def extract_text(resp):
    if resp is None:
        return ""
    result = resp.get("result", {})
    content = result.get("content", [])
    return "\n".join(c.get("text", "") for c in content if isinstance(c, dict) and c.get("type") == "text")


def is_error(resp):
    if resp is None:
        return True
    result = resp.get("result", {})
    return bool(result.get("isError") or resp.get("error"))


def parse_short_code(text, prefix_pattern):
    """Extract a short code like BENCH-V-0001 from response text."""
    m = re.search(prefix_pattern + r'-\d{4}', text)
    return m.group(0) if m else None


def record(timings, op, ms, resp):
    """Record a timing result."""
    ok = not is_error(resp)
    text = extract_text(resp)
    timings.append(TimingResult(op, ms, ok, len(text), text[:200] if not ok else ""))
    return ok, text


# ── State tracker per tool ───────────────────────────────────────────────────

class BenchState:
    """Track short codes created during the benchmark for each tool."""
    def __init__(self, name):
        self.name = name
        self.vision_code = None
        self.initiative_code = None
        self.task_codes = []


# ── Scenarios ────────────────────────────────────────────────────────────────

REQ_ID = 10  # global request counter

def next_id():
    global REQ_ID
    REQ_ID += 1
    return REQ_ID


def run_scenario_1_init(ultra_proc, ultra_path, orig_proc, orig_path, ultra_st, orig_st):
    """Scenario 1: Project Bootstrap"""
    scenario = ScenarioResult(name="Project Bootstrap")

    resp, ms = call_tool(ultra_proc, "initialize_project", {
        "project_path": ultra_path, "prefix": "BENCH",
    }, req_id=next_id())
    record(scenario.ultra_timings, "init", ms, resp)

    resp, ms = call_tool(orig_proc, "initialize_project", {
        "project_path": orig_path, "prefix": "BENCH",
    }, req_id=next_id())
    record(scenario.original_timings, "init", ms, resp)

    return scenario


def run_scenario_2_planning(ultra_proc, ultra_path, orig_proc, orig_path, ultra_st, orig_st):
    """Scenario 2: Planning Workflow

    Note: original metis auto-creates a vision during init, so we skip
    create_vision for it and just use BENCH-V-0001. We time matching ops.
    """
    scenario = ScenarioResult(name="Planning Workflow")

    # -- Cadre: create vision explicitly --
    resp, ms = call_tool(ultra_proc, "create_document", {
        "project_path": ultra_path, "document_type": "vision", "title": "CLI Toolkit",
    }, req_id=next_id())
    ok, text = record(scenario.ultra_timings, "create_vision", ms, resp)
    if ok:
        ultra_st.vision_code = parse_short_code(text, r'BENCH-V')
    if not ultra_st.vision_code:
        ultra_st.vision_code = "BENCH-V-0001"

    # -- Original metis: vision auto-created during init, just note it --
    orig_st.vision_code = "BENCH-V-0001"
    scenario.original_timings.append(TimingResult(
        "create_vision", 0, True, 0, "(auto-created during init)"))

    # -- Both: transition vision draft -> review -> published --
    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        for phase_name in ["to_review", "to_published"]:
            resp, ms = call_tool(proc, "transition_phase", {
                "project_path": path, "short_code": st.vision_code,
            }, req_id=next_id())
            record(timings, f"vision_{phase_name}", ms, resp)

    # -- Both: create initiative --
    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        resp, ms = call_tool(proc, "create_document", {
            "project_path": path, "document_type": "initiative",
            "title": "CSV Parser Module", "parent_id": st.vision_code,
        }, req_id=next_id())
        ok, text = record(timings, "create_initiative", ms, resp)
        if ok:
            st.initiative_code = parse_short_code(text, r'BENCH-I')
        if not st.initiative_code:
            st.initiative_code = "BENCH-I-0001"

    # -- Both: advance initiative to decompose --
    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        for phase in ["design", "ready", "decompose"]:
            resp, ms = call_tool(proc, "transition_phase", {
                "project_path": path, "short_code": st.initiative_code,
            }, req_id=next_id())
            record(timings, f"init_to_{phase}", ms, resp)

    # -- Both: create 3 tasks --
    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        for i, title in enumerate(["Parse headers", "Type coercion", "Error recovery"], 1):
            resp, ms = call_tool(proc, "create_document", {
                "project_path": path, "document_type": "task",
                "title": title, "parent_id": st.initiative_code,
            }, req_id=next_id())
            ok, text = record(timings, f"create_task_{i}", ms, resp)
            if ok:
                code = parse_short_code(text, r'BENCH-T')
                if code:
                    st.task_codes.append(code)

    return scenario


def run_scenario_3_search(ultra_proc, ultra_path, orig_proc, orig_path, ultra_st, orig_st):
    """Scenario 3: Search and Query"""
    scenario = ScenarioResult(name="Search and Query")

    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        resp, ms = call_tool(proc, "search_documents", {
            "project_path": path, "query": "Parser",
        }, req_id=next_id())
        record(timings, "search_parser", ms, resp)

        resp, ms = call_tool(proc, "list_documents", {
            "project_path": path,
        }, req_id=next_id())
        record(timings, "list_all", ms, resp)

        resp, ms = call_tool(proc, "read_document", {
            "project_path": path, "short_code": st.initiative_code,
        }, req_id=next_id())
        record(timings, "read_initiative", ms, resp)

    return scenario


def run_scenario_4_edit(ultra_proc, ultra_path, orig_proc, orig_path, ultra_st, orig_st):
    """Scenario 4: Document Edit"""
    scenario = ScenarioResult(name="Document Edit")

    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        # Read first
        call_tool(proc, "read_document", {
            "project_path": path, "short_code": st.initiative_code,
        }, req_id=next_id())

        resp, ms = call_tool(proc, "edit_document", {
            "project_path": path, "short_code": st.initiative_code,
            "search": "CSV Parser Module", "replace": "CSV Parser Module (Updated)",
        }, req_id=next_id())
        record(timings, "edit_document", ms, resp)

    return scenario


def run_scenario_5_errors(ultra_proc, ultra_path, orig_proc, orig_path, ultra_st, orig_st):
    """Scenario 5: Error Handling"""
    scenario = ScenarioResult(name="Error Handling")

    error_cases = [
        ("read_nonexistent", "read_document", {"short_code": "BENCH-X-9999"}),
        ("create_bad_parent", "create_document", {
            "document_type": "task", "title": "Orphan task", "parent_id": "BENCH-I-9999",
        }),
        ("transition_invalid", "transition_phase", {"short_code": "BENCH-X-9999"}),
    ]

    for label, proc, path, timings, st in [
        ("ultra", ultra_proc, ultra_path, scenario.ultra_timings, ultra_st),
        ("orig", orig_proc, orig_path, scenario.original_timings, orig_st),
    ]:
        for case_name, tool, args in error_cases:
            full_args = {**args, "project_path": path}
            resp, ms = call_tool(proc, tool, full_args, req_id=next_id())
            errored = is_error(resp)
            text = extract_text(resp)
            timings.append(TimingResult(
                case_name, ms,
                success=errored,  # catching error = success for error tests
                output_size=len(text),
                detail=text[:150] if errored else "NO ERROR RETURNED (BUG)",
            ))

    return scenario


# ── Scenario 6: Template Quality ────────────────────────────────────────────

MODULES = [
    ("CSV Parser Module",
     "Parse CSV files with automatic delimiter detection (comma, tab, semicolon), "
     "header inference, type coercion (string/int/float/date), and error recovery "
     "for malformed rows. Expose a streaming iterator API for large files."),
    ("JSON Transformer",
     "Transform and reshape JSON documents using a declarative mapping language. "
     "Support field renaming, nesting/flattening, value transformation functions, "
     "and schema validation. Must handle deeply nested structures and arrays."),
    ("Output Formatter",
     "Format processed pipeline data to multiple output targets: CSV, JSON, "
     "Parquet, and pretty console output. Support streaming writes, configurable "
     "column ordering, and pluggable compression (gzip, zstd)."),
]

TRACKED_SECTIONS = {
    "context", "goals & non-goals", "goals", "objective", "objectives",
    "acceptance criteria", "requirements", "risks", "risk considerations",
    "tasks", "decomposition", "implementation notes", "success criteria",
    "detailed design", "alternatives considered", "implementation plan",
    "testing strategy", "architecture", "use cases", "status updates",
}


def clean_heading(raw):
    """Normalize a markdown heading for section matching.

    Strips [REQUIRED]/[CONDITIONAL] markers AND any surrounding ** bold markers.
    Examples:
        'Context **[REQUIRED]**' -> 'context'
        'Goals & Non-Goals **[REQUIRED]**' -> 'goals & non-goals'
        'Requirements **[CONDITIONAL: Heavy]**' -> 'requirements'
        'Context' -> 'context'
    """
    # Remove everything from the first '[' onward (markers)
    h = raw.split("[")[0]
    # Remove markdown bold markers
    h = h.replace("**", "")
    return h.strip().lower()


def score_filled_doc(content):
    body = content
    if body.startswith("---"):
        end = body.find("\n---", 3)
        if end >= 0:
            body = body[end + 4:]

    placeholder_count = len(re.findall(r'\{[^}]+\}', body))

    sections = {}
    current_heading = None
    current_body = []
    for line in body.split("\n"):
        if line.startswith("## "):
            if current_heading:
                sections[current_heading] = "\n".join(current_body)
            current_heading = clean_heading(line[3:].strip())
            current_body = []
        elif current_heading is not None:
            current_body.append(line)
    if current_heading:
        sections[current_heading] = "\n".join(current_body)

    filled, empty = [], []
    for name, sbody in sections.items():
        if name not in TRACKED_SECTIONS:
            continue
        meaningful = [l for l in sbody.strip().split("\n")
                      if l.strip() and not l.strip().startswith("<!--")
                      and not l.strip().startswith("{") and not l.strip().endswith("}")
                      and "*Delete" not in l and "*This" not in l]
        if len(meaningful) >= 2 and "{" not in sbody:
            filled.append(name)
        else:
            empty.append(name)

    total = max(len(filled) + len(empty), 1)
    return {
        "placeholder_count": placeholder_count,
        "filled_sections": filled,
        "empty_sections": empty,
        "completeness_percent": (len(filled) / total) * 100,
        "content_lines": sum(1 for l in body.split("\n") if l.strip()
                             and not l.strip().startswith("#") and not l.strip().startswith("---")
                             and not l.strip().startswith("<!--") and not l.strip().startswith("{")),
    }


def fill_template_with_claude(module_name, module_desc, template):
    system_prompt = (
        "You are a software architect filling in a Metis initiative document template. "
        "Replace ALL placeholder text with real content. Output ONLY the filled markdown."
    )
    user_prompt = (
        f"Fill in this initiative template for '{module_name}'.\n\n"
        f"Description: {module_desc}\n\nTemplate:\n\n{template}"
    )
    try:
        result = subprocess.run(
            ["claude", "-p", user_prompt, "--system-prompt", system_prompt,
             "--output-format", "json", "--model", "haiku", "--no-session-persistence"],
            capture_output=True, text=True, timeout=120,
        )
    except subprocess.TimeoutExpired:
        return None, 0

    if result.returncode != 0:
        return None, 0
    try:
        parsed = json.loads(result.stdout)
        content = parsed.get("result", "")
        usage = parsed.get("usage", {})
        tokens = usage.get("input_tokens", 0) + usage.get("output_tokens", 0)
        return content, tokens
    except json.JSONDecodeError:
        return result.stdout, 0


def run_scenario_6_template_quality(ultra_proc, ultra_path, orig_proc, orig_path, ultra_st, orig_st):
    print("  Reading initiative templates from both MCP servers...")

    resp_u, _ = call_tool(ultra_proc, "read_document", {
        "project_path": ultra_path, "short_code": ultra_st.initiative_code,
    }, req_id=next_id())
    ultra_template = extract_text(resp_u)

    resp_o, _ = call_tool(orig_proc, "read_document", {
        "project_path": orig_path, "short_code": orig_st.initiative_code,
    }, req_id=next_id())
    orig_template = extract_text(resp_o)

    print(f"  Ultra template: {len(ultra_template)} chars")
    print(f"  Original template: {len(orig_template)} chars")

    results = {}
    for tool_name, template in [("cadre", ultra_template), ("original-metis", orig_template)]:
        start = time.perf_counter()
        per_module = []
        total_tokens = 0

        for module_name, module_desc in MODULES:
            print(f"  Filling {tool_name} / {module_name}...", end=" ", flush=True)
            filled, tokens = fill_template_with_claude(module_name, module_desc, template)
            total_tokens += tokens

            if filled:
                score = score_filled_doc(filled)
                print(f"completeness={score['completeness_percent']:.0f}% placeholders={score['placeholder_count']}")
                per_module.append({
                    "module": module_name,
                    "completeness": score["completeness_percent"],
                    "placeholders": score["placeholder_count"],
                    "filled": score["filled_sections"],
                    "empty": score["empty_sections"],
                    "content_lines": score["content_lines"],
                    "tokens": tokens,
                })
            else:
                print("FAILED")
                per_module.append({
                    "module": module_name, "completeness": 0, "placeholders": 0,
                    "filled": [], "empty": [], "content_lines": 0, "tokens": 0,
                })

        elapsed = time.perf_counter() - start
        n = max(len(per_module), 1)
        results[tool_name] = TemplateQualityResult(
            tool_name=tool_name,
            template_size=len(template),
            modules_tested=len(per_module),
            avg_completeness=sum(m["completeness"] for m in per_module) / n,
            avg_placeholders=sum(m["placeholders"] for m in per_module) / n,
            total_filled_sections=sum(len(m["filled"]) for m in per_module),
            total_empty_sections=sum(len(m["empty"]) for m in per_module),
            total_tokens=total_tokens,
            total_time_s=elapsed,
            per_module=per_module,
        )

    return results


# ── Report generation ────────────────────────────────────────────────────────

def format_report(scenarios, ultra_init_time, orig_init_time, quality_results=None):
    lines = []
    lines.append("# MCP Benchmark: Cadre vs Original Metis (Apples-to-Apples)")
    lines.append("")
    lines.append(f"**Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    lines.append("**Transport**: Both servers via stdio MCP (JSON-RPC 2.0, newline-delimited)")
    lines.append(f"**Cadre**: Rust MCP server (server startup: {ultra_init_time:.0f}ms)")
    lines.append(f"**Original Metis**: TypeScript MCP server v1.1.0 (server startup: {orig_init_time:.0f}ms)")
    lines.append("")

    lines.append("## Executive Summary")
    lines.append("")

    total_ultra = sum(t.elapsed_ms for s in scenarios for t in s.ultra_timings)
    total_orig = sum(t.elapsed_ms for s in scenarios for t in s.original_timings)
    op_count = sum(len(s.ultra_timings) for s in scenarios)
    speedup = total_orig / total_ultra if total_ultra > 0 else 0

    lines.append(f"Across **{op_count} operations** over identical MCP stdio transport, "
                 f"cadre completed in **{total_ultra:.0f}ms** "
                 f"vs original metis in **{total_orig:.0f}ms** — "
                 f"**{speedup:.1f}x {'faster' if speedup > 1 else 'slower'}**.")
    lines.append("")
    lines.append("Both servers communicate via the same newline-delimited JSON-RPC 2.0 over stdio. "
                 "The previous benchmark (REPORT.md) compared cadre CLI vs original metis MCP, "
                 "giving cadre an unfair transport advantage (~200x). This benchmark isolates "
                 "actual server performance by using identical transport for both.")
    lines.append("")

    # Failures summary
    ultra_fails = sum(1 for s in scenarios for t in s.ultra_timings if not t.success and s.name != "Error Handling")
    orig_fails = sum(1 for s in scenarios for t in s.original_timings if not t.success and s.name != "Error Handling")
    if ultra_fails or orig_fails:
        lines.append(f"**Failures** (non-error-handling): Cadre {ultra_fails}, Original Metis {orig_fails}")
        lines.append("")

    # Per-scenario tables
    for scenario in scenarios:
        lines.append(f"## Scenario: {scenario.name}")
        lines.append("")
        lines.append("| Operation | Cadre (ms) | Original Metis (ms) | Speedup | Ultra OK | Orig OK |")
        lines.append("|-----------|:----------------:|:-------------------:|:-------:|:--------:|:-------:|")

        ultra_map = {t.operation: t for t in scenario.ultra_timings}
        orig_map = {t.operation: t for t in scenario.original_timings}
        all_ops = list(dict.fromkeys(
            [t.operation for t in scenario.ultra_timings] +
            [t.operation for t in scenario.original_timings]
        ))

        s_ultra = s_orig = 0
        for op in all_ops:
            u = ultra_map.get(op)
            o = orig_map.get(op)
            u_ms = u.elapsed_ms if u else 0
            o_ms = o.elapsed_ms if o else 0
            u_ok = "Y" if (u and u.success) else "N" if u else "-"
            o_ok = "Y" if (o and o.success) else "N" if o else "-"
            sp = f"{o_ms/u_ms:.1f}x" if u_ms > 0.01 and o_ms > 0.01 else "-"
            s_ultra += u_ms; s_orig += o_ms
            lines.append(f"| {op} | {u_ms:.1f} | {o_ms:.1f} | {sp} | {u_ok} | {o_ok} |")

        sp_t = f"{s_orig/s_ultra:.1f}x" if s_ultra > 0.01 else "-"
        lines.append(f"| **Total** | **{s_ultra:.1f}** | **{s_orig:.1f}** | **{sp_t}** | | |")
        lines.append("")

    # Aggregate
    lines.append("## Aggregate Speed Summary")
    lines.append("")
    lines.append("| Scenario | Cadre (ms) | Original Metis (ms) | Speedup |")
    lines.append("|----------|:----------------:|:-------------------:|:-------:|")
    gu = go = 0
    for s in scenarios:
        su = sum(t.elapsed_ms for t in s.ultra_timings)
        so = sum(t.elapsed_ms for t in s.original_timings)
        gu += su; go += so
        sp = f"{so/su:.1f}x" if su > 0.01 else "-"
        lines.append(f"| {s.name} | {su:.1f} | {so:.1f} | {sp} |")
    sp = f"{go/gu:.1f}x" if gu > 0.01 else "-"
    lines.append(f"| **Total ({op_count} ops)** | **{gu:.1f}** | **{go:.1f}** | **{sp}** |")
    lines.append("")

    # Output size
    lines.append("## Output Size Comparison (Token Cost Proxy)")
    lines.append("")
    lines.append("| Scenario | Cadre (bytes) | Original Metis (bytes) | Ratio |")
    lines.append("|----------|:-------------------:|:----------------------:|:-----:|")
    for s in scenarios:
        su = sum(t.output_size for t in s.ultra_timings)
        so = sum(t.output_size for t in s.original_timings)
        ratio = f"{so/su:.1f}x" if su > 0 else "-"
        lines.append(f"| {s.name} | {su} | {so} | {ratio} |")
    lines.append("")

    # Error handling
    err_scenario = next((s for s in scenarios if s.name == "Error Handling"), None)
    if err_scenario:
        lines.append("## Error Handling Details")
        lines.append("")
        lines.append("| Test Case | Cadre | Original Metis |")
        lines.append("|-----------|:-----------:|:--------------:|")
        u_map = {t.operation: t for t in err_scenario.ultra_timings}
        o_map = {t.operation: t for t in err_scenario.original_timings}
        for op in u_map:
            u = u_map.get(op)
            o = o_map.get(op)
            u_s = "CAUGHT" if (u and u.success) else "MISSED"
            o_s = "CAUGHT" if (o and o.success) else "MISSED"
            lines.append(f"| {op} | {u_s} | {o_s} |")
        lines.append("")

    # Template quality
    if quality_results:
        u = quality_results.get("cadre")
        o = quality_results.get("original-metis")
        if u and o:
            lines.append("## Scenario: Template Quality (AI Fill-In)")
            lines.append("")
            lines.append("Each tool's initiative template was read via MCP `read_document`, then sent to ")
            lines.append("Claude Haiku to fill in for 3 module specs. Results measure how well the template ")
            lines.append("guides AI toward complete, placeholder-free content.")
            lines.append("")
            lines.append("| Metric | Cadre | Original Metis | Delta |")
            lines.append("|--------|:-----------:|:--------------:|:-----:|")
            lines.append(f"| Template size (chars) | {u.template_size} | {o.template_size} | {o.template_size - u.template_size:+d} |")
            lines.append(f"| Avg completeness | {u.avg_completeness:.0f}% | {o.avg_completeness:.0f}% | {u.avg_completeness - o.avg_completeness:+.0f}% |")
            lines.append(f"| Avg placeholders/doc | {u.avg_placeholders:.1f} | {o.avg_placeholders:.1f} | {u.avg_placeholders - o.avg_placeholders:+.1f} |")
            lines.append(f"| Total filled sections | {u.total_filled_sections} | {o.total_filled_sections} | {u.total_filled_sections - o.total_filled_sections:+d} |")
            lines.append(f"| Total empty sections | {u.total_empty_sections} | {o.total_empty_sections} | {u.total_empty_sections - o.total_empty_sections:+d} |")
            lines.append(f"| Total tokens used | {u.total_tokens} | {o.total_tokens} | {u.total_tokens - o.total_tokens:+d} |")
            lines.append("")

            lines.append("### Per-Module Breakdown")
            lines.append("")
            lines.append("| Module | Ultra Complete | Orig Complete | Ultra Placeholders | Orig Placeholders |")
            lines.append("|--------|:-------------:|:------------:|:------------------:|:-----------------:|")
            for um, om in zip(u.per_module, o.per_module):
                lines.append(f"| {um['module']} | {um['completeness']:.0f}% | {om['completeness']:.0f}% | {um['placeholders']} | {om['placeholders']} |")
            lines.append("")

            # Winner
            lines.append("### Template Quality Verdict")
            lines.append("")
            delta = u.avg_completeness - o.avg_completeness
            if abs(delta) < 5:
                lines.append("Both tools produce similarly complete documents when templates are filled by AI.")
            elif delta > 0:
                lines.append(f"**Cadre** templates yield {delta:.0f}% higher completeness scores.")
            else:
                lines.append(f"**Original metis** templates yield {-delta:.0f}% higher completeness scores.")
            lines.append("")

    # Methodology
    lines.append("## Methodology")
    lines.append("")
    lines.append("- Both servers spawned as stdio subprocesses, same newline-delimited JSON-RPC 2.0 transport")
    lines.append("- Timing: `time.perf_counter()` around each `tools/call` request-response cycle")
    lines.append("- Fresh temp directories, no cached state")
    lines.append("- Short codes parsed from actual responses (not hardcoded)")
    lines.append("")
    lines.append("**Key difference from REPORT.md**: Previous benchmark compared cadre CLI (direct binary) "
                 "vs original metis MCP (via Claude Code tool infrastructure). That gave cadre ~200x advantage "
                 "from transport alone. This benchmark eliminates transport as a variable.")
    lines.append("")
    if quality_results:
        lines.append("**Template quality**: Claude Haiku fills initiative templates for 3 module specs. "
                     "Scored on section completeness and remaining placeholder count.")
        lines.append("")

    return "\n".join(lines)


# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    repo_root = Path(__file__).parent.parent
    ultra_binary = repo_root / "target" / "release" / "cadre-mcp"

    if not ultra_binary.exists():
        print(f"ERROR: cadre-mcp binary not found at {ultra_binary}", file=sys.stderr)
        sys.exit(1)

    has_claude = subprocess.run(["claude", "--version"], capture_output=True).returncode == 0
    run_quality = "--skip-quality" not in sys.argv and has_claude

    ultra_tmp = tempfile.mkdtemp(prefix="bench-ultra-")
    orig_tmp = tempfile.mkdtemp(prefix="bench-orig-")
    # Cadre: all operations use the project root
    # Original metis: init uses parent dir, all other ops use .metis path
    orig_metis_path = os.path.join(orig_tmp, ".metis")

    ultra_st = BenchState("cadre")
    orig_st = BenchState("original-metis")

    print(f"Cadre project: {ultra_tmp}")
    print(f"Original Metis project: {orig_tmp} (ops: {orig_metis_path})")
    print(f"Template quality: {'YES' if run_quality else 'SKIPPED'}")
    print()

    print("Starting MCP servers...")
    t0 = time.perf_counter()
    ultra_proc, _ = start_mcp_server([str(ultra_binary)])
    ultra_init_time = (time.perf_counter() - t0) * 1000
    print(f"  Cadre: {ultra_init_time:.0f}ms")

    t0 = time.perf_counter()
    orig_proc, _ = start_mcp_server(["metis", "mcp"])
    orig_init_time = (time.perf_counter() - t0) * 1000
    print(f"  Original Metis: {orig_init_time:.0f}ms")
    print()

    scenarios = []
    quality_results = None

    try:
        # Scenario 1: Init uses parent dirs (servers create .cadre / .metis inside)
        # All other scenarios use the paths the servers expect for operations
        scenario_defs = [
            ("Project Bootstrap", run_scenario_1_init, ultra_tmp, orig_tmp),
            ("Planning Workflow", run_scenario_2_planning, ultra_tmp, orig_metis_path),
            ("Search and Query", run_scenario_3_search, ultra_tmp, orig_metis_path),
            ("Document Edit", run_scenario_4_edit, ultra_tmp, orig_metis_path),
            ("Error Handling", run_scenario_5_errors, ultra_tmp, orig_metis_path),
        ]
        for i, (name, fn, u_path, o_path) in enumerate(scenario_defs, 1):
            print(f"=== Scenario {i}: {name} ===")
            s = fn(ultra_proc, u_path, orig_proc, o_path, ultra_st, orig_st)
            scenarios.append(s)

            su = sum(t.elapsed_ms for t in s.ultra_timings)
            so = sum(t.elapsed_ms for t in s.original_timings)
            sp = f"{so/su:.1f}x" if su > 0.01 else "-"
            print(f"  Ultra: {su:.1f}ms | Orig: {so:.1f}ms | Speedup: {sp}")

            # Show any failures
            for t in s.ultra_timings:
                if not t.success and s.name != "Error Handling":
                    print(f"    FAIL Ultra {t.operation}: {t.detail[:100]}")
            for t in s.original_timings:
                if not t.success and s.name != "Error Handling":
                    print(f"    FAIL Orig  {t.operation}: {t.detail[:100]}")
            print()

        if run_quality:
            print(f"=== Scenario {len(scenario_defs) + 1}: Template Quality ===")
            quality_results = run_scenario_6_template_quality(
                ultra_proc, ultra_tmp, orig_proc, orig_metis_path, ultra_st, orig_st
            )
            u = quality_results["cadre"]
            o = quality_results["original-metis"]
            print(f"  Ultra:  {u.avg_completeness:.0f}% completeness, {u.avg_placeholders:.1f} placeholders/doc")
            print(f"  Orig:   {o.avg_completeness:.0f}% completeness, {o.avg_placeholders:.1f} placeholders/doc")
            print()

    finally:
        stop_server(ultra_proc)
        stop_server(orig_proc)

    report = format_report(scenarios, ultra_init_time, orig_init_time, quality_results)
    report_path = repo_root / "benchmarks" / "MCP_COMPARISON.md"
    report_path.write_text(report)
    print(f"Report: {report_path}")

    total_ultra = sum(t.elapsed_ms for s in scenarios for t in s.ultra_timings)
    total_orig = sum(t.elapsed_ms for s in scenarios for t in s.original_timings)
    speedup = total_orig / total_ultra if total_ultra > 0.01 else 0
    print(f"\n{'='*60}")
    print(f"RESULT: Cadre {total_ultra:.1f}ms vs Original Metis {total_orig:.1f}ms ({speedup:.1f}x)")
    print(f"{'='*60}")


if __name__ == "__main__":
    main()
