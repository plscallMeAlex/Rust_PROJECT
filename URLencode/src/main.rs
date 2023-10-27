use clap::Parser;
use URLencode::{Urlencode,Command, savefile};


fn main() {
    let args = Urlencode::parse();
    match args.inp {
        Command::Encode(form) => {
            let encoded_text = form.linking().unwrap(); // Call linking() on the Command::Encode variant
            let readinp = form.getbefore_process().unwrap();
            let _ = savefile(readinp, encoded_text.0, encoded_text.1);
            // println!("Encoded text: {:?}", encoded_text);
        },
        Command::Decode(form) => {
            let decoded_text = form.linking().unwrap(); // Call linking() on the Command::Decode variant
            let readinp = form.getbefore_process().unwrap();
            let _ = savefile(readinp, decoded_text.0, decoded_text.1);
        
        }
    }
}
