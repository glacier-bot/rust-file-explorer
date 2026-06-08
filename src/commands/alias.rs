use crate::managers::alias::AliasManager;
use colored::*;

#[inline]
fn format_alias_list(alias_manager: &AliasManager) -> String {
    let mut output = String::new();
    let aliases = alias_manager.list();
    if aliases.is_empty() {
        output.push_str(&format!("  {}\n", "No aliases defined yet.".bright_black()));
    } else {
        for (alias, path) in aliases {
            output.push_str(&format!(
                "  {} -> {}\n",
                format!("@{}", alias).cyan().bold(),
                path.bright_cyan()
            ));
        }
    }
    output
}

#[inline]
fn format_alias_usage() -> String {
    let mut output = String::new();
    output.push_str(&format!("\n{} Usage:\n", "💡".bright_green()));
    output.push_str(&format!(
        "  {}    Add/Update alias\n",
        "alias add <name> <path>".cyan().bold()
    ));
    output.push_str(&format!(
        "  {}    Remove alias\n",
        "alias remove <name>".cyan().bold()
    ));
    output.push_str(&format!(
        "  {}         List all aliases\n",
        "alias list".cyan().bold()
    ));
    output
}

pub fn cmd_alias(
    alias_manager: &mut AliasManager,
    args: &[&str],
) -> Result<(String, String), Box<dyn std::error::Error>> {
    if args.is_empty() {
        let mut output = format!("{}\n\n", "📛 Alias List:".bright_yellow().bold());
        output.push_str(&format_alias_list(alias_manager));
        output.push_str(&format_alias_usage());
        return Ok((output, String::new()));
    }

    match args[0].to_lowercase().as_str() {
        "add" | "set" => {
            if args.len() < 3 {
                return Err("Usage: alias add <name> <path>".into());
            }
            let alias = args[1];
            let path = args[2];
            alias_manager.add(alias, path)?;
            Ok((
                format!(
                    "{} Added alias {} -> {}",
                    "✔".bright_green(),
                    format!("@{}", alias).cyan(),
                    path.bright_cyan()
                ),
                String::new(),
            ))
        }
        "remove" | "rm" | "delete" => {
            if args.len() < 2 {
                return Err("Usage: alias remove <name>".into());
            }
            let alias = args[1];
            alias_manager.remove(alias)?;
            Ok((
                format!(
                    "{} Removed alias {}",
                    "✔".bright_green(),
                    format!("@{}", alias).cyan()
                ),
                String::new(),
            ))
        }
        "list" | "ls" => {
            let mut output = format!("{}\n\n", "📛 Alias List:".bright_yellow().bold());
            output.push_str(&format_alias_list(alias_manager));
            Ok((output, String::new()))
        }
        _ => Err(format!("Unknown alias subcommand: {}", args[0]).into()),
    }
}
