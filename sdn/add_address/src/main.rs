use futures::stream::TryStreamExt;
use std::env;

use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, Handle};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        usage();
        return Ok(());
    }

    let link_name = &args[1];
    let ip: IpNetwork = args[2].parse().unwrap_or_else(|_| {
        eprintln!("invalid address");
        std::process::exit(1);
    });

    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    if let Err(e) = add_address(link_name, ip, handle.clone()).await {
        eprintln!("{}", e);
    }
    Ok(())
}

async fn add_address(link_name: &str, ip: IpNetwork, handle: Handle) -> Result<(), Error> {
    let mut links = handle
        .link()
        .get()
        .set_name_filter(link_name.to_string())
        .execute();
    if let Some(link) = links.try_next().await? {
        handle
            .address()
            .add(link.header.index, ip.ip(), ip.prefix())
            .execute()
            .await?
    }
    Ok(())
}

fn usage() {
    eprintln!(
        "usage:
    cargo run -- <link_name> <ip_address>

Note that you need to run this program as root. Instead of running cargo as root,
build the example normally:

    cargo build

Then find the binary in the target directory:

    sudo ./target/debug/add_address <link_name> <ip_address>"
    );
}
