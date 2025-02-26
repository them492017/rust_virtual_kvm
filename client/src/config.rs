use std::{env::Args, net::SocketAddr};

use common::error::DynError;

pub fn parse_args(args: Args) -> Result<(SocketAddr, SocketAddr), DynError> {
    let args: Vec<_> = args.collect();

    if args.len() < 3 {
        panic!("Not enough arguments. Please provide a server address and client address");
    }

    Ok((args[1].parse()?, args[2].parse()?))
}
