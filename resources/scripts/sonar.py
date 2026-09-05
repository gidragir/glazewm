#!/usr/bin/env python3
"""
SonarQube / SonarCloud CLI & MCP Output Parser for GlazeWM.

Zero-dependency script (standard library only) to query SonarCloud API
and format/filter SonarQube issues or parse raw MCP tool JSON outputs.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
DEFAULT_HOST = "https://sonarcloud.io"


def load_project_properties() -> dict[str, str]:
    """Load configuration from sonar-project.properties if present."""
    props: dict[str, str] = {}
    props_path = REPO_ROOT / "sonar-project.properties"
    if props_path.is_file():
        for line in props_path.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, val = line.split("=", 1)
                props[key.strip()] = val.strip()
    return props


def get_mcp_config() -> dict[str, str]:
    """Retrieve Sonar configuration from global MCP config if available."""
    config_path = Path.home() / ".gemini" / "config" / "mcp_config.json"
    if config_path.is_file():
        try:
            with open(config_path, encoding="utf-8") as f:
                data = json.load(f)
            sonar_env = data.get("mcpServers", {}).get("sonarqube", {}).get("env", {})
            return {
                "host": sonar_env.get("SONAR_HOST_URL", ""),
                "token": sonar_env.get("SONAR_TOKEN", ""),
                "project_key": sonar_env.get("SONAR_PROJECT_KEY", ""),
            }
        except Exception:
            pass
    return {}


def resolve_auth_and_endpoint(args: argparse.Namespace) -> tuple[str, str, str, str]:
    """Resolve (host_url, token, project_key, organization)."""
    props = load_project_properties()
    mcp_env = get_mcp_config()

    host = (
        getattr(args, "host", None)
        or os.environ.get("SONAR_HOST_URL")
        or mcp_env.get("host")
        or DEFAULT_HOST
    ).rstrip("/")

    token = (
        getattr(args, "token", None)
        or os.environ.get("SONAR_TOKEN")
        or mcp_env.get("token")
        or ""
    )

    project_key = (
        getattr(args, "project", None)
        or os.environ.get("SONAR_PROJECT_KEY")
        or mcp_env.get("project_key")
        or props.get("sonar.projectKey", "gidragir_glazewm")
    )

    org = (
        getattr(args, "org", None)
        or os.environ.get("SONAR_ORGANIZATION")
        or props.get("sonar.organization", "")
    )

    return host, token, project_key, org


def api_request(url: str, token: str) -> dict[str, Any]:
    """Perform an authenticated GET request to SonarQube API."""
    req = urllib.request.Request(url)
    if token:
        # SonarQube HTTP Basic Auth uses token as username, password empty
        auth_bytes = f"{token}:".encode("utf-8")
        req.add_header("Authorization", f"Basic {base64.b64encode(auth_bytes).decode('ascii')}")
    req.add_header("Accept", "application/json")

    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as err:
        body = err.read().decode("utf-8", errors="replace")
        sys.exit(f"Error {err.code} from SonarQube API: {body}")
    except Exception as err:
        sys.exit(f"Failed to connect to SonarQube ({url}): {err}")


def cmd_status(args: argparse.Namespace) -> None:
    """Check quality gate status."""
    host, token, project_key, _ = resolve_auth_and_endpoint(args)
    url = f"{host}/api/qualitygates/project_status?projectKey={urllib.parse.quote(project_key)}"
    data = api_request(url, token)
    project_status = data.get("projectStatus", {})

    status = project_status.get("status", "UNKNOWN")
    print(f"Project:      {project_key}")
    print(f"Quality Gate: {status}")

    conditions = project_status.get("conditions", [])
    if conditions:
        print("\nConditions:")
        for cond in conditions:
            c_status = cond.get("status")
            metric = cond.get("metricKey")
            comparator = cond.get("comparator", "")
            error_thresh = cond.get("errorThreshold", "")
            actual = cond.get("actualValue", "")
            symbol = "✓" if c_status == "OK" else "✗"
            print(f"  [{symbol}] {metric}: {actual} (threshold: {comparator} {error_thresh}) -> {c_status}")


def fetch_open_issues(
    host: str,
    token: str,
    project_key: str,
    severity: str | None = None,
    rule: str | None = None,
    issue_type: str | None = None,
    page_size: int = 500,
) -> list[dict[str, Any]]:
    """Fetch all open/unresolved issues."""
    params: dict[str, str] = {
        "projectKeys": project_key,
        "resolved": "false",
        "ps": str(page_size),
    }
    if severity:
        params["severities"] = severity.upper()
    if rule:
        params["rules"] = rule
    if issue_type:
        params["types"] = issue_type.upper()

    query = urllib.parse.urlencode(params)
    url = f"{host}/api/issues/search?{query}"
    data = api_request(url, token)
    return data.get("issues", [])


def format_issues(issues: list[dict[str, Any]], limit: int | None = None) -> None:
    """Print issues formatted in a concise table."""
    total = len(issues)
    display_issues = issues[:limit] if limit else issues

    print(f"Total Open Issues: {total}")
    if limit and limit < total:
        print(f"Showing first {limit} issues:")
    print("-" * 100)

    for idx, issue in enumerate(display_issues, 1):
        rule = issue.get("rule", "")
        severity = issue.get("severity", "")
        itype = issue.get("type", "")
        component = issue.get("component", "")
        file_path = component.split(":", 1)[-1] if ":" in component else component
        line = issue.get("line") or issue.get("textRange", {}).get("startLine", 1)
        msg = issue.get("message", "").strip()

        print(f"{idx:2d}. [{severity:8s}] [{itype:13s}] {rule}")
        print(f"    Location: {file_path}:{line}")
        print(f"    Message:  {msg}")
        print()


def cmd_issues(args: argparse.Namespace) -> None:
    """List open issues."""
    host, token, project_key, _ = resolve_auth_and_endpoint(args)
    issues = fetch_open_issues(
        host, token, project_key,
        severity=args.severity,
        rule=args.rule,
        issue_type=args.type,
        page_size=args.limit or 500,
    )
    if args.file:
        issues = [i for i in issues if args.file in i.get("component", "")]

    if args.json:
        print(json.dumps(issues, indent=2))
    else:
        format_issues(issues, limit=args.limit)


def cmd_summary(args: argparse.Namespace) -> None:
    """Print issue counts grouped by severity and rule."""
    host, token, project_key, _ = resolve_auth_and_endpoint(args)
    issues = fetch_open_issues(host, token, project_key)

    by_severity: dict[str, int] = {}
    by_rule: dict[str, int] = {}
    by_type: dict[str, int] = {}

    for issue in issues:
        sev = issue.get("severity", "UNKNOWN")
        rule = issue.get("rule", "UNKNOWN")
        itype = issue.get("type", "UNKNOWN")

        by_severity[sev] = by_severity.get(sev, 0) + 1
        by_rule[rule] = by_rule.get(rule, 0) + 1
        by_type[itype] = by_type.get(itype, 0) + 1

    print(f"SonarQube Summary for {project_key}")
    print(f"Total Open Issues: {len(issues)}\n")

    print("By Severity:")
    for sev, count in sorted(by_severity.items(), key=lambda x: x[1], reverse=True):
        print(f"  {sev:12s}: {count}")

    print("\nBy Type:")
    for itype, count in sorted(by_type.items(), key=lambda x: x[1], reverse=True):
        print(f"  {itype:14s}: {count}")

    print("\nBy Rule:")
    for rule, count in sorted(by_rule.items(), key=lambda x: x[1], reverse=True):
        print(f"  {rule:30s}: {count}")


def cmd_rule(args: argparse.Namespace) -> None:
    """Show details and remediation guide for a specific rule."""
    host, token, _, org = resolve_auth_and_endpoint(args)
    url = f"{host}/api/rules/show?key={urllib.parse.quote(args.rule_key)}"
    if org:
        url += f"&organization={urllib.parse.quote(org)}"
    data = api_request(url, token)
    rule = data.get("rule", {})

    print(f"Rule:     {rule.get('key')}")
    print(f"Name:     {rule.get('name')}")
    print(f"Type:     {rule.get('type')}")
    print(f"Severity: {rule.get('severity')}")
    print(f"Lang:     {rule.get('langName')}")
    desc = rule.get("htmlDesc") or rule.get("mdDesc") or ""
    clean_desc = re.sub(r"<[^>]+>", "", desc).strip()
    print("\nDescription:")
    print(clean_desc[:1000] + ("..." if len(clean_desc) > 1000 else ""))


def cmd_parse_mcp(args: argparse.Namespace) -> None:
    """Parse raw JSON output from MCP sonarqube tool and print clean formatted report."""
    raw_content = ""
    if args.file == "-" or not args.file:
        raw_content = sys.stdin.read()
    else:
        file_path = Path(args.file)
        if not file_path.exists():
            sys.exit(f"File not found: {args.file}")
        raw_content = file_path.read_text(encoding="utf-8")

    try:
        data = json.loads(raw_content)
    except json.JSONDecodeError as err:
        sys.exit(f"Failed to parse JSON: {err}")

    if isinstance(data, list):
        format_issues(data, limit=args.limit)
    elif "issues" in data:
        issues = data.get("issues", [])
        print(f"Parsed {len(issues)} issues from MCP output.")
        format_issues(issues, limit=args.limit)
    elif "projectStatus" in data:
        status = data["projectStatus"].get("status", "UNKNOWN")
        print(f"Quality Gate Status: {status}")
        for cond in data["projectStatus"].get("conditions", []):
            print(f"  {cond.get('metricKey')}: {cond.get('actualValue')} ({cond.get('status')})")
    elif "rule" in data:
        rule = data["rule"]
        print(f"Rule: {rule.get('key')} - {rule.get('name')}")
        print(f"Severity: {rule.get('severity')} | Type: {rule.get('type')}")
    else:
        print(f"Unknown MCP JSON structure. Keys: {list(data.keys())}")


def cmd_duplications(args: argparse.Namespace) -> None:
    """List and inspect duplicate lines and blocks reported by Sonar."""
    host, token, project_key, _ = resolve_auth_and_endpoint(args)
    url = (
        f"{host}/api/measures/component_tree"
        f"?component={urllib.parse.quote(project_key)}"
        f"&metricKeys=duplicated_lines,duplicated_blocks,duplicated_lines_density,ncloc"
        f"&qualifiers=FIL"
        f"&ps=500"
    )
    data = api_request(url, token)
    components = data.get("components", [])

    files_with_dup = []
    for c in components:
        measures = {m["metric"]: m.get("value", "0") for m in c.get("measures", [])}
        dup_lines = int(float(measures.get("duplicated_lines", 0)))
        dup_blocks = int(float(measures.get("duplicated_blocks", 0)))
        density = float(measures.get("duplicated_lines_density", 0.0))
        ncloc = int(float(measures.get("ncloc", 0)))
        if dup_lines > 0 or dup_blocks > 0:
            files_with_dup.append({
                "key": c.get("key"),
                "path": c.get("path") or c.get("name"),
                "duplicated_lines": dup_lines,
                "duplicated_blocks": dup_blocks,
                "density": density,
                "ncloc": ncloc,
            })

    files_with_dup.sort(key=lambda x: x["duplicated_lines"], reverse=True)

    print(f"Duplication Report for {project_key}")
    print(f"Total files with duplications: {len(files_with_dup)}\n")

    if not files_with_dup:
        print("No duplicated files found.")
        return

    print(f"{'Path':<55} | {'Lines':>6} | {'Blocks':>6} | {'Density':>8} | {'NCLOC':>6}")
    print("-" * 88)
    for f in files_with_dup:
        print(f"{f['path']:<55} | {f['duplicated_lines']:>6} | {f['duplicated_blocks']:>6} | {f['density']:>7.1f}% | {f['ncloc']:>6}")

    if args.details:
        print("\n" + "=" * 88)
        print("Detailed Duplication Blocks:")
        print("=" * 88)
        for f in files_with_dup:
            if args.file and args.file not in f["path"]:
                continue
            dup_url = f"{host}/api/duplications/show?key={urllib.parse.quote(f['key'])}"
            try:
                dup_data = api_request(dup_url, token)
            except Exception as e:
                print(f"\nCould not fetch duplication details for {f['path']}: {e}")
                continue

            duplications = dup_data.get("duplications", [])
            files_map = dup_data.get("files", {})

            if duplications:
                print(f"\nFile: {f['path']} ({f['duplicated_lines']} duplicated lines in {f['duplicated_blocks']} blocks)")
                for idx, d in enumerate(duplications, 1):
                    blocks = d.get("blocks", [])
                    print(f"  Block #{idx}:")
                    for b in blocks:
                        file_ref = files_map.get(b.get("_ref"), {})
                        b_file = file_ref.get("name") or file_ref.get("key", "same file")
                        from_line = b.get("from")
                        size = b.get("size")
                        to_line = from_line + size - 1 if from_line and size else "?"
                        print(f"    - {b_file}:{from_line}-{to_line} ({size} lines)")



def main() -> None:
    parser = argparse.ArgumentParser(
        description="SonarQube / SonarCloud helper for GlazeWM.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--host", help="SonarQube host URL (default: SonarCloud)")
    parser.add_argument("--token", help="SonarQube API user token")
    parser.add_argument("--project", help="SonarQube project key")

    subparsers = parser.add_subparsers(dest="command", required=True)

    # status
    p_status = subparsers.add_parser("status", help="Check quality gate status")
    p_status.set_defaults(func=cmd_status)

    # summary
    p_summary = subparsers.add_parser("summary", help="Summarize issues by rule and severity")
    p_summary.set_defaults(func=cmd_summary)

    # issues
    p_issues = subparsers.add_parser("issues", help="List open issues")
    p_issues.add_argument("--severity", help="Filter by severity (INFO, MINOR, MAJOR, CRITICAL, BLOCKER)")
    p_issues.add_argument("--type", help="Filter by type (BUG, VULNERABILITY, CODE_SMELL)")
    p_issues.add_argument("--rule", help="Filter by rule key (e.g. rust:S3776)")
    p_issues.add_argument("--file", help="Filter by filename/path substring")
    p_issues.add_argument("--limit", type=int, help="Limit number of results")
    p_issues.add_argument("--json", action="store_true", help="Output raw JSON")
    p_issues.set_defaults(func=cmd_issues)

    # rule
    p_rule = subparsers.add_parser("rule", help="Show rule details and guidance")
    p_rule.add_argument("rule_key", help="Rule key (e.g. rust:S3776, yaml:S7630)")
    p_rule.set_defaults(func=cmd_rule)

    # parse-mcp
    p_mcp = subparsers.add_parser("parse-mcp", help="Parse raw JSON output from MCP sonarqube tool")
    p_mcp.add_argument("file", nargs="?", default="-", help="Path to JSON file, or '-' for stdin")
    p_mcp.add_argument("--limit", type=int, help="Limit number of results to display")
    p_mcp.set_defaults(func=cmd_parse_mcp)

    # duplications
    p_dup = subparsers.add_parser("duplications", help="List and inspect code duplications")
    p_dup.add_argument("--details", action="store_true", help="Show detailed matching blocks and lines")
    p_dup.add_argument("--file", help="Filter details by filename substring")
    p_dup.set_defaults(func=cmd_duplications)

    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
