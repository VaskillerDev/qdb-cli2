extern crate regex;
use crate::environment::logger::Logger;
use crate::text_processing::ast::types::{FuncType, Util, DataType};
use regex::*;
use std::borrow::Borrow;

// It's not time yet for use
// maybe useless abstraction.
enum ParserState {
    Start,
    Done,
    Exception(String),
}

// It's use for parse string line with command.
// Implement ways form create abstract syntax tree from string-based values.
struct Parser;

// warning: using regexp - it's not good solution.
// exec time: ~ 129ms
// Implement on lexer-base solution in future release.
// TODO: implement
impl Parser {
    // translate string to UnaryFuncExpr
    pub fn from_unary_func_expr(line: String) {
        lazy_static! {
            static ref RE_UNARY_FUNC_EXPR_PARSER : Regex = Regex::new(
                r"(?P<func>on[\w\s]*)(?P<channels>\(['\w\s,]*\))(?P<expressions>\((['\s\w,]*(?:==|<|>|>=|<=|!=)[\s\w']*,?)+\))?(?P<statements>\([\w\s=':,]*\))?"
            ).unwrap();
            static ref RE_BINARY_EXPR_PARSER :Regex = Regex::new(
                r"(?P<lterm>['\w]+)[\s]*(?P<oprt>(?:>=|<=|>|<|==|!=))[\s]*(?P<rterm>['\w]+)"
            ).unwrap();
        }
        let res: Captures = RE_UNARY_FUNC_EXPR_PARSER.captures(line.as_str()).unwrap();

        // regexp names group:
        let func = &res.name("func");
        let channels = &res.name("channels");
        let expressions = &res.name("expressions");
        let statements = &res.name("statements");

        // extract fields from string
        match func {
            Some(val) => {
                let func_type = FuncType::from_string(val.as_str().to_lowercase());
                println!("{:?}", func_type) //TODO: continue
            }
            None => Logger::error("Function name not found."),
        }
        match channels {
            Some(val) => {
                let channels: Vec<&str> = val
                    .as_str()
                    .trim_matches('(')
                    .trim_matches(')')
                    .split(",")
                    .collect();
                let mut is_valid = true;

                'checker: for channel in &channels {
                    if !Util::is_single_word(channel.to_string()) {
                        Logger::error(("Channel name error in :".to_owned() + channel).as_ref());
                        is_valid = false;
                        break 'checker;
                    }
                }

                if is_valid {
                    let channels: Vec<String> =
                        channels.iter().map(|channel| channel.to_string()).collect();
                    println!("{:?}", &channels)
                }
            }
            None => Logger::error("Channels name not found."),
        }
        match expressions {
            Some(val) => {
                let expressions: Vec<&str> = val
                    .as_str()
                    .trim_matches('(')
                    .trim_matches(')')
                    .split(",")
                    .collect();

                for expression in expressions {
                    let res: Captures = RE_BINARY_EXPR_PARSER.captures(expression.trim()).unwrap();
                    let lterm = &res.name("lterm").unwrap().as_str();
                    let rterm = &res.name("rterm").unwrap().as_str();
                    let oprt = &res.name("oprt").unwrap().as_str();
                    //TODO: continue
                }
            }
            None => (),
        }
        match statements {
            Some(val) => {
                let statements: Vec<&str> = val
                    .as_str()
                    .trim_matches('(')
                    .trim_matches(')')
                    .split(",")
                    .collect();
                let statements: Vec<&str> = statements
                    .iter()
                    .map(|statement| statement.trim())
                    .collect();
                println!("{:?}", statements); //TODO: continue
            }
            None => (),
        }
    }
}

mod test {
    use crate::text_processing::parser::states::Parser;

    #[test]
    fn test_from_unary_func_expr() {
        Parser::from_unary_func_expr(
            "onCreate('my_node','you node','you_node')( id : int ,a : int = 2323,b : text = '2323' );"
                .to_string(),
        );
    }
    #[test]
    fn test_from_unary_func_expr_2() {
        Parser::from_unary_func_expr(
            "onUpdate('my_node','you_node','you_node')(a >= 2, b == 2)( id : int ,a : int = 2323,b : text = '2323' );"
                .to_string(),
        );
    }
}
