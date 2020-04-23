extern crate regex;
use crate::environment::logger::Logger;
use crate::text_processing::ast::types::ArgumentGroup::FuncGroup;
use crate::text_processing::ast::types::{
    ArgumentGroup, BinaryExpr, DataType, DataVar, FuncType, UnaryFuncExpr, Util,
};
use std::borrow::Borrow;
use std::collections::{LinkedList, VecDeque};
use std::iter::FromIterator;
use std::str::from_utf8;

// rule for parse  DSL-line from string
// it's struct promotes pipeline logic for create UnaryFuncExpression
// please, see next schedule:
// UnaryFuncExpression = get_func_type + get_channels + get_expressions
pub struct Rule;

impl Rule {
    // helping to split line on string group for next processing
    fn split_on_raw_group<T: Into<String>>(line: T) -> Vec<String> {
        let mut line: String = line.into();
        let mut capture_mode: bool = true;
        line.retain(|e: char| {
            if e == '\'' {
                capture_mode = !capture_mode
            }
            if capture_mode {
                !e.is_whitespace()
            } else {
                true
            }
        });
        let lines: Vec<&[u8]> = line
            .as_bytes()
            .split(|elem| *elem == b'\n' || *elem == b';')
            .collect();

        let mut accum: Vec<Vec<&[u8]>> = vec![vec![]];
        for line in lines {
            let mut capture_mode = true;
            let a: Vec<&[u8]> = line
                .split(|elem| {
                    if *elem == b'\'' {
                        capture_mode = !capture_mode
                    }
                    (*elem == b'(' || *elem == b')') && capture_mode == true
                })
                .collect();
            if !a.is_empty() {
                accum.push(a);
            }
        }
        let accum: Vec<String> = accum
            .iter()
            .flatten()
            .map(|e| from_utf8(e).unwrap().to_owned())
            .filter(|e| !e.is_empty())
            .collect();

        accum
    }
    // helping to split line on binary expression (used in get_expressions)
    fn split_expression(raw_expression: &str) -> Option<BinaryExpr> {
        fn split_on_data_type_and_operator(line: &str, substr: &str) -> Option<Vec<String>> {
            if line.contains(substr) {
                return Some(
                    line.split(substr)
                        .map(|e: &str| e.to_owned())
                        .collect::<Vec<String>>(),
                );
            }
            None
        }
        fn create_data_type(term: &String) -> Option<DataType> {
            DataType::from_string(term, &Util::identify_type(term))
        }
        let operators = vec!["==", "!=", ">=", "<=", ">", "<"];
        let mut binary_expression: Option<BinaryExpr> = None;
        'a: for operator in operators {
            match split_on_data_type_and_operator(raw_expression, operator) {
                Some(ref val) => {
                    binary_expression = Some(BinaryExpr::new(
                        create_data_type(&val[0]).unwrap(),
                        create_data_type(&val[1]).unwrap(),
                        operator.to_string(),
                    ));
                    break 'a;
                }
                None => binary_expression = None,
            };
        }
        if binary_expression.is_none() {
            Logger::error(
                format!("error parse in binary expression at: {}", raw_expression).as_str(),
            )
        }
        binary_expression
    }
    // help to split string value to data var todo: rewrite function to type safety DataVar -> Option<DataVar>
    fn split_statement(raw_statement: &str) -> DataVar {
        let val: Vec<&str> = raw_statement.splitn(2, ":").collect();
        let type_and_value: Vec<&str> = val[1].split("=").collect();

        if type_and_value.len() == 2 {
            let (symbol, raw_type, value) = (val[0], type_and_value[0], type_and_value[1]);
            let data_type = DataType::from_string(value, raw_type).expect(
                format!(
                    "DataType creation has been failed at {}.\n  value: {} type: {}",
                    symbol, type_and_value[0], type_and_value[1]
                )
                .as_str(),
            );
            DataVar::new(symbol.to_string(), data_type)
        } else {
            let (symbol, raw_type) = (val[0], type_and_value[0]);
            let data_type = DataType::from_type_default_value(raw_type).expect(
                format!(
                    "DataType creation has been failed at {}.\n value: {} type: {}",
                    symbol, type_and_value[0], type_and_value[0]
                )
                .as_str(),
            );
            DataVar::new(symbol.to_string(), data_type)
        }
    }

    pub fn get_argument_groups<T: Into<String>>(line: T) -> Vec<ArgumentGroup> {
        let collection = Rule::split_on_raw_group(line);

        collection
            .iter()
            .map(|e| ArgumentGroup::from_string(e))
            .fold(vec![], |mut acc, e| {
                if matches!(e, ArgumentGroup::FuncGroup(ref x)) {
                    acc.push(ArgumentGroup::None);
                }
                acc.push(e);
                acc
            })
    }

    pub fn get_func_type<T: ToString>(val: T) -> Option<FuncType> {
        FuncType::from_string(val.to_string())
    }

    pub fn get_channels<T: ToString>(val: T) -> Option<Vec<DataType>> {
        let val: String = val.to_string();
        let types: Vec<DataType> = val
            .split(',')
            .map(|e: &str| {
                DataType::from_string(e.to_owned(), Util::identify_type(&e.to_owned())).unwrap()
            })
            .collect();

        let imbalance = types.iter().find(|e| match e {
            DataType::Symbol(ref val) => false,
            _ => true,
        });
        if imbalance.is_none() {
            Some(types)
        } else {
            Logger::error(
                format!("channel values not correctly in: {:?}", imbalance.unwrap()).as_str(),
            );
            None
        }
    }

    pub fn get_expressions<T: ToString>(val: T) -> Option<Vec<BinaryExpr>> {
        let val: String = val.to_string();
        if val.is_empty() {
            return None;
        }
        Some(
            val.split(',')
                .map(|e| {
                    Rule::split_expression(e)
                        .expect(format!("Expression error in: {:?}", e).as_str())
                })
                .collect(),
        )
    }

    pub fn get_statements<T: ToString>(val: T) -> Option<Vec<DataVar>> {
        let val: String = val.to_string();
        val.split(',').map(Rule::split_statement).collect()
    }
}

trait Parser {
    fn from_unary_func_expr<T: Into<String>>(line: T) {
        let argument_super_group = Rule::get_argument_groups(line);
        let argument_groups: Vec<&[ArgumentGroup]> = argument_super_group
            .split(|e| matches!(e, ArgumentGroup::None))
            .filter(|e| !e.is_empty())
            .collect();

        for argument_subgroups in argument_groups {
            let func_type = Rule::get_func_type(&argument_subgroups[0].to_string())
                .expect("function type not found");
            match func_type {
                FuncType::OnCreate => {
                    // todo: rewrite because type not compatible (look get_expression signature)
                  /*  let channels = Rule::get_channels(&argument_subgroups[1].to_string())
                        .expect("channels not found");
                    let statements = Rule::get_statements(&argument_subgroups[2].to_string());
                    let unary_func_expr =
                        UnaryFuncExpr::new(func_type, channels, None, Some(statements));*/
                }
                FuncType::OnRead => {
                    let channels = argument_subgroups
                        .get(1)
                        .expect("channel not found")
                        .to_string();
                    let expressions = argument_subgroups
                        .get(2)
                        .unwrap_or(&ArgumentGroup::OtherGroup("".to_string()))
                        .to_string();
                    let channels = Rule::get_channels(channels).expect("channels parsing error");
                    let expressions = Rule::get_expressions(expressions);

                   let g = UnaryFuncExpr::new(func_type, channels, expressions, None);
                    println!("{:?}",g)
                }
                FuncType::OnUpdate => println!("onupdate"),
                FuncType::OnDelete => println!("ondelete"),
            }
        }

        //println!("{:?}", argument_groups)
    }
}

struct ParserDefault;
impl Parser for ParserDefault {}

mod test {
    use crate::text_processing::parser::states::{Parser, ParserDefault};

    #[test]
    fn test_from_unary_func_expr() -> Result<(), ()> {
        ParserDefault::from_unary_func_expr("onread(a>=2)\nonread(a==2)");
        Ok(())
    }
}
