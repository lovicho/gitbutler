//! Detect which AI coding agent is invoking the CLI, if any.
//!
//! This checks well-known environment variables set by various AI coding agents
//! when they spawn shell commands. Based on the detection approach used by
//! `@vercel/detect-agent`.

use std::env;
use std::ffi::OsString;

/// An AI coding agent that may be driving the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agent {
    ClaudeCode,
    ClaudeCodeCowork,
    Cursor,
    CursorCli,
    Codex,
    Devin,
    GeminiCli,
    GitHubCopilot,
    OpenCode,
    Augment,
    Antigravity,
    Replit,
    V0,
    Crush,
    PulumiNeo,
    Goose,
    Amp,
    Cline,
    RooCode,
    Trae,
    TabnineCli,
    Pi,
    Unknown,
}

impl Agent {
    /// A short, stable identifier suitable for telemetry or output-format decisions.
    pub fn name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::ClaudeCodeCowork => "claude-code-cowork",
            Self::Cursor => "cursor",
            Self::CursorCli => "cursor-cli",
            Self::Codex => "codex",
            Self::Devin => "devin",
            Self::GeminiCli => "gemini-cli",
            Self::GitHubCopilot => "github-copilot",
            Self::OpenCode => "opencode",
            Self::Augment => "augment",
            Self::Antigravity => "antigravity",
            Self::Replit => "replit",
            Self::V0 => "v0",
            Self::Crush => "crush",
            Self::PulumiNeo => "pulumi-neo",
            Self::Goose => "goose",
            Self::Amp => "amp",
            Self::Cline => "cline",
            Self::RooCode => "roo-code",
            Self::Trae => "trae",
            Self::TabnineCli => "tabnine-cli",
            Self::Pi => "pi",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Detect the current AI coding agent from environment variables.
///
/// Returns `None` when the CLI appears to be invoked by a human.
/// Checks the generic `AI_AGENT` variable first, then tool-specific
/// variables in priority order, and finally the generic `AGENT` variable
/// as a last-resort fallback.
pub fn detect() -> Option<Agent> {
    detect_with(|key| env::var_os(key))
}

/// Core detection logic, parameterised over an env-var lookup function for testability.
fn detect_with(lookup: impl Fn(&str) -> Option<OsString>) -> Option<Agent> {
    let is_set = |var: &str| lookup(var).is_some_and(|v| !v.is_empty());
    let is_value =
        |var: &str, expected: &str| lookup(var).is_some_and(|v| v.to_str() == Some(expected));

    // Generic `AI_AGENT` convention (as documented by `@vercel/detect-agent`).
    if let Some(agent) = parse_ai_agent_var(&lookup) {
        return Some(agent);
    }

    // Tool-specific variables, roughly ordered by popularity.
    if is_set("CLAUDE_CODE_IS_COWORK") {
        return Some(Agent::ClaudeCodeCowork);
    }
    if is_set("CLAUDE_CODE") || is_set("CLAUDECODE") {
        return Some(Agent::ClaudeCode);
    }
    if is_set("CURSOR_AGENT") || is_value("CURSOR_EXTENSION_HOST_ROLE", "agent-exec") {
        return Some(Agent::CursorCli);
    }
    if is_set("CURSOR_TRACE_ID") {
        return Some(Agent::Cursor);
    }
    if is_set("CODEX_SANDBOX")
        || is_set("CODEX_CI")
        || is_set("CODEX_THREAD_ID")
        || is_set("CODEX_SHELL")
    {
        return Some(Agent::Codex);
    }
    if is_set("GEMINI_CLI") {
        return Some(Agent::GeminiCli);
    }
    if is_set("COPILOT_AGENT") {
        return Some(Agent::GitHubCopilot);
    }
    if is_set("OPENCODE_CLIENT") {
        return Some(Agent::OpenCode);
    }
    if is_set("AUGMENT_AGENT") {
        return Some(Agent::Augment);
    }
    if is_set("ANTIGRAVITY_AGENT") {
        return Some(Agent::Antigravity);
    }
    if is_set("REPL_ID") {
        return Some(Agent::Replit);
    }
    // Agents that set neither `AI_AGENT` nor `AGENT`, only a private marker.
    // These markers are per-mode: Cline sets `CLINE_ACTIVE` from its VS Code
    // extension (not its CLI); Roo Code sets `ROO_CLI_RUNTIME` from its headless
    // CLI and `ROO_ACTIVE` from its extension. Presence is enough to identify
    // the agent when it is set.
    if is_set("CLINE_ACTIVE") {
        return Some(Agent::Cline);
    }
    if is_set("ROO_CLI_RUNTIME") || is_set("ROO_ACTIVE") {
        return Some(Agent::RooCode);
    }
    if is_set("TRAE_AI_SHELL_ID") {
        return Some(Agent::Trae);
    }
    if is_set("TABNINE_CLI") {
        return Some(Agent::TabnineCli);
    }
    if is_set("PI_CODING_AGENT") {
        return Some(Agent::Pi);
    }

    // Shorter `AGENT` convention (Goose, Amp, Crush, Codex), checked last: it is
    // a generic variable name and can be stale or inherited from a parent shell,
    // so a fresh tool-specific marker above should win over it.
    if let Some(agent) = parse_agent_var(&lookup) {
        return Some(agent);
    }

    None
}

/// Parse the generic `AI_AGENT` env var into an agent detection.
///
/// A non-empty value we don't recognize still resolves to `Agent::Unknown`:
/// setting `AI_AGENT` is an explicit "an agent is driving this" signal, so we
/// keep it even when we can't name the agent.
fn parse_ai_agent_var(lookup: &impl Fn(&str) -> Option<OsString>) -> Option<Agent> {
    let val = normalize_agent_value(&lookup("AI_AGENT")?.to_string_lossy());
    if val.is_empty() {
        return None;
    }
    Some(
        match_agent_name(&val)
            .or_else(|| match_agent_name_prefix(&val))
            .unwrap_or(Agent::Unknown),
    )
}

/// Match a normalized `AI_AGENT` value whose leading segments name a known
/// agent, tolerating trailing decorations that embed a version without the
/// `@` separator — Claude Code desktop sets e.g. `claude-code_2-1-202_agent`.
/// Segments are dropped from the end one at a time, so the longest matching
/// prefix wins (`claude-code-...` resolves before the `claude` alias could).
fn match_agent_name_prefix(val: &str) -> Option<Agent> {
    let mut prefix = val;
    while let Some((rest, _)) = prefix.rsplit_once('-') {
        if let Some(agent) = match_agent_name(rest) {
            return Some(agent);
        }
        prefix = rest;
    }
    None
}

/// Parse the shorter `AGENT` convention (distinct from `AI_AGENT`), adopted by
/// Goose (`AGENT=goose`), Amp (`AGENT=amp`), Crush (`AGENT=crush`), and Codex.
///
/// `AGENT` is a generic variable name, so only a strict allowlist of known
/// values counts as an agent signal — anything else is ignored (returns `None`)
/// rather than reported as `Agent::Unknown`, to avoid false positives.
fn parse_agent_var(lookup: &impl Fn(&str) -> Option<OsString>) -> Option<Agent> {
    match normalize_agent_value(&lookup("AGENT")?.to_string_lossy()).as_str() {
        "goose" => Some(Agent::Goose),
        "amp" => Some(Agent::Amp),
        "crush" => Some(Agent::Crush),
        "codex" => Some(Agent::Codex),
        _ => None,
    }
}

/// Map a recognized agent identifier to an [`Agent`], or `None` if unknown.
fn match_agent_name(val: &str) -> Option<Agent> {
    Some(match val {
        "claude" | "claude-code" => Agent::ClaudeCode,
        "cowork" | "claude-code-cowork" => Agent::ClaudeCodeCowork,
        "cursor" => Agent::Cursor,
        "cursor-cli" => Agent::CursorCli,
        "codex" => Agent::Codex,
        "devin" => Agent::Devin,
        "gemini" | "gemini-cli" => Agent::GeminiCli,
        "copilot" | "github-copilot" | "github-copilot-cli" | "github-copilot-vscode-agent" => {
            Agent::GitHubCopilot
        }
        "opencode" => Agent::OpenCode,
        "augment" | "augment-cli" => Agent::Augment,
        "antigravity" => Agent::Antigravity,
        "replit" => Agent::Replit,
        "v0" => Agent::V0,
        "crush" => Agent::Crush,
        "neo" | "pulumi-neo" => Agent::PulumiNeo,
        "goose" => Agent::Goose,
        "amp" => Agent::Amp,
        "cline" => Agent::Cline,
        "roo-code" => Agent::RooCode,
        "trae" => Agent::Trae,
        "tabnine-cli" => Agent::TabnineCli,
        "pi" => Agent::Pi,
        _ => return None,
    })
}

/// Normalize an agent identifier so casing and separator drift still match a
/// canonical name. Trims, drops any `@version` suffix (the `AI_AGENT` naming
/// convention allows e.g. `claude-code@1`), lowercases, and collapses runs of
/// `_`, `-`, and whitespace into a single `-` (so `Claude_Code`, `claude code`,
/// and `claude-code@2` all resolve to `claude-code`).
fn normalize_agent_value(raw: &str) -> String {
    raw.trim()
        .split('@')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .split(|c: char| c == '_' || c == '-' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Build a lookup function from a set of key-value pairs.
    /// `use<>` tells the compiler the returned closure owns all its data and
    /// borrows nothing from `vars` (which is consumed into the `HashMap`).
    fn env_from(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> + use<> {
        let map: HashMap<String, OsString> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), OsString::from(v)))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    #[test]
    fn detect_claude_code() {
        assert_eq!(
            detect_with(env_from(&[("CLAUDE_CODE", "1")])),
            Some(Agent::ClaudeCode),
        );
    }

    #[test]
    fn detect_claude_code_legacy_var() {
        assert_eq!(
            detect_with(env_from(&[("CLAUDECODE", "1")])),
            Some(Agent::ClaudeCode),
        );
    }

    #[test]
    fn detect_cowork() {
        assert_eq!(
            detect_with(env_from(&[("CLAUDE_CODE_IS_COWORK", "1")])),
            Some(Agent::ClaudeCodeCowork),
        );
    }

    #[test]
    fn detect_cursor() {
        assert_eq!(
            detect_with(env_from(&[("CURSOR_TRACE_ID", "abc123")])),
            Some(Agent::Cursor),
        );
    }

    #[test]
    fn detect_cursor_cli() {
        assert_eq!(
            detect_with(env_from(&[("CURSOR_AGENT", "1")])),
            Some(Agent::CursorCli),
        );
    }

    #[test]
    fn detect_cursor_cli_extension_host() {
        assert_eq!(
            detect_with(env_from(&[("CURSOR_EXTENSION_HOST_ROLE", "agent-exec")])),
            Some(Agent::CursorCli),
        );
    }

    #[test]
    fn ignores_cursor_extension_host_non_agent_role() {
        assert_eq!(
            detect_with(env_from(&[(
                "CURSOR_EXTENSION_HOST_ROLE",
                "extension-host"
            )])),
            None,
        );
    }

    #[test]
    fn detect_codex() {
        assert_eq!(
            detect_with(env_from(&[("CODEX_SANDBOX", "seatbelt")])),
            Some(Agent::Codex),
        );
    }

    #[test]
    fn detect_codex_shell() {
        assert_eq!(
            detect_with(env_from(&[("CODEX_SHELL", "1")])),
            Some(Agent::Codex),
        );
    }

    #[test]
    fn detect_gemini() {
        assert_eq!(
            detect_with(env_from(&[("GEMINI_CLI", "1")])),
            Some(Agent::GeminiCli),
        );
    }

    #[test]
    fn detect_copilot_agent() {
        assert_eq!(
            detect_with(env_from(&[("COPILOT_AGENT", "1")])),
            Some(Agent::GitHubCopilot),
        );
    }

    #[test]
    fn copilot_config_vars_are_not_agent_signals() {
        for var in ["COPILOT_MODEL", "COPILOT_ALLOW_ALL", "COPILOT_GITHUB_TOKEN"] {
            assert_eq!(
                detect_with(env_from(&[(var, "1")])),
                None,
                "{var} should not be treated as an agent marker",
            );
        }
    }

    #[test]
    fn detect_opencode() {
        assert_eq!(
            detect_with(env_from(&[("OPENCODE_CLIENT", "1")])),
            Some(Agent::OpenCode),
        );
    }

    #[test]
    fn detect_augment() {
        assert_eq!(
            detect_with(env_from(&[("AUGMENT_AGENT", "1")])),
            Some(Agent::Augment),
        );
    }

    #[test]
    fn detect_antigravity() {
        assert_eq!(
            detect_with(env_from(&[("ANTIGRAVITY_AGENT", "1")])),
            Some(Agent::Antigravity),
        );
    }

    #[test]
    fn detect_replit() {
        assert_eq!(
            detect_with(env_from(&[("REPL_ID", "abc")])),
            Some(Agent::Replit),
        );
    }

    #[test]
    fn detect_none_when_clean() {
        assert_eq!(detect_with(|_| None), None);
    }

    #[test]
    fn ai_agent_var_takes_priority() {
        // Even though CLAUDE_CODE is set, AI_AGENT=codex should win.
        assert_eq!(
            detect_with(env_from(&[("AI_AGENT", "codex"), ("CLAUDE_CODE", "1")])),
            Some(Agent::Codex),
        );
    }

    #[test]
    fn empty_var_is_not_detected() {
        assert_eq!(detect_with(env_from(&[("GEMINI_CLI", "")])), None);
    }

    #[test]
    fn empty_ai_agent_var_is_not_detected() {
        assert_eq!(detect_with(env_from(&[("AI_AGENT", "")])), None);
        assert_eq!(detect_with(env_from(&[("AI_AGENT", "  ")])), None);
    }

    #[test]
    fn ai_agent_case_insensitive() {
        assert_eq!(
            detect_with(env_from(&[("AI_AGENT", "Claude-Code")])),
            Some(Agent::ClaudeCode),
        );
    }

    #[test]
    fn ai_agent_accepts_vercel_aliases() {
        let cases = [
            ("claude", Agent::ClaudeCode),
            ("cowork", Agent::ClaudeCodeCowork),
            ("gemini", Agent::GeminiCli),
            ("augment-cli", Agent::Augment),
            ("github-copilot-cli", Agent::GitHubCopilot),
            ("github_copilot_vscode_agent", Agent::GitHubCopilot),
            ("devin", Agent::Devin),
            ("v0", Agent::V0),
        ];

        for (name, agent) in cases {
            assert_eq!(
                detect_with(env_from(&[("AI_AGENT", name)])),
                Some(agent),
                "AI_AGENT alias {name} should resolve to {}",
                agent.name(),
            );
        }
    }

    #[test]
    fn ai_agent_unknown_value_still_detects_agent() {
        assert_eq!(
            detect_with(env_from(&[("AI_AGENT", "some-new-agent")])),
            Some(Agent::Unknown),
        );
    }

    #[test]
    fn ai_agent_matches_known_prefix_with_trailing_decorations() {
        // Claude Code desktop embeds its version without the `@` separator.
        for (value, expected) in [
            ("claude-code_2-1-202_agent", Agent::ClaudeCode),
            ("claude-code-2.1.202", Agent::ClaudeCode),
            ("cursor-cli-extra", Agent::CursorCli),
        ] {
            assert_eq!(
                detect_with(env_from(&[("AI_AGENT", value)])),
                Some(expected),
                "AI_AGENT={value} should prefix-match {}",
                expected.name(),
            );
        }
    }

    #[test]
    fn ai_agent_strips_version_suffix() {
        // The AI_AGENT naming convention allows a `@version` suffix
        // (e.g. `devin@1`, `custom-agent@2.0`); it must not defeat detection.
        for (value, expected) in [
            ("claude-code@1", Agent::ClaudeCode),
            ("devin@1", Agent::Devin),
            ("cursor-cli@2.0", Agent::CursorCli),
            ("gemini-cli@0.1.2", Agent::GeminiCli),
        ] {
            assert_eq!(
                detect_with(env_from(&[("AI_AGENT", value)])),
                Some(expected),
                "AI_AGENT={value} should strip the version and resolve to {}",
                expected.name(),
            );
        }
    }

    #[test]
    fn ai_agent_is_lenient_about_separators() {
        for value in ["claude code", "claude--code", "  Claude-Code  "] {
            assert_eq!(
                detect_with(env_from(&[("AI_AGENT", value)])),
                Some(Agent::ClaudeCode),
                "AI_AGENT={value:?} should normalize to claude-code",
            );
        }
    }

    #[test]
    fn detect_crush() {
        assert_eq!(
            detect_with(env_from(&[("AI_AGENT", "crush")])),
            Some(Agent::Crush),
        );
    }

    #[test]
    fn detect_pulumi_neo() {
        assert_eq!(
            detect_with(env_from(&[("AI_AGENT", "neo")])),
            Some(Agent::PulumiNeo),
        );
    }

    #[test]
    fn detect_goose_via_agent_var() {
        assert_eq!(
            detect_with(env_from(&[("AGENT", "goose")])),
            Some(Agent::Goose),
        );
    }

    #[test]
    fn detect_amp_via_agent_var() {
        assert_eq!(detect_with(env_from(&[("AGENT", "amp")])), Some(Agent::Amp));
    }

    #[test]
    fn agent_var_is_normalized() {
        // The `AGENT` value goes through the same normalization as `AI_AGENT`.
        for value in ["Goose", "GOOSE", "goose@1", "  goose  "] {
            assert_eq!(
                detect_with(env_from(&[("AGENT", value)])),
                Some(Agent::Goose),
                "AGENT={value:?} should normalize to goose",
            );
        }
    }

    #[test]
    fn agent_var_only_matches_known_values() {
        // `AGENT` is generic, so an unrecognized value must not be treated as
        // an agent (unlike `AI_AGENT`, which falls back to `Unknown`).
        assert_eq!(detect_with(env_from(&[("AGENT", "smith")])), None);
        assert_eq!(detect_with(env_from(&[("AGENT", "")])), None);
    }

    #[test]
    fn ai_agent_wins_over_agent_var() {
        assert_eq!(
            detect_with(env_from(&[("AI_AGENT", "codex"), ("AGENT", "goose")])),
            Some(Agent::Codex),
        );
    }

    #[test]
    fn tool_specific_marker_wins_over_stale_agent_var() {
        // A generic `AGENT` can be inherited from a parent shell; a fresh
        // tool-specific marker must still identify the real driver.
        assert_eq!(
            detect_with(env_from(&[("CODEX_THREAD_ID", "t1"), ("AGENT", "goose")])),
            Some(Agent::Codex),
        );
    }

    #[test]
    fn detect_cline() {
        assert_eq!(
            detect_with(env_from(&[("CLINE_ACTIVE", "true")])),
            Some(Agent::Cline),
        );
    }

    #[test]
    fn detect_roo_code() {
        assert_eq!(
            detect_with(env_from(&[("ROO_CLI_RUNTIME", "1")])),
            Some(Agent::RooCode),
        );
        assert_eq!(
            detect_with(env_from(&[("ROO_ACTIVE", "true")])),
            Some(Agent::RooCode),
        );
    }

    #[test]
    fn detect_trae() {
        assert_eq!(
            detect_with(env_from(&[("TRAE_AI_SHELL_ID", "abc123")])),
            Some(Agent::Trae),
        );
    }

    #[test]
    fn detect_tabnine_cli() {
        assert_eq!(
            detect_with(env_from(&[("TABNINE_CLI", "1")])),
            Some(Agent::TabnineCli),
        );
    }

    #[test]
    fn detect_pi() {
        assert_eq!(
            detect_with(env_from(&[("PI_CODING_AGENT", "true")])),
            Some(Agent::Pi),
        );
    }

    #[test]
    fn agent_name_roundtrip() {
        let agents = [
            Agent::ClaudeCode,
            Agent::ClaudeCodeCowork,
            Agent::Cursor,
            Agent::CursorCli,
            Agent::Codex,
            Agent::Devin,
            Agent::GeminiCli,
            Agent::GitHubCopilot,
            Agent::OpenCode,
            Agent::Augment,
            Agent::Antigravity,
            Agent::Replit,
            Agent::V0,
            Agent::Crush,
            Agent::PulumiNeo,
            Agent::Goose,
            Agent::Amp,
            Agent::Cline,
            Agent::RooCode,
            Agent::Trae,
            Agent::TabnineCli,
            Agent::Pi,
            Agent::Unknown,
        ];
        for agent in agents {
            let lookup = env_from(&[("AI_AGENT", agent.name())]);
            assert_eq!(
                detect_with(lookup),
                Some(agent),
                "roundtrip failed for {}",
                agent.name()
            );
        }
    }
}
