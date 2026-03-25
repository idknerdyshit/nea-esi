use std::borrow::Cow;

use clap::{CommandFactory, Parser};
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
        let active_context = self.context.as_deref();

        let (start, candidates) =
            if words.is_empty() || (words.len() == 1 && !prefix.ends_with(' ')) {
                let partial = words.first().copied().unwrap_or("");
                let start = pos - partial.len();
                let all: Vec<String> = BUILTINS
                    .iter()
                    .copied()
                    .map(str::to_string)
                    .chain(match active_context {
                        Some(ctx_name) => get_subcommands(ctx_name),
                        None => root_commands(),
                    })
                    .collect();

                let matches: Vec<Pair> = all
                    .into_iter()
                    .filter(|c| c.starts_with(partial))
                    .map(|c| Pair {
                        display: c.clone(),
                        replacement: c,
                    })
                    .collect();

                (start, matches)
            } else {
                let cmd = words[0];
                let partial = if prefix.ends_with(' ') {
                    ""
                } else {
                    words.last().copied().unwrap_or("")
                };
                let start = pos - partial.len();
                let subcmds = if is_root_command(cmd) {
                    get_subcommands(cmd)
                } else {
                    active_context
                        .map(get_subcommands)
                        .unwrap_or_default()
                };
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
fn get_subcommands(cmd: &str) -> Vec<String> {
    let command = Cli::command();
    command
        .find_subcommand(cmd)
        .map(|subcommand| {
            subcommand
                .get_subcommands()
                .map(|child| child.get_name().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn root_commands() -> Vec<String> {
    let command = Cli::command();
    command
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_string())
        .collect()
}

fn is_root_command(cmd: &str) -> bool {
    Cli::command().find_subcommand(cmd).is_some()
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
    if ctx.paths.history_path.exists() {
        let _ = rl.load_history(&ctx.paths.history_path);
    } else if let Some(parent) = ctx.paths.history_path.parent() {
        let _ = std::fs::create_dir_all(parent);
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
                name if root_commands().iter().any(|command| command == name) => {
                    context = Some(name.to_string());
                }
                other => {
                    eprintln!("Unknown category: {other}");
                    eprintln!("Available: {}", root_commands().join(", "));
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
                    ctx.format = OutputFormat::parse(fmt)?;
                }

                if let Err(e) = crate::dispatch(&ctx, parsed.command).await {
                    eprintln!("Error: {e}");
                }

                // Restore original context values.
                ctx.character_id = saved_character_id;
                ctx.format = saved_format;

                // Save tokens if refreshed
                if let Some(tokens) = ctx.client.get_tokens().await {
                    let _ = crate::token_store::save_tokens_at(&tokens, &ctx.paths.token_path);
                }
            }
            Err(e) => {
                // Clap parse errors include help text
                eprintln!("{e}");
            }
        }
    }

    // Save history
    if let Some(parent) = ctx.paths.history_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = rl.save_history(&ctx.paths.history_path);

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
        eprintln!("Commands: {}", root_commands().join(", "));
        eprintln!();
        eprintln!("Navigation: cd <command> to enter context, cd .. to go back");
        eprintln!("Settings:   set format <json|table|csv>");
        eprintln!("Other:      help, exit, quit");
    }
}

#[cfg(test)]
mod tests {
    use rustyline::completion::Completer;
    use clap::CommandFactory;
    use rustyline::history::DefaultHistory;
    use rustyline::Context;

    use crate::cli::Cli;

    use super::root_commands;

    fn complete(context: Option<&str>, line: &str) -> (usize, Vec<String>) {
        let helper = super::EsiHelper {
            context: context.map(str::to_string),
        };
        let history = DefaultHistory::new();
        let ctx = Context::new(&history);

        let (start, pairs) = helper.complete(line, line.len(), &ctx).unwrap();
        let mut values: Vec<String> = pairs.into_iter().map(|pair| pair.replacement).collect();
        values.sort();
        (start, values)
    }

    #[test]
    fn repl_root_commands_match_clap() {
        let command = Cli::command();
        let expected: Vec<String> = command
            .get_subcommands()
            .map(|subcommand| subcommand.get_name().to_string())
            .collect();

        assert_eq!(root_commands(), expected);
    }

    #[test]
    fn root_prompt_completion_includes_root_commands_and_builtins() {
        let (_, values) = complete(None, "");

        assert!(values.contains(&"wallet".to_string()));
        assert!(values.contains(&"auth".to_string()));
        assert!(values.contains(&"exit".to_string()));
    }

    #[test]
    fn wallet_context_completion_includes_wallet_subcommands() {
        let (_, values) = complete(Some("wallet"), "");

        assert!(values.contains(&"corp-journal".to_string()));
        assert!(values.contains(&"corp-transactions".to_string()));
        assert!(values.contains(&"exit".to_string()));
        assert!(!values.contains(&"auth".to_string()));
    }

    #[test]
    fn wallet_context_partial_completion_matches_corp_prefix() {
        let (_, values) = complete(Some("wallet"), "corp-");

        assert!(values.contains(&"corp-journal".to_string()));
        assert!(values.contains(&"corp-transactions".to_string()));
        assert!(!values.contains(&"auth".to_string()));
    }

    #[test]
    fn explicit_root_command_path_still_uses_root_subcommands() {
        let (_, values) = complete(Some("wallet"), "auth ");

        let expected: Vec<String> = Cli::command()
            .find_subcommand("auth")
            .unwrap()
            .get_subcommands()
            .map(|subcommand| subcommand.get_name().to_string())
            .collect();
        let mut expected = expected;
        expected.sort();

        assert_eq!(values, expected);
    }
}
