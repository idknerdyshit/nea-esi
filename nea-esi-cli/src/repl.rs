use std::borrow::Cow;

use clap::Parser;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config as RlConfig, Editor, Helper};

use crate::cli::Cli;
use crate::commands::ExecContext;
use crate::config::Config;
use crate::output::OutputFormat;

/// Top-level commands available in the REPL.
const ROOT_COMMANDS: &[&str] = &[
    "auth",
    "config",
    "status",
    "market",
    "character",
    "corporation",
    "alliance",
    "universe",
    "wallet",
    "skills",
    "assets",
    "mail",
    "fleet",
    "industry",
    "contracts",
    "killmails",
    "search",
    "sovereignty",
    "wars",
    "fw",
    "dogma",
    "navigation",
    "contacts",
    "fittings",
    "calendar",
    "clones",
    "loyalty",
    "pi",
    "mining",
    "resolve",
];

const BUILTINS: &[&str] = &["exit", "quit", "help", "cd", "set"];

struct EsiHelper {
    context: Option<String>,
}

impl Completer for EsiHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];
        let words: Vec<&str> = prefix.split_whitespace().collect();

        // Determine what to complete
        let (start, candidates) =
            if words.is_empty() || (words.len() == 1 && !prefix.ends_with(' ')) {
                // Completing the first word
                let partial = words.first().copied().unwrap_or("");
                let start = pos - partial.len();

                let all: Vec<&str> = if self.context.is_some() {
                    // In a context, offer subcommands via clap help text
                    // For now just offer builtins + root commands
                    BUILTINS
                        .iter()
                        .chain(ROOT_COMMANDS.iter())
                        .copied()
                        .collect()
                } else {
                    BUILTINS
                        .iter()
                        .chain(ROOT_COMMANDS.iter())
                        .copied()
                        .collect()
                };

                let matches: Vec<Pair> = all
                    .into_iter()
                    .filter(|c| c.starts_with(partial))
                    .map(|c| Pair {
                        display: c.to_string(),
                        replacement: c.to_string(),
                    })
                    .collect();

                (start, matches)
            } else {
                // Completing a subcommand — get subcommands for the first word
                let cmd = words[0];
                let partial = if prefix.ends_with(' ') {
                    ""
                } else {
                    words.last().copied().unwrap_or("")
                };
                let start = pos - partial.len();

                let subcmds = get_subcommands(cmd);
                let matches: Vec<Pair> = subcmds
                    .into_iter()
                    .filter(|c| c.starts_with(partial))
                    .map(|c| Pair {
                        display: c.to_string(),
                        replacement: c.to_string(),
                    })
                    .collect();

                (start, matches)
            };

        Ok((start, candidates))
    }
}

impl Hinter for EsiHelper {
    type Hint = String;
}

impl Highlighter for EsiHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Borrowed(prompt)
    }
}

impl Validator for EsiHelper {}
impl Helper for EsiHelper {}

/// Get known subcommands for a given top-level command.
fn get_subcommands(cmd: &str) -> Vec<&'static str> {
    match cmd {
        "auth" => vec!["login", "logout", "status"],
        "config" => vec!["init", "show", "set"],
        "market" => vec![
            "history",
            "orders",
            "prices",
            "types",
            "groups",
            "group",
            "structure-orders",
        ],
        "character" => vec![
            "info",
            "portrait",
            "affiliation",
            "roles",
            "titles",
            "corporation-history",
            "medals",
            "agents-research",
            "fatigue",
            "fw-stats",
            "fleet",
            "standings",
            "location",
            "ship",
            "online",
            "opportunities",
            "notifications",
            "contact-notifications",
            "killmails",
            "search",
        ],
        "corporation" => vec![
            "info",
            "alliance-history",
            "icons",
            "member-limit",
            "members",
            "member-tracking",
            "member-titles",
            "member-roles",
            "roles-history",
            "structures",
            "starbases",
            "starbase-detail",
            "divisions",
            "facilities",
            "fw-stats",
            "medals",
            "medals-issued",
            "container-logs",
            "customs-offices",
            "shareholders",
            "titles",
            "contacts",
            "contact-labels",
            "standings",
        ],
        "alliance" => vec![
            "info",
            "icons",
            "corporations",
            "contacts",
            "contact-labels",
            "list",
        ],
        "universe" => vec![
            "type",
            "types",
            "group",
            "groups",
            "category",
            "categories",
            "system",
            "systems",
            "constellation",
            "constellations",
            "region",
            "regions",
            "station",
            "stargate",
            "structure",
            "ancestries",
            "bloodlines",
            "races",
            "factions",
            "asteroid-belt",
            "moon",
            "planet",
            "star",
            "graphic",
            "graphics",
            "schematic",
            "public-structures",
            "system-jumps",
            "system-kills",
        ],
        "wallet" => vec![
            "balance",
            "journal",
            "transactions",
            "corp-balances",
            "corp-journal",
            "corp-transactions",
        ],
        "skills" => vec!["list", "queue", "attributes", "implants"],
        "assets" => vec![
            "list",
            "names",
            "locations",
            "corp-list",
            "corp-names",
            "corp-locations",
        ],
        "mail" => vec![
            "list",
            "read",
            "labels",
            "delete-label",
            "delete",
            "mailing-lists",
        ],
        "fleet" => vec![
            "my-fleet",
            "info",
            "members",
            "wings",
            "kick",
            "create-wing",
            "delete-wing",
            "rename-wing",
            "create-squad",
            "delete-squad",
            "rename-squad",
        ],
        "industry" => vec![
            "jobs",
            "blueprints",
            "corp-jobs",
            "corp-blueprints",
            "facilities",
            "systems",
        ],
        "contracts" => vec![
            "list",
            "items",
            "bids",
            "corp-list",
            "corp-items",
            "corp-bids",
            "public",
            "public-items",
            "public-bids",
        ],
        "killmails" => vec!["get", "character", "corporation", "war"],
        "search" => vec!["public", "character"],
        "sovereignty" => vec!["map", "campaigns", "structures"],
        "wars" => vec!["list", "get", "killmails"],
        "fw" => vec![
            "stats",
            "systems",
            "leaderboards",
            "wars",
            "character-leaderboards",
            "corporation-leaderboards",
        ],
        "dogma" => vec![
            "attribute",
            "effect",
            "dynamic-item",
            "attributes",
            "effects",
        ],
        "navigation" => vec![
            "route",
            "waypoint",
            "open-contract",
            "open-info",
            "open-market",
        ],
        "contacts" => vec!["list", "labels", "add", "edit", "delete"],
        "fittings" => vec!["list", "delete"],
        "calendar" => vec!["list", "event", "respond", "attendees"],
        "clones" => vec!["list", "implants", "fatigue"],
        "loyalty" => vec!["points", "offers"],
        "pi" => vec!["planets", "planet"],
        "mining" => vec!["ledger", "observers", "observer-details", "extractions"],
        "resolve" => vec!["names", "ids"],
        _ => vec![],
    }
}

#[allow(clippy::too_many_lines)]
pub async fn run(mut ctx: ExecContext, _config: Config) -> anyhow::Result<()> {
    let rl_config = RlConfig::builder()
        .completion_type(CompletionType::List)
        .build();

    let helper = EsiHelper { context: None };
    let mut rl = Editor::with_config(rl_config)?;
    rl.set_helper(Some(helper));

    // Load history
    if let Some(path) = Config::history_path() {
        let _ = rl.load_history(&path);
    }

    let mut context: Option<String> = None;

    eprintln!("nea-esi-cli interactive mode. Type 'help' for commands, 'exit' to quit.");

    loop {
        let prompt = match &context {
            Some(ctx_name) => format!("esi:{ctx_name}> "),
            None => "esi> ".to_string(),
        };

        // Update helper context
        if let Some(h) = rl.helper_mut() {
            h.context.clone_from(&context);
        }

        let line = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => return Err(e.into()),
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        rl.add_history_entry(trimmed)?;

        // Handle built-in commands
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        if trimmed == "help" {
            print_help(context.as_ref());
            continue;
        }

        if trimmed.starts_with("cd ") || trimmed == "cd" {
            let target = trimmed.strip_prefix("cd").unwrap().trim();
            match target {
                "" | "/" | ".." => {
                    context = None;
                }
                name if ROOT_COMMANDS.contains(&name) => {
                    context = Some(name.to_string());
                }
                other => {
                    eprintln!("Unknown category: {other}");
                    eprintln!("Available: {}", ROOT_COMMANDS.join(", "));
                }
            }
            continue;
        }

        if trimmed.starts_with("set format ") {
            let fmt = trimmed.strip_prefix("set format ").unwrap().trim();
            match fmt {
                "json" => ctx.format = OutputFormat::Json,
                "table" => ctx.format = OutputFormat::Table,
                "csv" => ctx.format = OutputFormat::Csv,
                other => eprintln!("Unknown format: {other}. Use json, table, or csv."),
            }
            continue;
        }

        // Build full command line
        let full_line = match &context {
            Some(ctx_name) => format!("{ctx_name} {trimmed}"),
            None => trimmed.to_string(),
        };

        // Parse with clap
        let mut args = vec!["nea-esi-cli".to_string()];
        if let Some(parts) = shlex::split(&full_line) {
            args.extend(parts);
        } else {
            eprintln!("Error: unmatched quotes in input");
            continue;
        }

        match Cli::try_parse_from(&args) {
            Ok(parsed) => {
                if matches!(parsed.command, crate::cli::Command::Interactive) {
                    eprintln!("Already in interactive mode.");
                    continue;
                }

                // Apply any per-command overrides, restoring defaults after dispatch.
                let saved_character_id = ctx.character_id;
                let saved_format = ctx.format;

                if let Some(cid) = parsed.character_id {
                    ctx.character_id = Some(cid);
                }
                if let Some(fmt) = parsed.format.as_deref() {
                    ctx.format = OutputFormat::from_str_or_auto(Some(fmt));
                }

                if let Err(e) = crate::dispatch(&ctx, parsed.command).await {
                    eprintln!("Error: {e}");
                }

                // Restore original context values.
                ctx.character_id = saved_character_id;
                ctx.format = saved_format;

                // Save tokens if refreshed
                if let Some(tokens) = ctx.client.get_tokens().await {
                    let _ = crate::token_store::save_tokens(&tokens);
                }
            }
            Err(e) => {
                // Clap parse errors include help text
                eprintln!("{e}");
            }
        }
    }

    // Save history
    if let Some(path) = Config::history_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = rl.save_history(&path);
    }

    Ok(())
}

fn print_help(context: Option<&String>) {
    if let Some(ctx_name) = context {
        let subcmds = get_subcommands(ctx_name);
        eprintln!("In context: {ctx_name}");
        eprintln!("Subcommands: {}", subcmds.join(", "));
        eprintln!();
        eprintln!("Built-ins: cd .., cd /, help, exit, set format <fmt>");
    } else {
        eprintln!("Commands: {}", ROOT_COMMANDS.join(", "));
        eprintln!();
        eprintln!("Navigation: cd <command> to enter context, cd .. to go back");
        eprintln!("Settings:   set format <json|table|csv>");
        eprintln!("Other:      help, exit, quit");
    }
}
