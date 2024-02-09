use clap::{value_parser, Arg, Command};

use crate::plots::{density, node, Args};

fn build() -> Command {
    let num_nodes = Arg::new("num_nodes")
        .required(true)
        .value_name("NUM_NODES")
        .value_parser(value_parser!(usize));
    let edge_density = Arg::new("edge_density")
        .required(true)
        .value_name("EDGE_DENSITY")
        .value_parser(value_parser!(f64));
    let edge_density_type = Arg::new("edge_density_type")
        .required(true)
        .value_name("EDGE_DENSITY_TYPE");
    let correction_density = Arg::new("correction_density")
        .required(true)
        .value_name("CORRECTION_DENSITY")
        .value_parser(value_parser!(f64));
    let correction_density_type = Arg::new("correction_density_type")
        .required(true)
        .value_name("CORRECTION_DENSITY_TYPE");

    Command::new(env!("CARGO_PKG_NAME"))
        .arg_required_else_help(true)
        .subcommand(
            Command::new("node")
                .arg(edge_density.clone())
                .arg(correction_density.clone())
                .arg(edge_density_type.clone())
                .arg(correction_density_type.clone()),
        )
        .subcommand(Command::new("density").arg(num_nodes).arg(edge_density))
}

pub fn parse() -> Args {
    let mut args = build().get_matches();
    let (name, mut args) = args.remove_subcommand().unwrap();
    match name.as_ref() {
        "node" => Args::Node(node::Args {
            edge_density: args.remove_one("edge_density").unwrap(),
            edge_density_type: args.remove_one("edge_density_type").unwrap(),
            correction_density: args.remove_one("correction_density").unwrap(),
            correction_density_type: args.remove_one("correction_density_type").unwrap(),
        }),
        "density" => Args::Density(density::Args {
            num_nodes: args.remove_one("num_nodes").unwrap(),
            edge_density_multiplier: args.remove_one("edge_density").unwrap(),
        }),
        _ => unreachable!(),
    }
}
