#![allow(dead_code)]
#![allow(unused)]

mod stdlib;
use std::fs::File;
use std::io::{Error, Result, BufReader, Read};
use std::collections::HashMap;

enum Token {
    Function(String, Option<(usize, usize)>, Vec<String>),
    StringLit(String),
    Unknown()
}

enum Bracket {
    OpenParenthesis,
    CloseParenthesis,
    OpenCurlyBrace,
    CloseCurlyBrace,
}

impl Bracket {
    fn character(&self) -> char {
        match self {
            Bracket::OpenParenthesis => '(',
            Bracket::CloseParenthesis => ')',
            Bracket::OpenCurlyBrace => '{',
            Bracket::CloseCurlyBrace => '}',
        }
    }
}

struct Tokenizer {
    in_string: bool,
    function_table: HashMap<String, (usize, usize)>
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            in_string: false,
            function_table: HashMap::new()
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

fn parse_string_literal(string: &str) -> &str {
    let mut out = string;
    if out.starts_with("\"") || out.starts_with("'") {
        out = out.get(1..).expect("Failed to remove begining quote");
    }
    if out.ends_with("\"") || out.ends_with("'") {
        out = out.get(..out.len()-1).expect("Failed to remove ending quote");
    }
    out
}

fn get_function(chars: &Vec<char>, offset: usize) -> Result<(String, Option<(usize, usize)>)> {
    let mut output: String = String::new();
    let mut is_definition: bool = false; 
    let mut body_start: usize = 0;
    let mut body_end: usize = 0;

    if offset > 0 && chars[offset-1].is_ascii_alphabetic() {
        return Err(Error::other("Did not start at the beginning"));
    }

    if offset > 1 && chars[offset-2].is_ascii_alphabetic() {
        is_definition = true; 
    }


    for i in offset..chars.len() {
        let character: char = chars[i];

        if character == '(' {
            for j in i..chars.len() {
                if chars[j] == '{' && j < chars.len() {
                    body_start = j+1;
                    break;
                }
            }
            for k in body_start..chars.len() {
                if chars[k] == '}' {
                    body_end = k;
                    break;
                }
            }
            return match output.len() {
                0 => Err(Error::other("No function")),
                _ => Ok((output, match is_definition {
                    true => Some((body_start, body_end)),
                    false => None
                }))
            }
        }
        if !character.is_ascii_alphabetic() {
            return Err(Error::other("Invalid function"));
        }
        output.push(character);
    }

    Ok((output, match is_definition {
        true => Some((body_start, body_end)),
        false => None
    }))
}

// Gets the content inside a bracket with index deep
fn get_bracket(chars: &Vec<char>, index: isize, start: usize, open_bracket_type: Bracket, close_bracket_type: Bracket) -> Vec<char> {
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
        if chars[i] == open_bracket_type.character() {
            if current_index == index {
                is_inbracket = true;
            }
            current_index += 1;

            if is_inbracket && is_firstbracket {
                is_firstbracket = false;
                continue;
            }
        }
        if chars[i] == close_bracket_type.character() {
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
    let bracket: Vec<char> = get_bracket(chars, 0, index, Bracket::OpenParenthesis, Bracket::CloseParenthesis);
    if bracket.len() == 0 {
        return Vec::new();
    }
    let bracketstr = bracket.iter().collect::<String>();
    bracketstr.split(",").map(|s| s.to_string()).collect()
}

fn tokenize(state: &mut Tokenizer, content: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = content.chars().collect();
    
    let mut line: usize = 0;

    let mut function_definition_indexes: Vec<usize> = Vec::new();

    let mut char_to_token_indices: HashMap<usize, usize> = HashMap::new();

    println!("Tokenizing");
    for (index, character) in chars.iter().enumerate() {
        if *character == '\n' {
            line += 1;
            continue;
        }

        let mut tokens_len_old = tokens.len();
        char_to_token_indices.insert(index, tokens.len());

        // println!("{}", check_word(&chars, "print", index));
        let func = get_function(&chars, index);
        if func.is_ok() {
            let func_info = func.unwrap(); // func_info.0 is the name of the function
                                           // func_info.1 is None if it is a call and Some(start
                                           // definition, end definition)
            let is_definition = func_info.1.is_some();
            let args = get_args(&chars, index);

            if is_definition {
                // state.function_table.insert(func_info.0.clone(), func_info.1.unwrap());
            }
            tokens.push(Token::Function(func_info.0, func_info.1, args));
        }

        if tokens_len_old != tokens.len() {
        }
        print!("{} tokens parsed            \r", tokens.len());
    }
    println!("");

    //second pass
    println!("Finding function definitions");
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::Function(name, info, args) => {
                if info.is_some() {
                    let loc = info.unwrap();
                    println!("Found {name}() definition {} {}", loc.0, loc.1);
                    state.function_table.insert(name.clone(), (index, char_to_token_indices.get(&loc.1).expect("Oops").clone()));
                    let func = char_to_token_indices.get(&index).expect("Oops").clone();
                }
            }
            _ => {}
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
    println!("Running {filename}");
    let file_content = read_file(filename)?;
    println!("File length {}", file_content.len());

    let mut state: Tokenizer = Tokenizer::new();
    let tokens = tokenize(&mut state, &*file_content)?;
    println!("\n\n");

    let mut ip: usize = 0;

    while ip < tokens.len() {
        let token = &tokens[ip];
        ip += 1;
        match token {
            Token::Function(name, info, args) => {
                let is_definition = info.is_some();
                // println!("Function {is_definition} {name} {:?}", args);
                if is_definition {
                    let loc_char = info.unwrap();
                    let function_str = file_content.get(loc_char.0..loc_char.1).unwrap();
                    let loc_token = state.function_table.get(name).expect("Oops");
                    // println!("{} {} {function}", loc.0, loc.1);
                    println!("Define {name} c{} c{}", loc_token.0, loc_token.1);
                    ip = loc_token.1-1; // Skip function
                } else if !is_definition {
                    println!("Call {name}");
                    if name == "print" && args.len() > 0 {
                        stdlib::io::print(parse_string_literal(args.iter().next().expect("print needs an argument")));
                    } else {
                        let errmsg = format!("Undefined function {name}");
                        let function = state.function_table.get(name).expect(&errmsg);
                        // ip = function.0-1;
                        // let tip = function.0-1;
                        // let t = &tokens[tip];
                        // println!("{tip} target name {}", match t {Token::Function(n, _, _) => n, _ => ""});
                        println!("{name} c{} {}", function.0, function.1);
                    }
                }
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
