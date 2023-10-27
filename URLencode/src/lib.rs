use clap::error::Result;
use clap::{builder, Args, Parser, Subcommand};
use thiserror::Error;
use std::error::Error;
use std::fmt::{format, write};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write, SeekFrom};
use std::path::PathBuf;
use std::str;
use std::string::FromUtf8Error;


//ANCHOR - Clap section
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Urlencode {
    #[clap(subcommand)]
    pub inp: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    ///Encode from inputting URL
    #[clap(short_flag = 'e', about = "Encode input URL components")]
    Encode(Form1),
    ///Decode from inputting URL
    #[clap(short_flag = 'd', about = "Decode input URL components")]
    Decode(Form2),
}

#[derive(Debug, Args)]
pub struct Form1 {
    ///Input text, file or filepath
    #[clap(required = true, value_name = "TEXT/PATH")]
    pub filetext: StringOrPath,

    ///Percent-Encoding
    #[clap(
        short = 'p',
        long = "percent",
        value_name = "TO_PERCENT",
        conflicts_with = "flg2"
    )]
    pub flg1: bool,

    ///Base64-Encoding
    #[clap(short = 'b', long = "base64", value_name = "TO_BASE64")]
    pub flg2: bool,

    ///Getting result via table html format
    #[clap(long="tohtml",value_name="TO_HTML",conflicts_with= "flg4", alias="th", short='h')]
    pub flg3:bool,

    ///Getting result via terminal
    #[clap(long="toterminal",value_name="TO_TERMINAL",alias="tt",short='t')]
    pub flg4:bool,
}

#[derive(Debug, Args)]
pub struct Form2 {
    ///Input text, file or filepath
    #[clap(required = true, value_name = "TEXT/PATH")]
    pub filetext: StringOrPath,

    ///Percent-Decoding
    #[clap(short = 'p', long = "percent", value_name = "FROM_PERCENT")]
    pub flg1: bool,

    ///Base64-Decoding
    #[clap(short = 'b', long = "base64", value_name = "FROM_BASE64")]
    pub flg2: bool,
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Link function to flag

//ANCHOR - Impl Linking Encode to clap
impl Form1 {
    pub fn linking(&self) -> Result<FileContent, Box<dyn Error>> {
        match &self.filetext {
            StringOrPath::String(inp) => {
                // println!("{:?}",inp);
                if self.flg1 {
                    Ok(FileContent::Single(encoding_percent_component(&inp)))
                } else if self.flg2 {
                    Ok(FileContent::Single(tobase64(&inp))) //wiil change when to base64 is finish
                } else {
                    Ok(FileContent::Single(tobase64(&inp))) //for default choice
                }
            }
            StringOrPath::Path(path) => {
                let filecontent = readfile(&path.clone().into_os_string().into_string().unwrap())?;
                match filecontent {
                    FileContent::Multiple(lines) => {
                        if self.flg1 {
                            let mut processed_lines = Vec::new();
                            for i in lines {
                                let a = encoding_percent_component(&i);
                                processed_lines.push(a);
                            }
                            Ok(FileContent::Multiple(processed_lines))
                        } else if self.flg2 {
                            let mut processed_lines = Vec::new();
                            for i in lines {
                                let a = tobase64(&i);
                                processed_lines.push(a);
                            }
                            Ok(FileContent::Multiple(processed_lines))
                        } else {
                            let mut processed_lines = Vec::new();
                            for i in lines {
                                let a = encoding_percent_component(&i);
                                processed_lines.push(a);
                            }
                            Ok(FileContent::Multiple(processed_lines))
                        }
                    }
                    FileContent::Single(chr) => {
                        if self.flg1 {
                            Ok(FileContent::Single(encoding_percent_component(&chr)))
                        } else if self.flg2 {
                            Ok(FileContent::Single(tobase64(&chr)))
                        } else {
                            Ok(FileContent::Single(encoding_percent_component(&chr)))
                        }
                    }
                }
            }
        }
    }
}

//ANCHOR - Impl Linking Decode to clap
impl Form2 {
    pub fn linking(&self) -> Result<FileContent, Box<dyn Error>> {
        match &self.filetext {
            StringOrPath::String(inp) => {
                if self.flg1 {
                    Ok(FileContent::Single(decoding_percent(&inp)))
                } else if self.flg2 {
                    Ok(FileContent::Single(frombase64(&inp).unwrap())) //wiil change when to base64 is finish
                } else {
                    Ok(FileContent::Single(decoding_percent(&inp))) //for default choice
                }
            }
            StringOrPath::Path(path) => {
                let filecontent = readfile(&path.clone().into_os_string().into_string().unwrap())?;
                match filecontent {
                    FileContent::Multiple(lines) => {
                        if self.flg1 {
                            let mut processed_lines = Vec::new();
                            for i in lines {
                                let a = decoding_percent(&i);
                                processed_lines.push(a);
                            }
                            Ok(FileContent::Multiple(processed_lines))
                        } else if self.flg2 {
                            let mut processed_lines = Vec::new();
                            for i in lines {
                                let a = frombase64(&i).unwrap();
                                processed_lines.push(a);
                            }
                            Ok(FileContent::Multiple(processed_lines))
                        } else {
                            let mut processed_lines = Vec::new();
                            for i in lines {
                                let a = decoding_percent(&i);
                                processed_lines.push(a);
                            }
                            Ok(FileContent::Multiple(processed_lines))
                        }
                    }
                    FileContent::Single(chr) => {
                        if self.flg1 {
                            Ok(FileContent::Single(decoding_percent(&chr)))
                        } else if self.flg2 {
                            Ok(FileContent::Single(frombase64(&chr)?))
                        } else {
                            Ok(FileContent::Single(decoding_percent(&chr)))
                        }
                    }
                }
            }
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
//For reading from file and saving the result to html table

#[derive(Debug)]
pub enum FileContent {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug)]
pub enum FileLocate {
   Html,
   Terminal, 
}

#[derive(Error, Debug)]
pub struct ErrorToSaveFile {
    msg:String,
}

impl std::fmt::Display for ErrorToSaveFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl ErrorToSaveFile { //create my own error
    pub fn new(msg:&str) -> Self {
        ErrorToSaveFile { msg: msg.to_string() }
    }
}

//ANCHOR - Readfile
pub fn readfile(filename: &str) -> Result<FileContent, Box<dyn Error>> {
    let content = fs::read_to_string(filename)?;
    let lines: Vec<String> = content.lines().map(|f| f.to_string()).collect();

    match lines.len() {
        0 => Err("Empty".into()),
        1 => Ok(FileContent::Single(lines[0].clone())),
        _ => Ok(FileContent::Multiple(lines)),
    }
}

// //ANCHOR - Savefile
// pub fn savefile(filen:&str, wtop:FileContent, destinate: FileLocate) -> Result<(),ErrorToSaveFile> {
//     match wtop {
//         FileContent::Single(n) => {
//             match destinate {
//                 FileLocate::Html => {
//                     let mut outfile = File::create(filen).unwrap();
//                     outfile.write_all(b"<style>\ntable, td {{\n    border:1px solid black;\n}}\n").unwrap();
//                     outfile.write_all(b"</style>\n\n<table>\n    <tr>\n").unwrap();
//                     outfile.write_all(b"        <th>Input</th>\n        <th>Result</th>\n").unwrap();
//                     outfile.write_all(b"    </tr>").unwrap();
//                     outfile.write_all(b"    <tr>\n").unwrap();
//                     outfile.write_all(format!("        <td>{}</td>\n",)).unwrap();
//                     outfile.write_all(format!("        <td>{}</td>\n",n).as_bytes()).unwrap();
//                     outfile.write_all(b("    </tr>\n")).unwrap();
//                     outfile.write_all(b"</table>\n").unwrap();
//                 }
//                 FileLocate::Terminal => {
//                     Ok(println!("Result:{}",n)).unwrap()
//                 }
//             } 
//         },
//         FileContent::Multiple(st) => {

//         }
//     }
    
//     Err(ErrorToSaveFile::new("Error to save file {filen}.html"))
// }


//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Percent Encoding and Decoding

//ANCHOR - Encode Percent
pub fn encoding_percent_component(inp: &str) -> String {
    let mut buffer = Vec::new();
    for &i in inp.as_bytes() {
        match i {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'$'
            | b'-'
            | b'_'
            | b'.'
            | b'+'
            | b'!'
            | b'*'
            | b'('
            | b')' => buffer.push(i),
            _ => {
                buffer.push(b'%');
                buffer.push(binoperate1(i >> 4)); //for getting 4bits high| example pass i>>4 = 3
                buffer.push(binoperate1(i & 0xF)); //for getting 4bits low| example pass i & 0xF = 2
            }
        }
    }
    unsafe { String::from_utf8_unchecked(buffer) }
}

pub fn binoperate1(digit: u8) -> u8 {
    match digit {
        0..=9 => b'0' + digit,        //it start from 48 + ... it return a number
        10..=15 => b'A' + digit - 10, //start from 65 + ... - 10 it will return a char
        _ => panic!("Invalid input: {}", digit),
    }
}

//ANCHOR - Decode Percent
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

fn matchingb64(inp: u8) -> u8 {
    match inp {
        0..=25 => b'A' + inp,
        26..=51 => b'a' + (inp - 26),
        52..=61 => b'0' + (inp - 52),
        62 => b'+',
        63 => b'/',
        _ => panic!("Invalid"),
    }
}

fn matchingb642(inp: u8) -> Option<u8> {
    match inp {
        b'A'..=b'Z' => Some(inp - b'A'),
        b'a'..=b'z' => Some(inp - b'a' + 26),
        b'0'..=b'9' => Some(inp - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Base64 Encoding and Decoding

//ANCHOR - Encode Base64
pub fn tobase64(inp: &str) -> String {
    let iter: Vec<_> = inp.as_bytes().chunks(3).collect(); //type: [[u8,u8,u8]] (3 bytes)
    let mut result = Vec::new();
    let mut checking: u8 = 0;
    for i in iter {
        if i.len() == 2 {
            checking = 2;
        } else if i.len() == 1 {
            checking = 1;
        }

        let a = binoperate3(i).unwrap();
        for j in a {
            result.push(matchingb64(j));
        }
    }

    let returnstring = String::from_utf8(result).expect("Invalid");
    match checking {
        1 => format!("{}==", returnstring),
        2 => format!("{}=", returnstring),
        _ => returnstring,
    }
}

pub fn binoperate3(inp: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    //make it to binary format
    match inp.len() {
        3 => {
            let res = format!("{:08b}{:08b}{:08b}", inp[0], inp[1], inp[2]);
            let newbase64 = res
                .as_bytes()
                .chunks(6)
                .map(str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            let newbase64s: Vec<u8> = newbase64
                .iter()
                .map(|&b| u8::from_str_radix(b, 2).unwrap())
                .collect();
            Ok(newbase64s)
        }
        2 => {
            let res = format!("{:08b}{:08b}", inp[0], inp[1]);
            let mut newbase64 = res
                .as_bytes()
                .chunks(6)
                .map(str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            let mut adjust = newbase64[newbase64.len() - 1].to_string();

            if let Some(last_element) = newbase64.last_mut() {
                while adjust.len() < 6 {
                    adjust.push('0');
                }
                *last_element = &adjust;
            }

            let newbase64s: Vec<u8> = newbase64
                .iter()
                .map(|&b| u8::from_str_radix(b, 2).unwrap())
                .collect();
            Ok(newbase64s)
        }
        1 => {
            let res = format!("{:08b}", inp[0]);
            let mut newbase64 = res
                .as_bytes()
                .chunks(6)
                .map(str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();
            let mut adjust = newbase64[newbase64.len() - 1].to_string();

            if let Some(last_element) = newbase64.last_mut() {
                while adjust.len() < 6 {
                    adjust.push('0');
                }
                *last_element = &adjust;
            }

            let newbase64s: Vec<u8> = newbase64
                .iter()
                .map(|&b| u8::from_str_radix(b, 2).unwrap())
                .collect();
            Ok(newbase64s)
        }
        _ => Err("Invalid".into()),
    }
}

//ANCHOR - Decode Base64
//FIXME - Function not work
fn frombase64(inp: &str) -> Result<String, Box<dyn Error>> {
    let numpad = inp.chars().filter(|&c| c == '=').count(); //get num of = at the end
    let rmpad = inp.trim_end_matches('='); //remove = at the end
    let mut buffer = Vec::new();

    for i in rmpad.as_bytes() {
        let rev1 = matchingb642(*i).unwrap();
        buffer.push(rev1);
    }

    let mut decoded_bytes = Vec::new();
    for i in 0..buffer.len() {
        match i % 3 {
            0 => decoded_bytes.push((buffer[i] << 2) | (buffer[i + 1] >> 4)),
            1 => decoded_bytes.push(((buffer[i] & 0b1111) << 4) | (buffer[i + 1] >> 2)),
            2 => decoded_bytes.push(((buffer[i] & 0b11) << 6) | buffer[i + 1]),
            _ => (),
        }
    }
    String::from_utf8(decoded_bytes).map_err(|e| e.into())
}

//------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
//Unit test

//ANCHOR - Testing Function
#[test]
fn testopenfile() {
    let filecontain = readfile("tests\\inputs\\rustdoc.txt").unwrap();
    // let a = encoding_percent_component(&filecontain);
    println!("\n{:?}\n{}\n", filecontain, "ss");
}

#[test]
fn testtobase64() {
    let res = tobase64("Hdfdfds");
    let a = frombase64(&res);
    // println!("{}",a.unwrap());
}

#[test]
fn topercent() {
    let rres = "https://www.quora.com/How-do-I-encode-the-website-URL";
    let rres1 = "https://www.quora.com/How-do-I-encode-the-website-URL";
    assert_eq!(
        rres,
        "https://www.quora.com/How-do-I-encode-the-website-URLtest"
    );
    let enco = encoding_percent_component(rres1);
    print!("{}", enco);
}

#[test]
fn testingfunctionhexoperating() {
    let space = b' '; //in ascii is 32 hex is 20
    let spacehigh = space >> 4; //bitwise it to get 2 from hex
    let spacelow = space & 0xF; //get 0 from hex
                                //hex number of space is 20

    //make 2 and 0 to ascii the result will be 50 and 48
    assert_eq!(50, binoperate1(spacehigh));
    assert_eq!(48, binoperate1(spacelow));
}

#[test]
fn misc() {
    let a = "this that";
    // println!("{}",encoding_percent_component(a));

    let b = "<<ƒƒƒ ““““";
    // println!("{}", encoding_percent_component(b));

    let url = decoding_percent("https://www.freecodecamp.org/news/url-encoded-characters-reference/#:~:text=Any%20character%20that%20is%20not,used%20needs%20to%20be%20encoded.");
    let encodedurl = encoding_percent_component(&url);
    println!("{}", url);
    println!();
    println!("https://www.freecodecamp.org/news/url-encoded-characters-reference/#:~:text=Any%20character%20that%20is%20not,used%20needs%20to%20be%20encoded.");
    println!();
    println!("{}", encodedurl);

    // assert_eq!(url,decodedurl);

    let b = "Hi".as_bytes();
    let iter: Vec<u8> = b.iter().map(|x| x >> 2).collect();
    // println!("{:?}",iter);
}

// #[test]
// fn misc() {
//     // assert_eq!("pureascii", encode("pureascii"));
//     // assert_eq!("", encode(""));
//     // assert_eq!("%00", encode("\0"));
//     assert_eq!("Hello%20G%C3%BCnter", encode("Hello Günter"));
//     println!("{}", encode("https://stackoverflow.com/questions/19076719/how-do-i-convert-a-vector-of-bytes-u8-to-a-string"));
// }
// fn test(mut string: String) {
//     let test "4321=12412"

//     let iteartor =   string.split('=').nth(1).unwrap();
// }
