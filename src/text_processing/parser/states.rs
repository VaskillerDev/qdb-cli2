use crate::environment::logger::Logger;
use crate::text_processing::ast::types::{
    ArgumentGroup, BinaryExpr, DataType, DataVar, FuncType, UnaryFuncExpr, Util,
};
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
    // help to split string value to data var
    fn split_statement(raw_statement: &str) -> Option<DataVar> {
        let val: Vec<&str> = raw_statement.splitn(2, ":").collect();
        let type_and_value: Vec<&str> = val[1].split("=").collect();

        let symbol = val.get(0);
        let raw_type = type_and_value.get(0);
        let value = type_and_value.get(1);

        if symbol.is_some() && raw_type.is_some() && value.is_some() {
            let (symbol, raw_type, value) = (symbol.unwrap(), raw_type.unwrap(), value.unwrap());
            let data_type = DataType::from_string(value, raw_type).expect(
                format!(
                    "DataType creation has been failed at {}.\n  value: {} type: {}",
                    symbol,
                    type_and_value.get(0).unwrap(),
                    type_and_value.get(1).unwrap()
                )
                .as_str(),
            );
            return Some(DataVar::new(symbol.to_string(), data_type));
        }

        if symbol.is_some() && raw_type.is_some() {
            let (symbol, raw_type) = (symbol.unwrap(), raw_type.unwrap());
            let data_type = DataType::from_type_default_value(raw_type).expect(
                format!(
                    "DataType creation has been failed at {}.\n value: {} type: {}",
                    symbol, type_and_value[0], type_and_value[0]
                )
                .as_str(),
            );
            return Some(DataVar::new(symbol.to_string(), data_type));
        };

        None
    }

    pub fn get_argument_groups<T: Into<String>>(line: T) -> Vec<ArgumentGroup> {
        let collection = Rule::split_on_raw_group(line);

        collection
            .iter()
            .map(|e| ArgumentGroup::from_string(e))
            .fold(vec![], |mut acc, e| {
                if matches!(e, ArgumentGroup::FuncGroup(ref _x)) {
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
            DataType::Symbol(ref _val) => false,
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
        if val.is_empty() {
            return None;
        }
        Some(
            val.split(',')
                .map(|e| {
                    Rule::split_statement(e).expect(format!("Statement error in: {:?}", e).as_str())
                })
                .collect(),
        )
    }
}

trait Parser {
    fn from_unary_func_expr<T: Into<String>>(line: T) -> Option<Vec<UnaryFuncExpr>> {
        let argument_super_group = Rule::get_argument_groups(line);
        let argument_groups: Vec<&[ArgumentGroup]> = argument_super_group
            .split(|e| matches!(e, ArgumentGroup::None))
            .filter(|e| !e.is_empty())
            .collect();
        let mut unary_func_expressions: Vec<UnaryFuncExpr> = vec![];

        for argument_subgroups in argument_groups {
            let func_type = Rule::get_func_type(&argument_subgroups[0].to_string())
                .expect("function type not found");
            let channels = argument_subgroups
                .get(1)
                .expect("channel not found")
                .to_string();
            let channels = Rule::get_channels(channels).expect("channels parsing error");

            match func_type {
                FuncType::OnCreate => {
                    // func_type : Y, channels: Y, expressions: N, statements: Y
                    let statements = argument_subgroups
                        .get(2)
                        .unwrap_or(&ArgumentGroup::OtherGroup("".to_string()))
                        .to_string();
                    let statements = Rule::get_statements(statements);
                    let unary_func_expr = UnaryFuncExpr::new(func_type, channels, None, statements);
                    unary_func_expressions.push(unary_func_expr);
                }
                FuncType::OnRead => {
                    // func_type : Y, channels: Y, expressions: Y, statements: N
                    let expressions = argument_subgroups
                        .get(2)
                        .unwrap_or(&ArgumentGroup::OtherGroup("".to_string()))
                        .to_string();
                    let expressions = Rule::get_expressions(expressions);
                    let unary_func_expr =
                        UnaryFuncExpr::new(func_type, channels, expressions, None);
                    unary_func_expressions.push(unary_func_expr);
                }
                FuncType::OnUpdate => {
                    // func_type : Y, channels: Y, expressions: Y, statements: Y
                    let expressions = argument_subgroups
                        .get(2)
                        .unwrap_or(&ArgumentGroup::OtherGroup("".to_string()))
                        .to_string();
                    let expressions = Rule::get_expressions(expressions);
                    let statements = argument_subgroups
                        .get(3)
                        .unwrap_or(&ArgumentGroup::OtherGroup("".to_string()))
                        .to_string();
                    let statements = Rule::get_statements(statements);
                    let unary_func_expr =
                        UnaryFuncExpr::new(func_type, channels, expressions, statements);
                    unary_func_expressions.push(unary_func_expr);
                }
                FuncType::OnDelete => {
                    // func_type : Y, channels: Y, expressions: N, statements: N
                    let unary_func_expr = UnaryFuncExpr::new(func_type, channels, None, None);
                    unary_func_expressions.push(unary_func_expr);
                }
            }
        }
        if !unary_func_expressions.is_empty() {
            return Some(unary_func_expressions);
        }
        None
    }
}

struct ParserDefault;
impl ParserDefault {
    pub fn from_unary_func_expr_callback<
        T: Into<String>,
        F: FnOnce(Vec<UnaryFuncExpr>) -> Vec<UnaryFuncExpr>,
    >(
        line: T,
        closure: F,
    ) -> Vec<UnaryFuncExpr> {
        let line: String = line.into();
        let result: Vec<UnaryFuncExpr> =
            ParserDefault::from_unary_func_expr::<String>(line).expect("parser error");
        closure(result)
    }
}
impl Parser for ParserDefault {}

mod test {
    use crate::text_processing::parser::states::{Rule, ParserDefault};
    // todo: add more tests

    #[test]
    fn test_split_on_raw_group() -> Result<(), ()> {
        let result = Rule::split_on_raw_group("onCreate(my_channel)(a: int,b : text)");
        let test_vec = vec![
            "onCreate".to_string(),
            "my_channel".to_string(),
            "a:int,b:text".to_string(),
        ];
        assert_eq!(result, test_vec);
        let result = Rule::split_on_raw_group("onUpdate(my_channel)(a >= 2)(a : int, b : text)");
        let test_vec = vec![
            "onUpdate".to_string(),
            "my_channel".to_string(),
            "a>=2".to_string(),
            "a:int,b:text".to_string(),
        ];
        assert_eq!(result, test_vec);
        Ok(())
    }

    #[test]
    // proof of concept
    fn test_from_unary_func_expr() -> Result<(), ()> {
        use crate::text_processing::ast::types::DataType::Symbol;
        use crate::text_processing::ast::types::{BinaryExpr, DataType, FuncType, UnaryFuncExpr};
        use crate::text_processing::parser::states::{Parser, ParserDefault, Rule};
        assert_eq!(
            true,
            matches!(ParserDefault::from_unary_func_expr(" "), None)
        );
        assert_eq!(
            true,
            matches!(ParserDefault::from_unary_func_expr(""), None)
        );
        let unary_func_expressions =
            ParserDefault::from_unary_func_expr("onRead(vector)(x>=2);").unwrap();
        let unary_func_expression = unary_func_expressions.get(0).unwrap();
        let func_type = unary_func_expression.get_func_type();
        assert_eq!(true, matches!(FuncType::OnRead, func_type));
        let channels = unary_func_expression.get_channel_names();
        assert_eq!(
            true,
            matches!(DataType::Symbol("vector".to_string()), channels)
        );
        let exprs = unary_func_expression.get_binary_exprs().as_ref().unwrap();
        assert_eq!(
            true,
            matches!(
                [BinaryExpr::new(
                    Symbol("x".to_string()),
                    DataType::Int(2),
                    ">=".to_string()
                )],
                exprs
            )
        );

        Ok(())
    }

    #[test]
    fn test_from_unary_func_expr_callback() -> Result<(),()> {
       let a = ParserDefault::from_unary_func_expr_callback("onUpdate(my_channel)(x>=2)(a:int,b:real)",|elem|{
            println!("{:?}",elem);
            elem
        });
        Ok(())
    }
}
