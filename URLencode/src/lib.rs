use clap::error::Result;
use clap::{Args, Parser, Subcommand, builder};
use std::error::Error;
use std::{io, clone};
use std::io::{Read,Write, BufReader};
use std::str;
use std::string::FromUtf8Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Urlencode {
    #[clap(subcommand)]
    pub inp: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    ///Encode from inputting URL
    #[clap(short_flag='e',about="Encode input URL components")]
    Encode(Form1),
    ///Decode from inputting URL
    #[clap(short_flag='d', about="Decode input URL components")]
    Decode(Form2),
}

#[derive(Debug, Args)]
pub struct Form1 {
    ///Input text, file or filepath
    #[clap(required=true,value_name="TEXT")]
    pub filetext: StringOrPath,
    
    ///Percent-Encoding
    #[clap(short='p',long="percent",value_name="TO_PERCENT", conflicts_with = "flg2")]
    pub flg1:bool,

    ///Base64-Encoding
    #[clap(short='b',long="base64",value_name="TO_BASE64")]
    pub flg2:bool,
}

#[derive(Debug, Args)]
pub struct Form2 {
    ///Input text, file or filepath
    #[clap(required=true,value_name="TEXT")]
    pub filetext: StringOrPath,
    
    ///Percent-Decoding
    #[clap(short='p',long="percent",value_name="FROM_PERCENT")]
    pub flg1:bool,

    ///Base64-Decoding
    #[clap(short='b',long="base64",value_name="FROM_BASE64")]
    pub flg2:bool,
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Link function to flag

impl Form1 {
    pub fn linking(&self) -> String {
        match &self.filetext {
            StringOrPath::String(inp) => {
                // println!("{:?}",inp);
                if self.flg1 {
                    encoding_percent(&inp)
                } else if self.flg2 {
                    tobase64(&inp) //wiil change when to base64 is finish
                }
                 else {
                    encoding_percent(&inp) //for default choice
                }
            },
            StringOrPath::Path(path) => {
                // println!("dfdf:{:?}",path);
                let filecontent = readfile(&path.clone().into_os_string().into_string().unwrap());
                if self.flg1 {
                    encoding_percent(&filecontent.unwrap())
                } else if self.flg2 {
                    tobase64(&filecontent.unwrap()) //wiil change when to base64 is finish
                } else {
                    encoding_percent(&filecontent.unwrap()) //for default choice
                }
            },
        }
    }
}

impl Form2 {
    pub fn linking(&self) -> String {
        match &self.filetext {
            StringOrPath::String(inp) => {
                if self.flg1 {
                    decoding_percent(&inp)
                } else if self.flg2 {
                    frombase64(&inp).unwrap() //wiil change when to base64 is finish
                } else {
                    decoding_percent(&inp) //for default choice
                }
            },
            StringOrPath::Path(path) => {
                let filecontent = readfile(&path.clone().into_os_string().into_string().unwrap());
                if self.flg1 {
                    decoding_percent(&filecontent.unwrap())
                } else if self.flg2 {
                    frombase64(&filecontent.unwrap()).unwrap() //wiil change when to base64 is finish
                } else {
                    decoding_percent(&filecontent.unwrap()) //for default choice
                }
            },
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Impl pathfile

#[derive(Debug, Clone)]
pub enum StringOrPath {
    String(String),
    Path(PathBuf),
}

impl std::str::FromStr for StringOrPath {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match std::fs::canonicalize(s) {
            Ok(path) => Ok(StringOrPath::Path(path)),
            Err(_) => Ok(StringOrPath::String(s.to_string())),
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//For reading from file

pub fn readfile(filename:&str,) -> Result<String, Box<dyn Error>>{
    match fs::read_to_string(filename) {
        Ok(content) => Ok(content),
        Err(err) => {eprintln!("Error to read file {}: {}", filename,err);
            Err(Box::new(err))
        }
    }
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Percent Encoding and Decoding

#[inline]
pub fn encoding_percent(inp: &str) -> String {
    let mut buffer = Vec::new();
    for &i in inp.as_bytes() {
        match i {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'$' | b'-' | b'_' | b'.' |
            b'+' | b'!' | b'*' | b'(' | b')' => {
                buffer.push(i) 
            },
            _ => {buffer.push(b'%');
            buffer.push(binoperate1(i >> 4)); //for getting 4bits high| example pass i>>4 = 3
            buffer.push(binoperate1(i & 0xF));//for getting 4bits low| example pass i & 0xF = 2
            }
        }        
    }
    let encoded = unsafe{String::from_utf8_unchecked(buffer)};
    encoded
}

#[inline]
pub fn binoperate1(digit: u8) -> u8 {
    match digit {
        0..=9 => b'0' + digit, //it start from 48 + ... it return a number
        10..=15 => b'A' + digit - 10, //start from 65 + ... - 10 it will return a char
        _ => panic!("Invalid input: {}", digit),
    }
}

#[inline]
pub fn decoding_percent(inp: &str) -> String {
    let mut buffer: Vec<u8> = Vec::new();
    let mut bytes = inp.as_bytes().iter();
    while let Some(&b) = bytes.next() {
        match b {
            b'%' => {
                if let (Some(&first), Some(&second)) = (bytes.next(), bytes.next()) {
                    if let (Some(v1), Some(v2)) = (binoperate2(first), binoperate2(second)) {
                        buffer.push((v1 << 4) | v2);
                    } else {
                        buffer.push(b'%');
                        buffer.push(first);
                        buffer.push(second);
                    }
                } else {
                    buffer.push(b'%');
                }
            }
            _ => buffer.push(b),
        }
    }
    String::from_utf8(buffer).expect("Invalid UTF-8 sequence")
}

#[inline]
pub fn binoperate2(inp: u8) -> Option<u8> {
    match inp {
        b'0'..=b'9' => Some(inp - b'0'),
        b'A'..=b'F' => Some(inp - b'A' + 10),
        b'a'..=b'f' => Some(inp - b'a' + 10),
        _ => None,
    }
} //found % it will remove % and next 2 digit

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Base64 Misc: Matching, trait, etc.

fn matchingb64(inp:u8) -> u8{
    match inp {
        0..=25 => b'A' + inp,
        26..=51 => b'a' + (inp - 26),
        52..=61 => b'0' + (inp - 52),
        62 => b'+',
        63 => b'/',
        _ => panic!("Invalid")
    }
}

fn matchingb642(inp:u8) -> Option<u8> {
    match inp{
        b'A'..=b'Z' => Some(inp - b'A'),
        b'a'..=b'z' => Some(inp - b'a' + 26),
        b'0'..=b'9' => Some(inp - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

trait ChecklastDig {
    fn fromlastindex(&self, n:usize) -> char;
}

impl<'a> ChecklastDig for &'a str {
    fn fromlastindex(&self, n:usize) -> char {
        self.chars().rev().nth(n).expect("Out of range")
    }
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Base64 Encoding and Decoding 

#[inline]
pub fn tobase64(inp: &str) -> String {
    let oldbuffer = decoding_percent(inp); //need to decode first before converting
    let iter:Vec<_> = oldbuffer.as_bytes().chunks(3).collect(); //type: [[u8,u8,u8]] (3 bytes)
    let mut result = Vec::new();
    let mut checking:u8 = 0;
    for i in iter {
        if i.len() == 2{
            checking = 2;
        } else if i.len() == 1 {
            checking = 1;
        }

        let a = binoperate3(i).unwrap();
        for j in a{
            result.push(matchingb64(j));
        }
    }
    println!("{:?}", result);
    let mut returnstring = String::from_utf8(result).expect("Invalid");
    match checking {
        1 => format!("{}==",returnstring),
        2 => format!("{}=",returnstring),
        _ => returnstring,
    }
}

#[inline]
pub fn binoperate3(inp: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>{
    match inp.len() {
        3 => {
            let res = format!("{:08b}{:08b}{:08b}", inp[0], inp[1], inp[2]);
            let newbase64 = res.as_bytes().chunks(6).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();
            let newbase64s:Vec<u8> = newbase64.iter().map(|&b| u8::from_str_radix(b, 2).unwrap()).collect();
            Ok(newbase64s)
        },
        2 => {
            let res = format!("{:08b}{:08b}", inp[0], inp[1]);
            let mut newbase64 = res.as_bytes().chunks(6).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();
            let mut adjust = newbase64[newbase64.len() - 1].to_string();

            if let Some(last_element) = newbase64.last_mut() {
            while adjust.len() < 6 {
                adjust.push('0');
            }
            *last_element = &adjust;
            }
                 
            let newbase64s:Vec<u8> = newbase64.iter().map(|&b| u8::from_str_radix(b, 2).unwrap()).collect();
            Ok(newbase64s)
        },
        1 => {
            let res = format!("{:08b}", inp[0]);
            let mut newbase64 = res.as_bytes().chunks(6).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();
            let mut adjust = newbase64[newbase64.len() - 1].to_string();

            if let Some(last_element) = newbase64.last_mut() {
            while adjust.len() < 6 {
                adjust.push('0');
            }
            *last_element = &adjust;
            }
            
            let newbase64s:Vec<u8> = newbase64.iter().map(|&b| u8::from_str_radix(b, 2).unwrap()).collect();
            Ok(newbase64s)
        },
        _ => Err("Invalid".into()),

    }
}

// //ANCHOR - Decoding function from base64
//NOTE - iter from the first char then if it found = then it will stop

// pub fn frombase64(input: &str) -> Result<String, Box<dyn Error>> {
//     // Remove padding characters
//     let input = input.trim_end_matches('=');

//     // Split input into 6-bit chunks
//     let chunks: Vec<&str> = input.chars().collect::<Vec<char>>().chunks(6).map(|c| c.iter().collect()).collect();

//     // Convert 6-bit chunks to 8-bit bytes
//     let bytes: Vec<u8> = chunks.into_iter()
//         .map(|chunk| {
//             let num = u8::from_str_radix(chunk, 2)?;
//             Ok(num << 2) // Shift left by 2 bits (to convert 6 bits to 8 bits)
//         })
//         .collect::Result<Vec<u8>,Box<Error>>?;

//     // Decode percent encoding
//     let decoded_bytes = decoding_percent(&String::from_utf8(bytes)?);

//     Ok(decoded_bytes)
// }

fn frombase64(inp: &str) -> Option<String> {
    let rmpad = inp.trim_end_matches('=');
    let numpad = inp.chars().filter(|&c| c == '=').count();
    let mut buffer = Vec::new();
    for i in rmpad.as_bytes() {
        let rev1 = matchingb642(*i).unwrap();
        buffer.push(rev1);
    }

    let mut decoded_bytes = Vec::new();
    for i in 0..buffer.len() {
        match i % 4 {
            0 => decoded_bytes.push((buffer[i] << 2) | (buffer[i + 1] >> 4)),
            1 => decoded_bytes.push(((buffer[i] & 0b1111) << 4) | (buffer[i + 1] >> 2)),
            2 => decoded_bytes.push(((buffer[i] & 0b11) << 6) | buffer[i + 1]),
            _ => (),
        }
    }
    String::from_utf8(decoded_bytes).ok()
}
//     for &byte in input.as_bytes() {
//         if byte == b'=' {
//             break; // Padding character, stop decoding
//         }
        
//         if let Some(&value) = char_to_value.get(byte as usize) {
//             if value != -1 {
//                 buffer = (buffer << 6) | (value as u32);
//                 buffer_size += 6;

//                 while buffer_size >= 8 {
//                     buffer_size -= 8;
//                     result.push(((buffer >> buffer_size) & 0xFF) as u8);
//                 }
//             } else {
//                 return None; // Invalid Base64 character
//             }
//         } else {
//             return None; // Invalid input character
//         }
//     }

//     // Check for leftover bits in the buffer
//     if buffer_size >= 8 {
//         return None; // Invalid Base64 encoding
//     }

//     // Convert the decoded bytes to a UTF-8 string
//     match String::from_utf8(result) {
//         Ok(decoded_string) => Some(decoded_string),
//         Err(_) => None, // Invalid UTF-8 sequence
//     }
// }


//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Unit test

#[test]
fn testopenfile(){
    let filecontain = readfile("tests\\inputs\\rustdoc.txt").unwrap();
    let a = encoding_percent(&filecontain);
    println!("\n{}\n{}\n",filecontain,a);

}

//ANCHOR - Testing
#[test]
fn testtobase64(){
    let res = tobase64("Hdfdfds");
    // println!("\n{}",res);
    // let res1 = tobase64("df");
    // println!("{}",res1);
    // let res2 = tobase64("d");
    // println!("{}",res2);
    let a = frombase64(&res);
    // println!("{}",a.unwrap());
}

#[test]
fn testingfunctionhexoperating() {
    let space = b' '; //in ascii is 32 hex is 20
    let spacehigh = space >> 4; //bitwise it to get 2 from hex
    let spacelow = space & 0xF; //get 0 from hex
    //hex number of space is 20
    
    //make 2 and 0 to ascii the result will be 50 and 48
    assert_eq!(50,binoperate1(spacehigh));
    assert_eq!(48,binoperate1(spacelow));

}


#[test]
fn misc() {
    let a = "this that";
    println!("{}",encoding_percent(a));

    let b = "<<ƒƒƒ ““““";
    println!("{}", encoding_percent(b));
    
    let url = "https://doc.rust-lang.org/std/option/enum.Option.html";
    let encodedurl = encoding_percent(url);
    let decodedurl = decoding_percent(&encodedurl);

    assert_eq!(url,decodedurl);
    
    let b = "Hi".as_bytes();
    let iter:Vec<u8> = b.iter().map(|x| x>>2).collect();
    println!("{:?}",iter);
}

// #[test]
// fn misc() {
//     // assert_eq!("pureascii", encode("pureascii"));
//     // assert_eq!("", encode(""));
//     // assert_eq!("%00", encode("\0"));
//     assert_eq!("Hello%20G%C3%BCnter", encode("Hello Günter"));
//     println!("{}", encode("https://stackoverflow.com/questions/19076719/how-do-i-convert-a-vector-of-bytes-u8-to-a-string"));
// }
