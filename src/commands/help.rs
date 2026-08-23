use crate::commands::help_data::{browse, ops, workflow};
use crate::utils::moe::is_moe;

pub fn cmd_help() -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut output = String::new();
    let moe = is_moe();
    browse::push_header(&mut output, moe);
    browse::push_ls(&mut output, moe);
    browse::push_regex(&mut output, moe);
    browse::push_pwd(&mut output, moe);
    browse::push_cd(&mut output, moe);
    browse::push_open(&mut output, moe);
    ops::push_mv(&mut output, moe);
    ops::push_mkdf(&mut output, moe);
    ops::push_change(&mut output, moe);
    ops::push_misc(&mut output, moe);
    ops::push_shell(&mut output, moe);
    workflow::push_keyboard(&mut output, moe);
    workflow::push_aliases(&mut output, moe);
    workflow::push_tags(&mut output, moe);
    workflow::push_chain(&mut output, moe);
    Ok((output, String::new()))
}
