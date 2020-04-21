use crate::environment::logger::Logger;
use crate::text_processing::ast::types::FuncType::{OnCreate, OnDelete, OnRead, OnUpdate};
use regex::{Match, Regex};
use std::borrow::Borrow;

#[derive(Debug, PartialEq, PartialOrd)]
// data types
// example: 23 : int
pub enum DataType {
    // null value
    Null,
    // bool value
    Bool(bool),
    // integer value
    Int(i64),
    // real value
    Real(f64),
    // text value
    Text(String),
    // shadow value
    Symbol(String),
}

impl DataType {
    pub fn from<T>(raw_value: String) -> Option<T>
    where
        T: std::fmt::Display + std::str::FromStr,
    {
        let result: Result<T, T::Err> = raw_value.parse::<T>();
        match result {
            Ok(v) => Some(v),
            Err(e) => {
                let mes = format!(
                    "Type converted for value {} as {} is not correctly.",
                    raw_value,
                    std::any::type_name::<T>()
                );
                Logger::error(mes.as_str());
                None
            }
        }
    }

    pub fn from_string<T: ToString>(raw_value: T, raw_type: T) -> Option<DataType> {
        use super::types::DataType::*;
        use super::types_annotations::{BOOL, INT, NULL, REAL, SYMBOL, TEXT};

        let raw_value = raw_value.to_string().to_lowercase();
        let raw_type = raw_type.to_string().to_lowercase();

        match raw_type.as_str() {
            NULL => Some(Null),
            BOOL => Some(Bool(Self::from::<bool>(raw_value)?)),
            INT => Some(Int(Self::from::<i64>(raw_value)?)),
            REAL => Some(Real(Self::from::<f64>(raw_value)?)),
            TEXT => Some(Text(raw_value)),
            SYMBOL => Some(Symbol(raw_value)),
            _ => None,
        }
    }

    pub fn from_type_default_value<T: ToString>(raw_type: T) -> Option<DataType> {
        use super::types::DataType::*;
        use super::types_annotations::{BOOL, INT, NULL, REAL, TEXT};

        let raw_type = raw_type.to_string().to_lowercase();

        match raw_type.as_str() {
            NULL => Some(Null),
            BOOL => Some(Bool(false)),
            INT => Some(Int(0)),
            REAL => Some(Real(0.0)),
            TEXT => Some(Text("".to_string())),
            _ => None,
        }
    }
}

#[derive(Debug)]
// data variable - composition from data types
// example: <variable name> = 23 : int
pub struct DataVar(String, DataType);

impl DataVar {
    pub fn new(var_name: String, data_type: DataType) -> DataVar {
        DataVar(var_name, data_type)
    }
}

impl std::fmt::Display for DataVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, type: {:?}", self.0, self.1)
    }
}

#[derive(Debug)]
// function types, it's can use for Expr struct
// example: onCreate
pub enum FuncType {
    // onCreate value
    OnCreate,
    // onRead value
    OnRead,
    // onUpdate
    OnUpdate,
    // onDelete
    OnDelete,
}

impl FuncType {
    pub fn from_string(func_type: String) -> Option<FuncType> {
        use crate::text_processing::ast::types_annotations::{
            ONCREATE, ONDELETE, ONREAD, ONUPDATE,
        };
        let raw_type = func_type.to_string().to_lowercase();

        match raw_type.as_str() {
            // for create channel
            ONCREATE => Some(OnCreate),
            //for read channel
            ONREAD => Some(OnRead),
            // for update node in channel
            ONUPDATE => Some(OnUpdate),
            // for delete node from channel
            ONDELETE => Some(OnDelete),
            _ => None,
        }
    }
}

#[derive(Debug)]
// expressions for execution operation.
// It's composition from function types and data variable
// (look enum FuncType and struct DataVar )
// example: Expr equal to onCreate(a : int, b : bool)
pub struct UnaryFuncExpr {
    func_type: FuncType,
    channel_names: Vec<DataType>,
    binary_exprs: Option<Vec<BinaryExpr>>,
    vars: Option<Vec<DataVar>>,
}

impl UnaryFuncExpr {
    pub fn new(
        func_type: FuncType,
        channel_names: Vec<DataType>,
        binary_exprs: Option<Vec<BinaryExpr>>,
        vars: Option<Vec<DataVar>>,
    ) -> UnaryFuncExpr {
        UnaryFuncExpr {
            func_type,
            channel_names,
            binary_exprs,
            vars,
        }
    }
}

#[derive(Debug)]
pub enum ArgumentGroup {
    FuncGroup(String),
    ChannelsGroup(String),
    ExpressionsGroup(String),
    StatementsGroup(String),
    Other(String),
    None
}

impl ArgumentGroup {
    pub fn from_string(val: &String) -> ArgumentGroup {
        use crate::text_processing::ast::types_annotations::{
            ONCREATE, ONDELETE, ONREAD, ONUPDATE,
        };
        let val = val.to_lowercase();
        let val = val.as_str();
        if val == ONCREATE || val == ONREAD || val == ONUPDATE || val == ONDELETE {
            return ArgumentGroup::FuncGroup(val.to_string());
        }
        ArgumentGroup::Other(val.to_string())
    }
}

impl ToString for ArgumentGroup {
    fn to_string(&self) -> String {
        use crate::text_processing::ast::types::ArgumentGroup::{FuncGroup, Other};

        match self {
            FuncGroup(val) => val.to_owned(),
            Other(val) => val.to_owned(),
            _ => ()
        }
    }
}

impl std::fmt::Display for UnaryFuncExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "function type: {:?}", &self.func_type);
        write!(f, "channel names: {:?}", &self.channel_names);
        write!(f, "binary expressions: {:?}", &self.binary_exprs);
        write!(f, "vars: {:?}", &self.vars);
        Ok(())
    }
}

#[derive(Debug)]
// expressions for left-hand and right-hand data types
pub struct BinaryExpr(DataType, DataType, String);

impl BinaryExpr {
    pub fn new(lterm: DataType, rterm: DataType, operator: String) -> BinaryExpr {
        BinaryExpr(lterm, rterm, operator)
    }

    fn eq(&self) -> bool {
        self.0 == self.1
    }
    fn neq(&self) -> bool {
        self.0 != self.1
    }
    fn ge(&self) -> bool {
        self.0 >= self.1
    }
    fn gt(&self) -> bool {
        self.0 > self.1
    }
    fn le(&self) -> bool {
        self.0 <= self.1
    }
    fn lt(&self) -> bool {
        self.0 < self.1
    }

    pub fn compare(&self) -> Option<bool> {
        match self.2.as_str() {
            "==" => Some(self.eq()),
            "!=" => Some(self.neq()),
            ">=" => Some(self.ge()),
            ">" => Some(self.gt()),
            "<=" => Some(self.le()),
            "<" => Some(self.lt()),
            _ => None,
        }
    }
}

// template functions for shared code
pub struct Util;

impl Util {
    // detect one word in string
    pub fn is_single_word(var_name: String) -> bool {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(\b[A-Za-z_][A-Za-z0-9_]*\b)").unwrap();
        }
        let matches_len = RE
            .find_iter(var_name.as_str())
            .collect::<Vec<Match>>()
            .len();
        if matches_len == 1 {
            true
        } else {
            false
        }
    }

    // identify type from string value
    pub fn identify_type(term: &String) -> String {
        let term = term.chars().collect::<Vec<char>>();

        if term.iter().fold(true, |mut acc, e| acc && (e.is_numeric())) {
            return "int".to_string();
        };
        if term
            .iter()
            .fold(true, |mut acc, e| acc && (e.is_numeric() || e == &'.'))
        {
            return "real".to_string();
        };

        match term[..] {
            ['n', 'u', 'l', 'l'] => "null".to_string(),
            ['t', 'r', 'u', 'e'] | ['f', 'a', 'l', 's', 'e'] => "bool".to_string(),
            ['\'', .., '\''] => "text".to_string(),
            _ => "symbol".to_string(),
        }
    }
}

#[cfg(test)]
// test module
mod test {
    use crate::text_processing::ast::types::{BinaryExpr, DataType, Util};

    #[test]
    fn test_data_type_from_string() -> Result<(), ()> {
        assert_eq!(
            DataType::Null,
            DataType::from_string("NULL", "null").unwrap()
        );
        assert_eq!(
            DataType::Bool(true),
            DataType::from_string("true", "bool").unwrap()
        );
        assert_eq!(
            DataType::Int(32),
            DataType::from_string("32", "int").unwrap()
        );
        assert_eq!(
            DataType::Real(64.01),
            DataType::from_string("64.01", "real").unwrap()
        );
        assert_eq!(
            DataType::Real(32.0001),
            DataType::from_string("32.0001", "real").unwrap()
        );
        assert_eq!(
            DataType::Text("my test text".to_string()),
            DataType::from_string("my test text", "text").unwrap()
        );
        DataType::from_string("tru", "bool");

        Ok(())
    }

    #[test]
    fn test_is_single_word() -> Result<(), ()> {
        assert_eq!(true, Util::is_single_word("myvarexample".to_string()));
        assert_eq!(true, Util::is_single_word("myvar23varmy".to_string()));
        assert_eq!(true, Util::is_single_word("m".to_string()));
        assert_eq!(true, Util::is_single_word("  myvar23varmy".to_string()));

        assert_eq!(false, Util::is_single_word("5".to_string()));
        assert_eq!(
            false,
            Util::is_single_word("myvar exa23mple text".to_string())
        );
        assert_eq!(false, Util::is_single_word("2123example".to_string()));
        Ok(())
    }

    #[test]
    fn test_identify_type() -> Result<(), ()> {
        assert_eq!("null", Util::identify_type(&"null".to_string()));
        assert_eq!("text", Util::identify_type(&"'my string text'".to_string()));
        assert_eq!("int", Util::identify_type(&"28".to_string()));
        assert_eq!("symbol", Util::identify_type(&"my_var".to_string()));
        assert_eq!("real", Util::identify_type(&"32.0".to_string()));
        Ok(())
    }

    #[test]
    fn test_binary_expr_compare() -> Result<(), ()> {
        assert_eq!(
            true,
            BinaryExpr(
                DataType::Text("my text".to_string()),
                DataType::Text("my text".to_string()),
                "==".to_string()
            )
            .compare()
            .unwrap()
        );

        assert_eq!(
            true,
            BinaryExpr(
                DataType::Text("my text double".to_string()),
                DataType::Text("my text".to_string()),
                ">=".to_string()
            )
            .compare()
            .unwrap()
        );

        assert_eq!(
            true,
            BinaryExpr(
                DataType::Text("my text".to_string()),
                DataType::Text("my text double".to_string()),
                "<=".to_string()
            )
            .compare()
            .unwrap()
        );

        assert_eq!(
            true,
            BinaryExpr(DataType::Int(32), DataType::Real(32.0), "!=".to_string())
                .compare()
                .unwrap()
        );

        assert_eq!(
            true,
            BinaryExpr(DataType::Null, DataType::Null, "==".to_string())
                .compare()
                .unwrap()
        );

        assert_eq!(
            false,
            BinaryExpr(DataType::Bool(true), DataType::Null, "==".to_string())
                .compare()
                .unwrap()
        );

        assert_eq!(
            false,
            BinaryExpr(DataType::Int(32), DataType::Real(32.0), "==".to_string())
                .compare()
                .unwrap()
        );

        Ok(())
    }
}
