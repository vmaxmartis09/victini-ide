pub mod ast_slice;
pub mod bus_verdict;
pub mod devops_sync;
pub mod router;
pub mod s3_guard;

/// Dispatches CLI subcommands when Victini is invoked with arguments (e.g. `victini route`, `victini sloplint`).
pub fn try_run_cli_subcommand(args: &[String]) -> Option<i32> {
    let sub = args.first()?.as_str();
    let tail = &args[1..];
    match sub {
        "route" => Some(router::run_route(tail)),
        "slice" => Some(ast_slice::run_slice(tail)),
        "ast-blast" => Some(ast_slice::run_ast_blast(tail)),
        "env-guard" => Some(devops_sync::run_env_guard(tail)),
        "browser-bridge" => Some(devops_sync::run_browser_bridge()),
        "sync" => Some(devops_sync::run_sync(tail)),
        "update" => Some(devops_sync::run_update()),
        "check-edit" => Some(s3_guard::run_check_edit(tail)),
        "sloplint" => Some(s3_guard::run_sloplint(tail)),
        "seo-check" => Some(s3_guard::run_seo_check(tail)),
        "bus" => Some(bus_verdict::run_bus(tail)),
        "qa-filter" => Some(bus_verdict::run_qa_filter(tail)),
        "verdict" => Some(bus_verdict::run_verdict(tail)),
        _ => None,
    }
}
