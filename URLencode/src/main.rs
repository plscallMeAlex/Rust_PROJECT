use clap::Parser;
use URLencode::{Urlencode,Command};

fn main() {
    let args = Urlencode::parse();
    match args.inp {
        Command::Encode(form) => {
            let encoded_text = form.linking(); // Call linking() on the Command::Encode variant
            println!("Encoded text: {:?}", encoded_text);
            // println!("{:?}",form);
        },
        Command::Decode(form) => {
            let decoded_text = form.linking(); // Call linking() on the Command::Decode variant
            // println!("{:?}",form);
            println!("Decoded text: {:?}", decoded_text);
        }
    }
}

// fn main() {
//     // let args = Urlencode::parse();
//     if let Err(args) = Urlencode::parse() {
//         eprintln!("{}", args);
//         std::process::exit(1);
//     } else {   
//         let args = Urlencode::parse();
//         match args.inp {
//                     Command::Encode(form) => {
//                         let encoded_text = form.linking(); // Call linking() on the Command::Encode variant
//                         println!("Encoded text: {:?}", encoded_text);
//                         // println!("{:?}",form);
//                     },
//                     Command::Decode(form) => {
//                         let decoded_text = form.linking(); // Call linking() on the Command::Decode variant
//                         // println!("{:?}",form);
//                         println!("Decoded text: {:?}", decoded_text);
//                     }
//         }
//     }
// }