#![allow(dead_code)]
#![allow(unused)]

mod stdlib;
use std::fs::File;
use std::io::{Error, Result, BufReader, Read};

enum Token {
    Function(&'static str, Vec<String>),
    StringLit(&'static str),
    Unknown()
}

struct Tokenizer {
    in_string: bool
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            in_string: false
        }
    }
}

fn check_word(chars: &Vec<char>, word: &str, index: usize) -> bool {
    if index + word.len() > chars.len() {
        return false;
    }
    let word_chars: Vec<char> = word.chars().collect();

    for i in index..chars.len() {
        let word_index = i - index;
        if (word_index >= word_chars.len()) {
            break;
        }
        if (chars[i] != word_chars[word_index]) {
            return false;
        }
    }
    true
}

fn get_function(chars: &Vec<char>, offset: usize) -> Result<String> {
    let mut output: String = String::new();
    let mut is_function: bool = true; 

    if offset > 0 && chars[offset-1].is_ascii_alphabetic() {
        return Err(Error::other("Did not start at the beginning"));
    }

    for i in offset..chars.len() {
        let character: char = chars[i];

        if character == '(' {
            return match output.len() {
                0 => Err(Error::other("No function")),
                _ => Ok(output)
            }
        }
        if !character.is_ascii_alphabetic() {
            return Err(Error::other("Invalid function"));
        }
        output.push(character);
    }

    Ok(output)
}

// Gets the content inside a bracket with index deep
fn get_bracket(chars: &Vec<char>, index: isize, start: usize) -> Vec<char> {
    let mut is_inbracket = false;
    let mut is_firstbracket = true;
    let mut in_string = false;
    let mut in_char = false;
    let mut current_index = 0;

    let mut out: Vec<char> = Vec::new();

    for i in start..chars.len() {
        if chars[i] == '"' {
            in_string = !in_string;
        }
        if chars[i] == '\'' {
            in_char = !in_char;
        }
        if chars[i] == '(' {
            if current_index == index {
                is_inbracket = true;
            }
            current_index += 1;

            if is_inbracket && is_firstbracket {
                is_firstbracket = false;
                continue;
            }
        }
        if chars[i] == ')' {
            if current_index == index+1 {
                break;
            }
            current_index -= 1;
        }
        if is_inbracket {
            if (chars[i] == ' ' || chars[i] == '\n') && (!in_string && !in_char) {
                continue;
            }
            out.push(chars[i]); 
        }
    }
    out
}

fn get_args(chars: &Vec<char>, index: usize) -> Vec<String> {
    let bracketstr = get_bracket(chars, 0, index).iter().collect::<String>();
    bracketstr.split(",").map(|s| s.to_string()).collect()
}

fn tokenize(content: &str) -> Result<Vec<Token>> {
    let state: Tokenizer = Tokenizer::new();
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    
    let mut line: usize = 0;

    for (index, character) in chars.iter().enumerate() {
        if *character == '\n' {
            line += 1;
            continue;
        }

        // println!("{}", check_word(&chars, "print", index));
        // if (check_word(&chars, "print", index) && !state.in_string) {
        //     let mut bracket: Vec<char> = get_bracket(&chars, 0);
        //     let bracket_str: String = bracket.iter().collect::<String>();
        //     let mut args = get_args(&chars);
        //     println!("{:?}", args);
        //     tokens.push(Token::Function("print", args));
        // }
        let func = get_function(&chars, index);
        if func.is_ok() {
            let func_name = func.unwrap();
            let args = get_args(&chars, index);
            println!("{func_name} {:?}", args);
        }
    }
    Ok(tokens)
}

fn read_file(filename: &str) -> Result<String> {
    let file = File::open(filename)?;
    let mut contents = String::new(); 
    let mut reader = BufReader::new(file);
    reader.read_to_string(&mut contents)?;
    return Ok(contents);
}

fn main() -> Result<()> {
    let filename = "examples/fib.zov";
    println!("Compiling {filename}");
    let file_content = read_file(filename)?;

    let tokens = tokenize(&*file_content)?;

    for token in &tokens {
        match token {
            Token::Function(name, args) => {
                println!("Function {name} {:?}", args);
            }
            Token::StringLit(_) => {
                println!("String literal");
            }
            Token::Unknown() => {
                println!("Syntax error!");
            }
        }
    }

    // stdlib::io::print(&file);

    Ok(())
}
