use super::super::script::ScriptError;
use super::var_type::VarType;

use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Write};
use std::ptr::hash;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum Variable {
    Bool(VarType, Option<bool>),
    String(VarType, Option<String>),
    Integer(VarType, Option<isize>),
    Float(VarType, Option<f64>),
    Char(VarType, Option<u8>),
    List(VarType, Option<Vec<Variable>>),
    Map(VarType, Option<HashMap<Variable, Variable>>),
    Optional(VarType, Option<Option<Box<Variable>>>),
    InStream(VarType, Option<Arc<Mutex<dyn Read>>>),
    OutStream(VarType, Option<Arc<Mutex<dyn Write>>>),
    Null(VarType),
}

impl Variable {
    pub fn get_type(&self) -> VarType {
        match self {
            Variable::Bool(t, _) => t.clone(),
            Variable::String(t, _) => t.clone(),
            Variable::Integer(t, _) => t.clone(),
            Variable::Float(t, _) => t.clone(),
            Variable::Char(t, _) => t.clone(),
            Variable::List(t, _) => t.clone(),
            Variable::Map(t, _) => t.clone(),
            Variable::Optional(t, _) => t.clone(),
            Variable::InStream(t, _) => t.clone(),
            Variable::OutStream(t, _) => t.clone(),
            Variable::Null(t) => t.clone(),
        }
    }

    pub fn to_string(&self) -> Result<String, ScriptError> {
        Ok(match self.clone() {
            Variable::Bool(_, Some(v)) => if v { "true" } else { "false" }.to_string(),
            Variable::String(_, Some(v)) => v,
            Variable::Integer(_, Some(v)) => v.to_string(),
            Variable::Float(_, Some(v)) => v.to_string(),
            Variable::Char(_, Some(v)) => {
                String::from_utf8(vec![v]).or(Err(ScriptError::StringUTF8Error))?
            }
            Variable::List(VarType::Char, Some(v)) => {
                let mut bytes = Vec::new();
                for ele in v {
                    bytes.push(ele.as_char()?);
                }
                String::from_utf8(bytes).or(Err(ScriptError::StringUTF8Error))?
            }
            Variable::List(_, Some(v)) => {
                let mut text = String::from("[");
                for i in 0..v.len() {
                    let item = &v[i];
                    text.push_str(&item.to_string()?);
                    if i != v.len() - 1 {
                        text.push_str(", ");
                    }
                }
                text.push(']');
                text
            }
            Variable::Map(_, Some(v)) => {
                let mut text = String::from("{");
                let mut i = 0;
                for (key, value) in &v {
                    text.push_str(&key.to_string()?);
                    text.push_str(": ");
                    text.push_str(&value.to_string()?);
                    if i != v.len() - 1 {
                        text.push_str(", ");
                    }
                    i += 1;
                }
                text.push('}');
                text
            }
            Variable::Optional(_, Some(v)) => match v {
                Some(v) => format!("({})", v.to_string()?),
                None => String::from("none"),
            },
            Variable::InStream(_, Some(_)) => String::from("IN_STREAM"),
            Variable::OutStream(_, Some(_)) => String::from("OUT_STREAM"),
            Variable::Null(_) => String::from("null"),
            _ => return Err(ScriptError::VarNotInitedError),
        })
    }

    pub fn is_null(&self) -> bool {
        if let Variable::Null(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_initialized(&self) -> bool {
        match self {
            Variable::Bool(_, b) => b.is_some(),
            Variable::String(_, b) => b.is_some(),
            Variable::Integer(_, b) => b.is_some(),
            Variable::Float(_, b) => b.is_some(),
            Variable::Char(_, b) => b.is_some(),
            Variable::List(_, b) => b.is_some(),
            Variable::Map(_, b) => b.is_some(),
            Variable::Optional(_, b) => b.is_some(),
            Variable::InStream(_, b) => b.is_some(),
            Variable::OutStream(_, b) => b.is_some(),
            Variable::Null(_) => true,
        }
    }

    pub fn from_bool(value: Option<bool>) -> Variable {
        Variable::Bool(VarType::Bool, value)
    }

    pub fn from_str(value: Option<String>) -> Variable {
        Variable::String(VarType::String, value)
    }

    pub fn from_int(value: Option<isize>) -> Variable {
        Variable::Integer(VarType::Integer, value)
    }

    pub fn from_float(value: Option<f64>) -> Variable {
        Variable::Float(VarType::Float, value)
    }

    pub fn from_char(value: Option<u8>) -> Variable {
        Variable::Char(VarType::Char, value)
    }

    pub fn from_list(value: Option<Vec<Variable>>, value_type: VarType) -> Variable {
        Variable::List(VarType::List(Box::new(value_type)), value)
    }

    pub fn from_map(
        value: Option<HashMap<Variable, Variable>>,
        key_type: VarType,
        value_type: VarType,
    ) -> Variable {
        Variable::Map(
            VarType::Map(Box::new(key_type), Box::new(value_type)),
            value,
        )
    }

    pub fn from_optional(value: Option<Option<Variable>>, var_type: VarType) -> Variable {
        Variable::Optional(
            VarType::Optional(Box::new(var_type)),
            match value {
                Some(value) => match value {
                    Some(value) => Some(Some(Box::new(value))),
                    None => Some(None),
                },
                None => None,
            },
        )
    }

    pub fn from_null() -> Variable {
        Variable::Null(VarType::Null)
    }

    pub fn from_out_stream(value: Option<Arc<Mutex<dyn Write>>>) -> Variable {
        Variable::OutStream(VarType::OutStream, value)
    }

    pub fn from_in_stream(value: Option<Arc<Mutex<dyn Read>>>) -> Variable {
        Variable::InStream(VarType::InStream, value)
    }

    pub fn as_out_stream(&self) -> Result<Arc<Mutex<dyn Write>>, ScriptError> {
        if let Variable::OutStream(_, Some(b)) = self {
            Ok(b.clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_in_stream(&self) -> Result<Arc<Mutex<dyn Read>>, ScriptError> {
        if let Variable::InStream(_, Some(b)) = self {
            Ok(b.clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn get_option_type(&self) -> Result<VarType, ScriptError> {
        if let Variable::Optional(VarType::Optional(v), _) = self {
            Ok(v.as_ref().clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_option(&self) -> Result<Option<Box<Variable>>, ScriptError> {
        if let Variable::Optional(_, Some(b)) = self {
            Ok(b.clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn get_map_types(&self) -> Result<(VarType, VarType), ScriptError> {
        if let Variable::Map(VarType::Map(k, v), _) = self {
            Ok((k.as_ref().clone(), v.as_ref().clone()))
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_map(&self) -> Result<HashMap<Variable, Variable>, ScriptError> {
        if let Variable::Map(_, Some(b)) = self {
            Ok(b.clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn get_list_type(&self) -> Result<VarType, ScriptError> {
        if let Variable::List(VarType::List(v), _) = self {
            Ok(v.as_ref().clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_list(&self) -> Result<Vec<Variable>, ScriptError> {
        if let Variable::List(_, Some(b)) = self {
            Ok(b.clone())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_char(&self) -> Result<u8, ScriptError> {
        if let Variable::Char(_, Some(b)) = self {
            Ok(*b)
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_float(&self) -> Result<f64, ScriptError> {
        if let Variable::Float(_, Some(b)) = self {
            Ok(*b)
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_int(&self) -> Result<isize, ScriptError> {
        if let Variable::Integer(_, Some(b)) = self {
            Ok(*b)
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_str(&self) -> Result<String, ScriptError> {
        if let Variable::String(_, Some(b)) = self {
            Ok(b.to_string())
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn as_bool(&self) -> Result<bool, ScriptError> {
        if let Variable::Bool(_, Some(b)) = self {
            Ok(*b)
        } else {
            Err(ScriptError::TypeMismatchError)
        }
    }

    pub fn not_inited_var(var_type: VarType) -> Result<Variable, ScriptError> {
        match var_type {
            VarType::Bool => Ok(Variable::Bool(VarType::Bool, None)),
            VarType::String => Ok(Variable::String(VarType::String, None)),
            VarType::Integer => Ok(Variable::Integer(VarType::Integer, None)),
            VarType::Float => Ok(Variable::Float(VarType::Float, None)),
            VarType::Char => Ok(Variable::Char(VarType::Char, None)),
            VarType::Optional(optional_type) => {
                Ok(Variable::Optional(VarType::Optional(optional_type), None))
            }
            VarType::List(value_type) => Ok(Variable::List(VarType::List(value_type), None)),
            VarType::Map(key_type, value_type) => {
                Ok(Variable::Map(VarType::Map(key_type, value_type), None))
            }
            VarType::InStream => Ok(Variable::InStream(VarType::InStream, None)),
            VarType::OutStream => Ok(Variable::OutStream(VarType::OutStream, None)),
            VarType::Null => Ok(Variable::Null(VarType::Null)),
        }
    }

    pub fn empty_var(var_type: VarType) -> Result<Variable, ScriptError> {
        match var_type {
            VarType::Bool => Ok(Variable::Bool(VarType::Bool, None)),
            VarType::String => Ok(Variable::String(VarType::String, None)),
            VarType::Integer => Ok(Variable::Integer(VarType::Integer, None)),
            VarType::Float => Ok(Variable::Float(VarType::Float, None)),
            VarType::Char => Ok(Variable::Char(VarType::Char, None)),
            VarType::Optional(optional_type) => Ok(Variable::Optional(
                VarType::Optional(optional_type),
                Some(None),
            )),
            VarType::List(value_type) => {
                Ok(Variable::List(VarType::List(value_type), Some(Vec::new())))
            }
            VarType::Map(key_type, value_type) => Ok(Variable::Map(
                VarType::Map(key_type, value_type),
                Some(HashMap::new()),
            )),
            VarType::InStream => Ok(Variable::InStream(VarType::InStream, None)),
            VarType::OutStream => Ok(Variable::OutStream(VarType::OutStream, None)),
            VarType::Null => Ok(Variable::Null(VarType::Null)),
        }
    }

    pub fn parse_var(var_type: VarType, text: String) -> Result<Variable, ScriptError> {
        match var_type {
            VarType::Bool => Ok(Variable::Bool(
                VarType::Bool,
                Some(match text.as_str() {
                    "true" => true,
                    "false" => false,
                    "1" => true,
                    "0" => false,
                    _ => {
                        return Err(ScriptError::ParseVarError);
                    }
                }),
            )),
            VarType::Null => Ok(Variable::Null(VarType::Null)),
            VarType::String => Ok(Variable::String(VarType::String, Some(text))),
            VarType::Integer => Ok(Variable::Integer(
                VarType::Integer,
                Some(match text.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(ScriptError::ParseVarError);
                    }
                }),
            )),
            VarType::Float => Ok(Variable::Float(
                VarType::Float,
                Some(match text.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(ScriptError::ParseVarError);
                    }
                }),
            )),
            VarType::Char => Ok(Variable::Char(
                VarType::Char,
                Some(match text.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return Err(ScriptError::ParseVarError);
                    }
                }),
            )),
            VarType::Optional(optional_type) => {
                if text.starts_with("[") && text.ends_with("]") {
                    let text = text[1..text.len() - 1].to_string();
                    Ok(Variable::Optional(
                        VarType::Optional(optional_type.clone()),
                        Some(Some(Box::new(Self::parse_var(
                            optional_type.clone().as_mut().clone(),
                            text,
                        )?))),
                    ))
                } else if text.as_str() == "none" {
                    Ok(Variable::Optional(
                        VarType::Optional(optional_type),
                        Some(None),
                    ))
                } else {
                    Err(ScriptError::ParseVarError)
                }
            }
            _ => Err(ScriptError::ParseVarError),
        }
    }
}

impl Eq for Variable {}
impl Hash for Variable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Variable::Bool(_, value) => {
                value.hash(state);
            }
            Variable::String(_, value) => {
                value.hash(state);
            }
            Variable::Integer(_, value) => {
                value.hash(state);
            }
            Variable::Float(_, value) => {
                hash(value, state);
            }
            Variable::Char(_, value) => {
                value.hash(state);
            }
            Variable::List(_, value) => {
                value.hash(state);
            }
            Variable::Map(_, value) => {
                hash(value, state);
            }
            Variable::Optional(_, value) => {
                value.hash(state);
            }
            Variable::InStream(_, value) => {
                hash(value, state);
            }
            Variable::OutStream(_, value) => {
                hash(value, state);
            }
            Variable::Null(t) => {
                hash(t, state);
            }
        }
    }
}
impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Variable::Bool(_, value) => match other {
                Variable::Bool(_, other_value) => value == other_value,
                _ => false,
            },
            Variable::Null(o) => match other {
                Variable::Null(t) => o == t,
                _ => false,
            },
            Variable::String(_, value) => match other {
                Variable::String(_, other_value) => value == other_value,
                _ => false,
            },
            Variable::Integer(_, value) => match other {
                Variable::Integer(_, other_value) => value == other_value,
                _ => false,
            },
            Variable::Float(_, value) => match other {
                Variable::Float(_, other_value) => value == other_value,
                _ => false,
            },
            Variable::Char(_, value) => match other {
                Variable::Char(_, other_value) => value == other_value,
                _ => false,
            },
            Variable::List(_, value) => match other {
                Variable::List(_, other_value) => value == other_value,
                _ => false,
            },
            Variable::Map(_, value) => match other {
                Variable::Map(_, other_value) => match value {
                    Some(value) => match other_value {
                        Some(other_value) => {
                            if other_value.len() == value.len() {
                                let mut ovi = other_value.iter();
                                let mut vi = value.iter();

                                loop {
                                    let Some((ok, ov)) = ovi.next() else {
                                        break;
                                    };
                                    let Some((k, v)) = vi.next() else {
                                        break;
                                    };
                                    if k != ok || v != ov {
                                        return false;
                                    }
                                }
                                true
                            } else {
                                false
                            }
                        }
                        None => false,
                    },
                    None => match other_value {
                        Some(_) => false,
                        None => true,
                    },
                },
                _ => false,
            },
            Variable::Optional(_, value) => match other {
                Variable::Optional(_, other_value) => other_value == value,
                _ => false,
            },
            Variable::InStream(_, value) => match other {
                Variable::InStream(_, other_value) => match value {
                    Some(value) => match other_value {
                        Some(other_value) => Arc::ptr_eq(value, other_value),
                        None => false,
                    },
                    None => match other_value {
                        Some(_) => false,
                        None => true,
                    },
                },
                _ => false,
            },
            Variable::OutStream(_, value) => match other {
                Variable::OutStream(_, other_value) => match value {
                    Some(value) => match other_value {
                        Some(other_value) => Arc::ptr_eq(value, other_value),
                        None => false,
                    },
                    None => match other_value {
                        Some(_) => false,
                        None => true,
                    },
                },
                _ => false,
            },
        }
    }
}
