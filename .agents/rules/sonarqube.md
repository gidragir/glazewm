# SonarQube & SonarCloud Rules for GlazeWM

This rule defines how AI assistants and developers interact with SonarQube / SonarCloud in this repository.

## 1. Project Context & Endpoints

- **Platform**: SonarCloud (`https://sonarcloud.io`)
- **Project Key**: `gidragir_glazewm`
- **Organization**: `gidragir`
- **Config File**: `sonar-project.properties` at repository root

## 2. Standard Tooling & Utilities

### Project Utility Script (`resources/scripts/sonar.py`)
To inspect quality gates, search issues, or review rules, **always use the built-in CLI** or `mise` tasks instead of writing ad-hoc python scripts:

```bash
# Check quality gate status
mise run sonar:status
python3 resources/scripts/sonar.py status

# High-level summary of open issues by rule & severity
mise run sonar:summary
python3 resources/scripts/sonar.py summary

# List open issues with filters
mise run sonar:issues
python3 resources/scripts/sonar.py issues --severity CRITICAL --limit 10
python3 resources/scripts/sonar.py issues --rule rust:S3776
python3 resources/scripts/sonar.py issues --file packages/wm/src/wm.rs

# View detailed rule documentation and remediation examples
python3 resources/scripts/sonar.py rule rust:S3776
python3 resources/scripts/sonar.py rule yaml:S7630

# Parse raw MCP tool output JSON without writing one-off scripts
python3 resources/scripts/sonar.py parse-mcp path/to/mcp_output.json
```

### Model Context Protocol (MCP) Server
When calling tools from the `sonarqube` MCP server directly:
- **`search_sonar_issues`**: Always pass `projectKey: "gidragir_glazewm"` and `resolved: false`.
- **`get_quality_gate_status`**: Always pass `projectKey: "gidragir_glazewm"`.
- **`get_rule_details`**: Pass `key: "<rule_key>"` (e.g. `rust:S3776`).
- **`get_sonar_fix_plan`**: Pass `issueKey` to retrieve tailored remediation context.

## 3. Workflow for Addressing Sonar Issues

1. **Audit & Prioritize**: Run `mise run sonar:summary` to get the distribution of issues. Focus on BLOCKER and CRITICAL vulnerabilities and code smells first.
2. **Inspect Context**: Run `python3 resources/scripts/sonar.py issues --rule <rule_key>` to locate the exact source files and line numbers.
3. **Refactor Idiomatically**:
   - For **`rust:S3776`** (Cognitive Complexity): Decompose methods into cohesive, private helper functions with cognitive complexity $\le 8$.
   - Preserve `#![warn(clippy::all, clippy::pedantic)]` compliance:
     - Annotate casts with reasons or use lossless `f64::from(...)`.
     - Avoid `unwrap()` or `expect()` in production code.
     - Keep argument counts $\le 7$ or pass reference to models/structs.
4. **Verify Locally**:
   - `mise run check`
   - `mise run clippy`
   - `mise run test`
5. **Quality Gate Verification**: Run `mise run sonar:status` to verify quality gate thresholds.
