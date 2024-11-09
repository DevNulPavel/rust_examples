use hand_made_base::{base32_decode, base32_encode, base64_decode, base64_encode};
use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// base32 or base64 encoding
    #[arg(short, long, value_enum)]
    encoding: Encoding,

    /// encode or decode
    #[arg(short, long, value_enum)]
    mode: Mode,

    /// content
    #[arg(short, long)]
    content: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Encoding {
    Base64,
    Base32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Encode,
    Decode,
}

fn main() {
    let args = Args::parse();
    let content = args.content;
    if args.mode == Mode::Encode {
        if args.encoding == Encoding::Base64 {
            let result = base64_encode(content.as_bytes());
            println!("BASE64: {} -> {}", content, result);
        } else {
            let result = base32_encode(content.as_bytes());
            println!("BASE32: {} -> {}", content, result);
        }
    } else {
        let mut buffer = vec![];
        if args.encoding == Encoding::Base64 {
            base64_decode(&content, &mut buffer);
            println!("BASE64: {} -> {}", content, String::from_utf8(buffer).unwrap());
        } else {
            base32_decode(&content, &mut buffer);
            println!("BASE32: {} -> {}", content, String::from_utf8(buffer).unwrap());
        }
    }
}
