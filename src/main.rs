#[macro_use]
extern crate lazy_static;

use std::env;
use std::process;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;

#[derive(PartialEq,Debug, Eq, Hash,Clone,Copy)]
enum Case{
    Upper,
    Lower,
}

enum Type{
    String(String),
    Number(f64),
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
enum Operation{
    Print,
    Add,
    Sub,
    Mul,
    Div,
    Var,
    Branch,
    Label,
}

#[derive(PartialEq, Eq, Hash)]
struct TokenMapIndex (usize,Case);

lazy_static!{
    static ref TokenOperationMapping:HashMap<TokenMapIndex,Operation> = HashMap::from([
        (TokenMapIndex(1,Case::Upper), Operation::Print),

        (TokenMapIndex(2,Case::Upper),Operation::Add),
        (TokenMapIndex(2,Case::Lower),Operation::Sub),

        (TokenMapIndex(3,Case::Upper),Operation::Mul),
        (TokenMapIndex(3,Case::Lower),Operation::Div),

        (TokenMapIndex(4,Case::Upper),Operation::Var),
        (TokenMapIndex(4,Case::Lower),Operation::Var),

        (TokenMapIndex(5,Case::Upper),Operation::Branch),
        (TokenMapIndex(5,Case::Lower),Operation::Branch),

        (TokenMapIndex(6,Case::Upper),Operation::Label),
        (TokenMapIndex(6,Case::Lower),Operation::Label),
    ]);
}

lazy_static!{
    static ref NumberStringNumberMap:HashMap<&'static str,i32> = HashMap::from([
     ("zero",0),
     ("one",1),
     ("two",2),
     ("three",3),
     ("four",4),
     ("five",5),
     ("six",6),
     ("seven",7),
     ("eight",8),
     ("nine",9),
    ]);
}


struct Variable{
    name: String,
    data: Type,
}

#[derive(Debug)]
struct Token{
    op: Operation,
    nargs: usize,
    case: Case,
    args: Vec<String>
}

#[derive(PartialEq, Eq, Debug)]
enum LineParseErrorTypes{
    WrongArgCount,
    UnknownOperation,
    NoOpcodeProvided,
    CouldntParseOpcode,
}

#[derive(Debug,PartialEq, Eq)]
struct LineParseError{
    typ:LineParseErrorTypes,
    msg: &'static str,
}

// Expects **one** line
// This returns a result with a token or error.
// The parent will, if an error occurs, print it together with the line
//  the error is on (and the err msg)

fn tokenize_text_code(code:&str) -> Result<Token,LineParseError>{
    // Strip whitspaced at front and back (so you can use intend.)
    let code = code.trim();

    let mut line_words:Vec<&str> = code.split(" ").collect();
    
    // Getting the operation based on case + len of first "mot"
    let first_word = match line_words.first(){
        Some(n) => n,
        None => {return Err(LineParseError{typ:LineParseErrorTypes::NoOpcodeProvided,msg:"No OpCode provided."})},
    };

    let first_word_case = match first_word.chars().next(){
        Some(n) => {
            if n.is_uppercase(){
                Case::Upper
            }else{
                Case::Lower
            }
        },
        None => {return Err(LineParseError{typ:LineParseErrorTypes::CouldntParseOpcode,msg:"OpCode couldn't be parsed (check spaces)"})}
    };

    let operation = *match TokenOperationMapping.get(&TokenMapIndex(first_word.chars().count(),first_word_case)){
        Some(n) => n,
        None => {return Err(LineParseError{typ:LineParseErrorTypes::UnknownOperation,msg:"Provided Operation is invalid."})},
    };

    line_words.remove(0);
    // Converting the Vec<&str> to a Vec<String>
    let string_line_words: Vec<String> = line_words.iter().map(|s| String::from(*s)).collect();

    // And finally building a token
    let token:Token = Token{op:operation,case:first_word_case,nargs:line_words.len(),args:string_line_words};

    Ok(token)
}


static STATEMENT_SEP: &str = ".";

fn process_mt_file(filename:&str){
    let mut code_tokens: Vec<Token> = Vec::new();

    // Read in filename
    let path = Path::new(filename);

    // Reading it into memory
    let mut file = match File::open(path){
        Err(err) => panic!("Couldn't open file: {}",err),
        Ok(f) => f
    };

    let mut content = String::new();
    match file.read_to_string(&mut content){
        Err(e) => panic!("Error reading file contents: {}",e),
        Ok(_) => ()
    }

    // Split string on STATEMENT_SEP
    let seperated_strings:Vec<&str> = content.split(STATEMENT_SEP).collect();

    for line in seperated_strings{
        tokenize_text_code(line);
    }

}

fn get_file_parse(){
    let commandline_args: Vec<String> = env::args().collect();
    if commandline_args.len() < 2{
        println!("Didn't provide the source file to run.");
        process::exit(1);
    }

    let filename_to_run:String = commandline_args[1].clone(); 
    process_mt_file(&filename_to_run);
}


fn primitive_test(){
    let example_program: &str = "aaaaaaaaaa";
    let token: Result<Token,LineParseError> = tokenize_text_code(example_program);
    println!("Token: {:?}",token);
}


fn main() {
    primitive_test();
    println!("Program is done.");
}

// ################
// #   Tests      #
// ################


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn invalid_opcode(){
        let example_program: &str = "invalidopcode";
        let token: Result<Token,LineParseError> = tokenize_text_code(example_program);
        match token{
            Ok(_) => {panic!()},
            Err(e) => {
                 assert_eq!(e.typ , LineParseErrorTypes::UnknownOperation);
                }
        }
    }
    #[test]
    fn print_opcode(){
        let example_program: &str = "  T alpha beta gamma  ";
        let token: Result<Token,LineParseError> = tokenize_text_code(example_program);
        match token{
            Err(e) =>{
                panic!();
            },
            Ok(t) => {
                assert_eq!(t.op, Operation::Print);
                assert_eq!(t.case,Case::Upper);
                assert_eq!(t.nargs,3);
                assert_eq!(t.args,vec!["alpha","beta","gamma"]);
            }
        }
    }

    #[test]
    fn add_opcode(){
        let ex1: &str = "Bd";
        let tok1: Token = tokenize_text_code(ex1).unwrap();

        let ex2: &str = " GZ hey duh";
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Add);
        assert_eq!(tok2.op, Operation::Add);
        
        assert_eq!(tok1.nargs,0);
        assert_eq!(tok2.nargs,2);
    }

    #[test]
    fn sub_opcode(){
        let ex1: &str = "du";
        let tok1: Token = tokenize_text_code(ex1).unwrap();

        let ex2: &str = " bU hey duh";
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op, Operation::Sub);
        assert_eq!(tok2.op, Operation::Sub);

        assert_eq!(tok1.nargs,0);
        assert_eq!(tok2.nargs,2);
    }

    #[test]
    fn mul_opcode(){
        let ex: &str = "Hey you and me";
        let tok: Token = tokenize_text_code(ex).unwrap();
        assert_eq!(tok.op,Operation::Mul);
        assert_eq!(tok.nargs,3);
    }

    #[test]
    fn div_opcode(){
        let ex: &str = "all I want";
        let tok: Token = tokenize_text_code(ex).unwrap();
        assert_eq!(tok.op,Operation::Div);
        assert_eq!(tok.nargs,2);
    }

    #[test]
    fn var_opcode(){
        // Testing if both upper and lowercase result in var
        let ex1: &str = "Ball one";
        let ex2: &str = "hell zero";

        let tok1: Token = tokenize_text_code(ex1).unwrap();
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op,Operation::Var);
        assert_eq!(tok2.op,Operation::Var);

        assert_eq!(tok1.nargs,1);
        assert_eq!(tok2.nargs,1);
    }

    #[test]
    fn branch_opcode(){
        // Testing if both upper and lowercase result in var
        let ex1: &str = "GHIJK";
        let ex2: &str = "abcde";

        let tok1: Token = tokenize_text_code(ex1).unwrap();
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op,Operation::Branch);
        assert_eq!(tok2.op,Operation::Branch);

        assert_eq!(tok1.nargs,0);
        assert_eq!(tok2.nargs,0);
    }

    #[test]
    fn label_opcode(){
        // Testing if both upper and lowercase result in var
        let ex1: &str = "GHklKh";
        let ex2: &str = "abcdeE";

        let tok1: Token = tokenize_text_code(ex1).unwrap();
        let tok2: Token = tokenize_text_code(ex2).unwrap();

        assert_eq!(tok1.op,Operation::Label);
        assert_eq!(tok2.op,Operation::Label);

        assert_eq!(tok1.nargs,0);
        assert_eq!(tok2.nargs,0);
    }

}