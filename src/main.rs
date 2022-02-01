#[macro_use]
extern crate lazy_static;

extern crate colored; // not needed in Rust 2018
use colored::*;

use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::path::Path;
use std::{env, fmt, process};
use text_io::read;

const DEBUG:bool = false;

#[derive(PartialEq, Debug, Eq, Hash, Clone, Copy)]
enum Case {
    Upper,
    Lower,
}

#[derive(PartialEq, PartialOrd, Clone)]
enum Type {
    String(String),
    Number(f64),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operation {
    Print,
    Input,
    Add,
    Sub,
    Mul,
    Div,
    Var,
    Branch,
    Label,
    Exit,
    Invalid,
}

#[derive(PartialEq, Eq, Hash)]
struct TokenMapIndex(usize, Case);

lazy_static! {
    static ref TOKEN_OPERATION_MAPPING: HashMap<TokenMapIndex, Operation> = HashMap::from([
        (TokenMapIndex(1, Case::Upper), Operation::Print),
        (TokenMapIndex(1,Case::Lower), Operation::Input),
        (TokenMapIndex(2, Case::Upper), Operation::Add),
        (TokenMapIndex(2, Case::Lower), Operation::Sub),
        (TokenMapIndex(3, Case::Upper), Operation::Mul),
        (TokenMapIndex(3, Case::Lower), Operation::Div),
        (TokenMapIndex(4, Case::Upper), Operation::Var),
        (TokenMapIndex(4, Case::Lower), Operation::Var),
        (TokenMapIndex(5, Case::Upper), Operation::Branch),
        (TokenMapIndex(5, Case::Lower), Operation::Branch),
        (TokenMapIndex(6, Case::Upper), Operation::Label),
        (TokenMapIndex(6, Case::Lower), Operation::Label),
    ]);
}

lazy_static! {
    static ref NUMBER_STRING_NUMBER_MAP: HashMap<String, i8> = HashMap::from([
        (String::from("zero"), 0),
        (String::from("one"), 1),
        (String::from("two"), 2),
        (String::from("three"), 3),
        (String::from("four"), 4),
        (String::from("five"), 5),
        (String::from("six"), 6),
        (String::from("seven"), 7),
        (String::from("eight"), 8),
        (String::from("nine"), 9),
    ]);
}

#[derive(Clone)]
struct Variable {
    // name: String,
    data: Type,
}

#[derive(Debug, Clone)]
struct Token {
    op: Operation,
    nargs: usize,
    name: String,
    // case: Case,
    args: Vec<String>,
}

#[derive(PartialEq, Eq, Debug)]
enum LineParseErrorTypes {
    UnknownOperation,
    NoOpcodeProvided,
    CouldntParseOpcode,
}

#[derive(Debug, PartialEq, Eq)]
struct LineParseError {
    typ: LineParseErrorTypes,
    msg: &'static str,
}

// Expects **one** line
// This returns a result with a token or error.
// The parent will, if an error occurs, print it together with the line
//  the error is on (and the err msg)
fn tokenize_text_code(code: &str) -> Result<Token, LineParseError> {
    // Strip whitspaced at front and back (so you can use intend.)
    let code = code.trim();

    let mut line_words: Vec<&str> = code.split(" ").collect();

    // Getting the operation based on case + len of first "mot"
    let first_word = match line_words.first() {
        Some(n) => n,
        None => {
            return Err(LineParseError {
                typ: LineParseErrorTypes::NoOpcodeProvided,
                msg: "No OpCode provided.",
            })
        }
    };

    let first_word_case = match first_word.chars().next() {
        Some(n) => {
            if n.is_uppercase() {
                Case::Upper
            } else {
                Case::Lower
            }
        }
        None => {
            return Err(LineParseError {
                typ: LineParseErrorTypes::CouldntParseOpcode,
                msg: "OpCode couldn't be parsed (check spaces)",
            })
        }
    };

    let operation = *match TOKEN_OPERATION_MAPPING
        .get(&TokenMapIndex(first_word.chars().count(), first_word_case))
    {
        Some(n) => n,
        None => {
            return Err(LineParseError {
                typ: LineParseErrorTypes::UnknownOperation,
                msg: "Provided Operation is invalid.",
            })
        }
    };

    let first_word: String = line_words[0].to_string();
    line_words.remove(0);
    // Converting the Vec<&str> to a Vec<String>
    let string_line_words: Vec<String> = line_words.iter().map(|s| String::from(*s)).collect();

    // And finally building a token
    let token: Token = Token {
        op: operation,
        // case: first_word_case,
        nargs: line_words.len(),
        args: string_line_words,
        name: first_word,
    };

    Ok(token)
}

fn print_compile_warning(token_num: usize, warn_str: &str) {
    println!(
        "{} on token {}: {}",
        "Warning".yellow(),
        token_num,
        warn_str
    );
}

fn print_compile_error(line: &str, error: LineParseError, line_num: usize) {
    println!(
        "{} `{}` on token {}: \"{}\" \n",
        "Error:".red(),
        error.msg,
        line_num,
        line,
    );
}

fn panic_generic_compile_error(token_number: usize, err_msg: &str) {
    println!(
        "{} `{}` on token {} \n",
        "Error:".red(),
        err_msg,
        token_number
    );
    println!(
        "{}",
        "The program terminated because of the above error.".red()
    );
    std::process::exit(1);
}

fn create_labels(tokens: Vec<Token>) -> HashMap<String, usize> {
    let mut labels: HashMap<String, usize> = HashMap::new();
    for (i, tok) in tokens.iter().enumerate() {
        if tok.op == Operation::Label {
            if tok.nargs > 0 {
                print_compile_warning(i, "You have a label with more than zero arguments.");
            }

            // Check if label exists
            match labels.get(&tok.name) {
                Some(_) => {
                    // FIXME: make this an *error*, not a warning
                    print_compile_warning(
                        i,
                        format!("You are defining the label `{}` more than once!", tok.name)
                            .as_str(),
                    );
                }
                None => {
                    labels.insert(tok.name.clone(), i);
                }
            }
        }
    }
    // And returning it
    labels
}

#[derive(Debug, PartialEq, Eq)]
enum NumberParseError {
    NoNumberProvided,
    InvalidNumberLiteral,
    DoubleComma,
}
impl std::fmt::Display for NumberParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// static mut current_parsed_line:usize = 0;

fn parse_text_number(text: Vec<String>) -> Result<f64, NumberParseError> {
    let is_negative: bool = match text.first() {
        Some(n) => n.to_lowercase() == "minus",
        None => {
            // Error, because not enough args were provided or just accepting? => Error
            return Err(NumberParseError::NoNumberProvided);
        }
    };

    // let mut is_comma = false;
    let mut is_comma_mode: bool = false;
    let mut comma_multiplier: i32 = 10;
    let mut parsed_number: f64 = 0.0;

    for (i, n_str) in text.iter().enumerate() {
        let n_str = n_str.to_lowercase();
        if i == 0 && n_str == "minus" {
            continue;
        }

        if n_str == "comma" {
            if is_comma_mode {
                return Err(NumberParseError::DoubleComma);
            }
            is_comma_mode = true;
        } else {
            let actual_number: i8 = match NUMBER_STRING_NUMBER_MAP.get(&n_str) {
                Some(n) => *n,
                None => return Err(NumberParseError::InvalidNumberLiteral),
            };

            if is_comma_mode {
                parsed_number += (actual_number as f64 / comma_multiplier as f64) as f64;
                comma_multiplier *= 10;
            } else {
                parsed_number *= 10.0;
                parsed_number += actual_number as f64;
            }
        }
    }

    if is_negative {
        parsed_number *= -1.0;
    }
    Ok(parsed_number)
}

#[derive(Debug)]
enum ArithmethicError {
    ZeroDivisionError,
    InvalidAmountArguments,
    VariableDoesNotExist,
    ArithmeticOnString,
    StoringToString,
}
impl fmt::Display for ArithmethicError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

fn simple_arithmetic_operation(
    t: Token,
    vars: &mut HashMap<String, Variable>,
    expr: fn(x: f64, y: f64) -> f64,
) -> Result<(), ArithmethicError> {
    // Check, that the token has exactly 3 arguments
    if t.nargs != 3 {
        return Err(ArithmethicError::InvalidAmountArguments);
    }

    let first_var_val: f64 = match vars.get(&t.args[0]) {
        Some(n) => match n.data {
            Type::Number(g) => g,
            Type::String(_) => {
                return Err(ArithmethicError::ArithmeticOnString);
            }
        },
        None => {
            return Err(ArithmethicError::VariableDoesNotExist);
        }
    };

    let second_var_val: f64 = match vars.get(&t.args[1]) {
        Some(n) => match n.data {
            Type::Number(g) => g,
            Type::String(_) => {
                return Err(ArithmethicError::ArithmeticOnString);
            }
        },
        None => {
            return Err(ArithmethicError::VariableDoesNotExist);
        }
    };

    let arithmetic_result: f64 = expr(first_var_val, second_var_val);

    // Storing it in the third given field (if not existent, will be created)
    match vars.get(&t.args[2]) {
        Some(n) => {
            // If it exists, check that type is Num and if so, store value
            match n.data {
                Type::Number(_) => {
                    vars.insert(
                        t.args[2].clone(),
                        Variable {
                            data: Type::Number(arithmetic_result),
                        },
                    );
                }
                Type::String(_) => {
                    return Err(ArithmethicError::StoringToString);
                }
            }
        }
        None => {
            // Create new var
            vars.insert(
                t.args[2].clone(),
                Variable {
                    data: Type::Number(arithmetic_result),
                },
            );
        }
    }

    Ok(())
}
#[derive(Debug)]
enum BranchError{
    VariableDoesNotExist,
    VarsNotOfSameType,
    InvalidComparisonForTypes,
}
impl fmt::Display for BranchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}


fn branch_condition_met(t:&Token,vars:&HashMap<String,Variable>,check: fn(x:Type,y:Type)->Result<bool,BranchError>) -> Result<bool,BranchError>{
    let first_elm: &Variable = match vars.get(&t.args[0]){
        Some(n) => n,
        None => {
            return Err(BranchError::VariableDoesNotExist);
        }
    };
    let second_elm: &Variable = match vars.get(&t.args[1]){
        Some(n) => n,
        None => {
            return Err(BranchError::VariableDoesNotExist);
        }
    };

    // Check if vars of same type
    if std::mem::discriminant(&first_elm.data) != std::mem::discriminant(&second_elm.data){
        return Err(BranchError::VarsNotOfSameType);
    }

    check(first_elm.data.clone(),second_elm.data.clone())
}

// Branch check functions
fn is_equal(x:Type,y:Type) -> Result<bool,BranchError>{
    // We know, that the types are the same
    let res: bool = match x{
        Type::Number(x_v) => {
            match y{
                Type::Number(y_v) => x_v == y_v,
                _ => {panic!("More ahhhh")}

                }
        },

        Type::String(x_v) => {
            match y{
                Type::String(y_v) => x_v == y_v,
                _ => {panic!("AHHHH")}
            }
        }
    };
    Ok(res)
}


fn is_less(x:Type,y:Type) -> Result<bool,BranchError>{
    // We know, that the types are the same
    let res: bool = match x{
        Type::Number(x_v) => {
            match y{
                Type::Number(y_v) => x_v < y_v, _ => {panic!("More ahhhh")}
                }
        },
        Type::String(_) => {
            match y{
                Type::String(_) => {
                    return Err(BranchError::InvalidComparisonForTypes);
                }, _ => {panic!("AHHHH")}
            }
        }
    };
    Ok(res)
}

fn is_greater(x:Type,y:Type) -> Result<bool,BranchError>{
    // We know, that the types are the same
    let res: bool = match x{
        Type::Number(x_v) => {
            match y{
                Type::Number(y_v) => x_v > y_v, _ => {panic!("More ahhhh")}
                }
        },
        Type::String(_) => {
            match y{
                Type::String(_) => {
                    return Err(BranchError::InvalidComparisonForTypes);
                }, _ => {panic!("AHHHH")}
            }
        }
    };
    Ok(res)
}


fn execute_code_tokens(tokens: Vec<Token>, labels: HashMap<String, usize>) {
    let mut ip: usize = 0;
    let mut variables: HashMap<String, Variable> = HashMap::new();

    // Adding pre-defined variables
    variables.insert(String::from("newl"),Variable{data:Type::String(String::from("\n"))});
    variables.insert(String::from("spce"),Variable{data:Type::String(String::from(" "))});
    variables.insert(String::from("dott"),Variable{data:Type::String(String::from("."))});


    loop {
        match tokens.get(ip) {
            Some(t) => {
                match t.op {
                    Operation::Add => {
                        match simple_arithmetic_operation(t.clone(), &mut variables, |x, y| x + y) {
                            Err(e) => {
                                panic_generic_compile_error(ip, &e.to_string());
                            }
                            _ => (),
                        }
                    }
                    Operation::Sub => {
                        match simple_arithmetic_operation(t.clone(), &mut variables, |x, y| x - y) {
                            Err(e) => {
                                panic_generic_compile_error(ip, &e.to_string());
                            }
                            _ => (),
                        }
                    }
                    Operation::Mul => {
                        match simple_arithmetic_operation(t.clone(), &mut variables, |x, y| x * y) {
                            Err(e) => {
                                panic_generic_compile_error(ip, &e.to_string());
                            }
                            _ => (),
                        }
                    }
                    Operation::Div => {
                        // Checking for zero division
                        match t.args.get(1) {
                            Some(n) => match variables.get(n) {
                                Some(g) => match g.data {
                                    Type::Number(h) => {
                                        if h == 0.0 {
                                            panic_generic_compile_error(
                                                ip,
                                                &ArithmethicError::ZeroDivisionError.to_string(),
                                            );
                                        }
                                    }
                                    Type::String(_) => panic_generic_compile_error(
                                        ip,
                                        &ArithmethicError::ArithmeticOnString.to_string(),
                                    ),
                                },
                                None => {
                                    panic_generic_compile_error(
                                        ip,
                                        &ArithmethicError::VariableDoesNotExist.to_string(),
                                    );
                                }
                            },
                            None => {
                                panic_generic_compile_error(
                                    ip,
                                    &ArithmethicError::InvalidAmountArguments.to_string(),
                                );
                            }
                        }
                        match simple_arithmetic_operation(t.clone(), &mut variables, |x, y| x / y) {
                            Err(e) => {
                                panic_generic_compile_error(ip, &e.to_string());
                            }
                            _ => (),
                        }
                    }
                    Operation::Print => {
                        let mut final_str = String::new();

                        // loop over args
                        for arg in &t.args {
                            match variables.get(&arg.clone()) {
                                Some(n) => match &n.data {
                                    Type::String(c) => {
                                        final_str.push_str(&c);
                                    }
                                    Type::Number(c) => {
                                        final_str.push_str(&c.to_string());
                                    }
                                },
                                None => {
                                    panic_generic_compile_error(
                                        ip,
                                        "Couldn't find var, you are trying to use.",
                                    );
                                }
                            }
                        }
                        println!("{}", final_str);
                    },
                    Operation::Input => {
                        // If first arg is Upper case, the result is a number, if Lowercase => String. newline is stripped either way.
                        if t.nargs != 2{
                            panic_generic_compile_error(ip,"Input needs exactly two args.");
                        }

                        let is_number:bool = match t.args[0].chars().next(){
                            Some(n) => n.is_ascii_uppercase(),
                            None => {panic!("str withouth any chars!")}
                        };

                        let user_input: String = read!("{}\n");

                        
                        if is_number{
                            match &user_input.parse::< f64 >(){
                                Ok(n) => {
                                    variables.insert(t.args[1].clone(),Variable{data:Type::Number((n).clone())});
                                },
                                Err(_) => {
                                    println!("The program expected a {}, which your input is *not*!","Number".red());
                                    return;
                                }
                            };
                            
                        }else{
                            variables.insert(t.args[1].clone(),Variable{data:Type::String(user_input)});
                        }                        
                    },
                    Operation::Branch => {
                        // Get label, check if valid, move ip there
                        

                        // Check for correct amount of args
                        if t.nargs != 3{
                            panic_generic_compile_error(ip,"Branch Opcode does not have exactly *3* arguments.");
                        }
                        
                        // Get first letter of name
                        let first_letter = match t.name.chars().next(){
                            Some(n)=> n,
                            None => {
                                panic_generic_compile_error(ip, "No characters in branch name.");
                                'b'
                            }
                        };

                        let first_letter = first_letter.to_ascii_lowercase();

                        let label_location:usize = match labels.get(&t.args[2]){
                            Some(n) => *n,
                            None => {
                                panic_generic_compile_error(ip, "Couldn't find label you are trying to jump to.");
                                0
                            }
                        };

                        // If branch starts with [E=> Equal, G => Greater than, L => Less than]
                        if first_letter == 'e'{
                            match branch_condition_met(t, &variables, is_equal){
                                Ok(n) => {
                                    if n{
                                        ip = label_location;
                                    }
                                },
                                Err(e) => {
                                    panic_generic_compile_error(ip,&e.to_string());
                                }
                            }
                        }else if first_letter == 'l'{
                            match branch_condition_met(t, &variables, is_less){
                                Ok(n) => {
                                    if n{
                                        ip = label_location;
                                    }
                                },
                                Err(e) => {
                                    panic_generic_compile_error(ip,&e.to_string());
                                }
                            }
                        }else if first_letter == 'g'{
                            match branch_condition_met(t, &variables, is_greater){
                                Ok(n) => {
                                    if n{
                                        ip = label_location;
                                    }
                                },
                                Err(e) => {
                                    panic_generic_compile_error(ip,&e.to_string());
                                }
                            }
                        }else{
                            panic_generic_compile_error(ip,"Branch command doesn't start with <e/l/g> (or uppercase version) and is invalid.")
                        }

                    }
                    Operation::Var => {
                        // If first arg is uppercase, the var is a number, else a string
                        let first_arg_is_uppercase: bool = match t.args.first() {
                            Some(n) => match n.chars().next() {
                                Some(n) => n.is_uppercase(),
                                None => panic!("string without any chars"),
                            },
                            None => {
                                panic_generic_compile_error(
                                    ip,
                                    "Var token is missing argument(s).",
                                );
                                false
                            }
                        };

                        if first_arg_is_uppercase {
                            // This means, that the variable stores a number TODO: get rid of clone
                            let value: f64 = match parse_text_number(t.args.clone()) {
                                Ok(n) => n,
                                Err(e) => {
                                    panic_generic_compile_error(ip, &e.to_string());
                                    -1.0
                                }
                            };
                            // Check if variable exists
                            match variables.get(&t.name) {
                                Some(n) => {
                                    // Check if data type is also number (will exit if error)
                                    match n.data {
                                        Type::String(_) => {
                                            panic_generic_compile_error(
                                                ip,
                                                "Changing type of variable from String to number",
                                            );
                                        }
                                        Type::Number(_) => (),
                                    }

                                    // If this passes, we can safely replace value of variable
                                    variables.insert(
                                        t.name.clone(),
                                        Variable {
                                            data: Type::Number(value),
                                        },
                                    );
                                }
                                None => {
                                    // Just insert TODO: make this not use `clone`
                                    variables.insert(
                                        t.name.clone(),
                                        Variable {
                                            data: Type::Number(value),
                                        },
                                    );
                                }
                            }
                        } else {
                            // Joining the string with spaces.
                            let arg: String = t.args.join(" ");

                            match variables.get(&t.name) {
                                Some(n) => {
                                    // Check if it is of type Number ( if so, panic )
                                    match n.data {
                                        Type::Number(_) => {
                                            panic_generic_compile_error(
                                                ip,
                                                "Changing type of variable from Number to String",
                                            );
                                        }
                                        Type::String(_) => {
                                            variables.insert(
                                                t.name.clone(),
                                                Variable {
                                                    data: Type::String(arg),
                                                },
                                            );
                                        }
                                    }
                                }
                                None => {
                                    // Just insert
                                    variables.insert(
                                        t.name.clone(),
                                        Variable {
                                            data: Type::String(arg),
                                        },
                                    );
                                }
                            }
                        }
                    }
                    Operation::Label => {
                        // Labels are just being skipped, since they have already been collected
                        ()
                    }
                    Operation::Exit => {
                        // Returning out of this function == exiting
                        return;
                    }
                    Operation::Invalid => {
                        // The invalid opcode does only exist, to be able to display all compile errors and
                        // not exit after the first one is found
                        panic!("Trying to execute 'Invalid' operation.");
                    }
                }
            }
            None => {
                panic!("Couldn't get token!");
            }
        }
        ip += 1;
    }
}

static STATEMENT_SEP: &str = ".";

fn process_mt_file(filename: &str) {
    let mut is_valid_code: bool = true;
    let exit_token: Token = Token {
        op: Operation::Exit,
        nargs: 0,
        args: vec![],
        name: String::from(""),
        // case: Case::Upper,
    };

    // Read in filename
    let path = Path::new(filename);

    // Reading it into memory
    let mut file = match File::open(path) {
        Err(err) => panic!("Couldn't open file: {}", err),
        Ok(f) => f,
    };

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(e) => panic!("Error reading file contents: {}", e),
        Ok(_) => (),
    }

    // Split string on STATEMENT_SEP
    let mut seperated_strings: Vec<&str> = content.split(STATEMENT_SEP).collect();
    // println!("sepearted string: {:?}",seperated_strings);

    // Since source code ends in ".", we have to strip away the last element.
    match seperated_strings.pop() {
        Some(n) => {
            if n.trim() != "" {
                seperated_strings.push(n);
                println!(
                    "{}: You forgot the dot in the last line of your code.",
                    "Warning".yellow()
                );
            }
        }
        None => (),
    }

    let mut tokens: Vec<Token> = Vec::new();
    for (index, line) in seperated_strings.iter().enumerate() {
        let tok = match tokenize_text_code(line) {
            Ok(t) => t,
            Err(e) => {
                is_valid_code = false;
                print_compile_error(line.trim(), e, index);

                Token {
                    op: Operation::Invalid,
                    // case: Case::Lower,
                    nargs: 0,
                    args: vec![],
                    name: String::from("Invalid!"),
                }
            }
        };

        tokens.push(tok);
    }
    if DEBUG{
        println!("Tokens: {:?}",tokens);
    }

    // Add "Exit" Token at end.
    tokens.push(exit_token);

    // creates an index of the used labels with their position (token index) in the code
    let code_labels: HashMap<String, usize> = create_labels(tokens.clone());

    // If false: print("code couldn't be compiled as a cause of the above errors")
    if is_valid_code {
        execute_code_tokens(tokens, code_labels);
    } else {
        println!(
            "{}",
            "Code can't run as a result of the above errors.".red()
        );
    }
}

fn get_file_parse() {
    let commandline_args: Vec<String> = env::args().collect();
    if commandline_args.len() < 2 {
        println!("{}", "Didn't provide the source file to run.".red());
        process::exit(1);
    }

    let filename_to_run: String = commandline_args[1].clone();
    process_mt_file(&filename_to_run);
}

fn main() {
    get_file_parse();
    // test_number_parsing();
    println!("{}", "Program is done.".green());
}

// ################
// #   Tests      #
// ################

#[cfg(test)]
mod tests {
    use super::*;

    // Testing the number-parse function
    #[test]
    fn number_parse_test() {
        // Testing normal, full number
        let args: Vec<String> = vec!["nine", "seven", "three"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let n: f64 = parse_text_number(args).unwrap();
        assert_eq!(n, 973.0);

        // Testing negative number
        let args: Vec<String> = vec!["minus", "seven", "three"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let n: f64 = parse_text_number(args).unwrap();
        assert_eq!(n, -73.0);

        // Testing comma number (with number in front of comma)
        let args: Vec<String> = vec!["seven", "comma", "three", "nine"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let n: f64 = parse_text_number(args).unwrap();
        assert_eq!(n, 7.39);

        // Testing comma number (with no number in front of comma)
        let args: Vec<String> = vec!["comma", "three", "nine"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let n: f64 = parse_text_number(args).unwrap();
        assert_eq!(n, 0.39);

        // Testing negative comma number
        let args: Vec<String> = vec!["minus", "six", "comma", "three", "nine"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let n: f64 = parse_text_number(args).unwrap();
        assert_eq!(n, -6.39);

        // Testing negative comma number with no number prefix
        let args: Vec<String> = vec!["minus", "comma", "three", "zero"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        let n: f64 = parse_text_number(args).unwrap();
        assert_eq!(n, -0.3);

        // ######################
        // #   Testing Errors   #
        // ######################

        // Testing noNumbersProvided Error
        let args: Vec<String> = vec![];
        match parse_text_number(args) {
            Ok(_) => {
                panic!()
            }
            Err(e) => {
                if e != NumberParseError::NoNumberProvided {
                    panic!();
                }
            }
        }

        // Testing InvalidNumber Error
        let args: Vec<String> = vec!["one", "two", "invalid", "zero"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        match parse_text_number(args) {
            Ok(_) => {
                panic!()
            }
            Err(e) => {
                if e != NumberParseError::InvalidNumberLiteral {
                    panic!();
                }
            }
        }

        // Testing DoubleComma Error
        let args: Vec<String> = vec!["one", "comma", "two", "comma", "four"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        match parse_text_number(args) {
            Ok(_) => {
                panic!()
            }
            Err(e) => {
                if e != NumberParseError::DoubleComma {
                    panic!();
                }
            }
        }
    }

    // TESTING ALL THE OPCODES
    #[test]
    fn invalid_opcode() {
        let example_program: &str = "invalidopcode";
        let token: Result<Token, LineParseError> = tokenize_text_code(example_program);
        match token {
            Ok(_) => {
                panic!()
            }
            Err(e) => {
                assert_eq!(e.typ, LineParseErrorTypes::UnknownOperation);
            }
        }
    }
    #[test]
    fn print_opcode() {
        let example_program: &str = "  T alpha beta gamma  ";
        let token: Result<Token, LineParseError> = tokenize_text_code(example_program);
        match token {
            Err(_) => {
                panic!();
            }
            Ok(t) => {
                assert_eq!(t.op, Operation::Print);
                // assert_eq!(t.case, Case::Upper);
                assert_eq!(t.nargs, 3);
                assert_eq!(t.args, vec!["alpha", "beta", "gamma"]);
            }
        }
    }
    #[test]
    fn add_opcode() {
        let ex1: &str = "Bd";
        let tok1: Token = tokenize_text_code(ex1).unwrap();

        let ex2: &str = " GZ hey duh";
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Add);
        assert_eq!(tok2.op, Operation::Add);

        assert_eq!(tok1.nargs, 0);
        assert_eq!(tok2.nargs, 2);
    }

    #[test]
    fn sub_opcode() {
        let ex1: &str = "du";
        let tok1: Token = tokenize_text_code(ex1).unwrap();

        let ex2: &str = " bU hey duh";
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Sub);
        assert_eq!(tok2.op, Operation::Sub);

        assert_eq!(tok1.nargs, 0);
        assert_eq!(tok2.nargs, 2);
    }

    #[test]
    fn mul_opcode() {
        let ex: &str = "Hey you and me";
        let tok: Token = tokenize_text_code(ex).unwrap();
        assert_eq!(tok.op, Operation::Mul);
        assert_eq!(tok.nargs, 3);
    }

    #[test]
    fn div_opcode() {
        let ex: &str = "all I want";
        let tok: Token = tokenize_text_code(ex).unwrap();
        assert_eq!(tok.op, Operation::Div);
        assert_eq!(tok.nargs, 2);
    }

    #[test]
    fn var_opcode() {
        // Testing if both upper and lowercase result in var
        let ex1: &str = "Ball one";
        let ex2: &str = "hell zero";

        let tok1: Token = tokenize_text_code(ex1).unwrap();
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Var);
        assert_eq!(tok2.op, Operation::Var);

        assert_eq!(tok1.nargs, 1);
        assert_eq!(tok2.nargs, 1);
    }

    #[test]
    fn branch_opcode() {
        // Testing if both upper and lowercase result in var
        let ex1: &str = "GHIJK";
        let ex2: &str = "abcde";

        let tok1: Token = tokenize_text_code(ex1).unwrap();
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Branch);
        assert_eq!(tok2.op, Operation::Branch);

        assert_eq!(tok1.nargs, 0);
        assert_eq!(tok2.nargs, 0);
    }

    #[test]
    fn label_opcode() {
        // Testing if both upper and lowercase result in var
        let ex1: &str = "GHklKh";
        let ex2: &str = "abcdeE";

        let tok1: Token = tokenize_text_code(ex1).unwrap();
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Label);
        assert_eq!(tok2.op, Operation::Label);

        assert_eq!(tok1.nargs, 0);
        assert_eq!(tok2.nargs, 0);
    }
}
