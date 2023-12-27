use clap::{
    value_parser,
    Arg,
    ArgAction,
    Command,
};

const SPACIAL_GRAPH: &str = "spacial_graph";
const SPACIAL_GRAPH_FORMAT: &str = "spacial_graph_format";
const DEPENDENCY_GRAPH: &str = "dependency_graph";
const DEPENDENCY_GRAPH_FORMAT: &str = "dependency_graph_format";
const PATHS: &str = "paths";
const PATHS_FORMAT: &str = "paths_format";
const NTHREADS: &str = "nthreads";
const TASK_BOUND: &str = "task_bound";
const SEARCH: &str = "search";

fn build() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about("Compare the documentation of interface::run for more information.")
        .arg_required_else_help(true)
        .arg(
            Arg::new(SPACIAL_GRAPH)
                .value_name("SPACIAL_GRAPH")
                .help("The spacial_graph's file name")
                .required(true),
        )
        .arg(
            Arg::new(SPACIAL_GRAPH_FORMAT)
                .value_name("SPACIAL_GRAPH_FORMAT")
                .help("The spacial_graph's serialization format")
                .required(true),
        )
        .arg(
            Arg::new(DEPENDENCY_GRAPH)
                .value_name("DEPENDENCY_GRAPH")
                .help("The dependency_graph's file name")
                .required(true),
        )
        .arg(
            Arg::new(DEPENDENCY_GRAPH_FORMAT)
                .value_name("DEPENDENCY_GRAPH_FORMAT")
                .help("The dependency_graph's serialization format")
                .required(true),
        )
        .arg(
            Arg::new(PATHS)
                .value_name("PATHS")
                .help("The paths' file name")
                .required(true),
        )
        .arg(
            Arg::new(PATHS_FORMAT)
                .value_name("PATHS_FORMAT")
                .help("The paths' serialization format")
                .required(true),
        )
        .arg(
            Arg::new(SEARCH)
                .short('s')
                .long("search")
                .help("Search for all best paths")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new(NTHREADS)
                .value_name("NTHREADS")
                .short('n')
                .long("nthreads")
                .help("The number of threads to use for the search")
                .default_value("1")
                .value_parser(value_parser!(u16)),
        )
        .arg(
            Arg::new(TASK_BOUND)
                .value_name("TASK_BOUND")
                .short('b')
                .long("task-bound")
                .help("A bound on the possible number of tasks")
                .value_parser(value_parser!(u32)),
        )
}

pub fn parse() -> (
    String,
    String,
    String,
    String,
    String,
    String,
    bool,
    u16,
    Option<u32>,
) {
    let mut args = build().get_matches();
    (
        args.remove_one(SPACIAL_GRAPH).expect("is required"),
        args.remove_one(SPACIAL_GRAPH_FORMAT).expect("is required"),
        args.remove_one(DEPENDENCY_GRAPH).expect("is required"),
        args.remove_one(DEPENDENCY_GRAPH_FORMAT).expect("is required"),
        args.remove_one(PATHS).expect("is required"),
        args.remove_one(PATHS_FORMAT).expect("is required"),
        args.remove_one(SEARCH).expect("is required"),
        args.remove_one(NTHREADS).expect("has default"),
        args.remove_one::<Option<u32>>(TASK_BOUND).unwrap_or(None),
    )
}
