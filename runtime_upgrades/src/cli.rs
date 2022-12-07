use clap::Parser;

const LOCAL_WS: &str = "ws://localhost:40043";

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	/// websocket endpoint of the source chain
	#[clap(short, long, value_parser, default_value_t = SPIRITNET_URL.to_string())]
	pub ws_address: String,
}
