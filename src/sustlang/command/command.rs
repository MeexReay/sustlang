use bytebuffer::ByteBuffer;
use rand::Rng;

use crate::{variable, FileOutStream, Pohuy};

use super::super::command::CommandType;
use super::super::script::{RunningScript, ScriptError};
use super::super::var::{VarType, Variable};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

#[derive(PartialEq, Clone, Debug)]
pub struct Command {
    pub command_type: CommandType,
    pub args: Vec<String>,
    pub line: usize,
}

impl Command {
    pub fn new(command_type: CommandType, line: usize, args: Vec<String>) -> Command {
        Command {
            command_type,
            args,
            line,
        }
    }

    pub fn execute(
        &self,
        script: Arc<Mutex<RunningScript>>,
        global: bool,
        locals: &mut HashMap<String, Variable>,
        temp_vars: &mut Vec<String>,
    ) -> Result<(), (ScriptError, Command)> {
        match self.command_type {
            CommandType::InitVar => {
                let type_var = self.args[0].clone();
                let type_var = VarType::from_name(&type_var).map_err(|f| (f, self.clone()))?;
                let name_var = self.args[1].clone();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        name_var,
                        Variable::empty_var(type_var).map_err(|f| (f, self.clone()))?,
                        global,
                        true,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::SetVar => {
                let name_var = self.args[0].clone();
                let value_var = self.args[1..].join(" ");

                let type_var = script
                    .lock()
                    .unwrap()
                    .get_var(name_var.clone(), &mut locals.clone())
                    .map_err(|f| (f, self.clone()))?
                    .get_type();
                let var =
                    Variable::parse_var(type_var, value_var).map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(name_var, var, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::TempVar => {
                let type_var = self.args[0].clone();
                let name_var = self.args[1].clone();
                let value_var = self.args[2..].join(" ");

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        name_var.clone(),
                        Variable::parse_var(
                            VarType::from_name(&type_var).map_err(|f| (f, self.clone()))?,
                            value_var,
                        )
                        .map_err(|f| (f, self.clone()))?,
                        global,
                        true,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;

                temp_vars.push(name_var);
            }
            CommandType::MoveVar => {
                let source_var = self.args[0].clone();
                let target_var = self.args[1].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(target_var, var, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
                script
                    .lock()
                    .unwrap()
                    .drop_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::CopyVar => {
                let source_var = self.args[0].clone();
                let target_var = self.args[1].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(target_var, var, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::DropVar => {
                let name_var = self.args[0].clone();

                script
                    .lock()
                    .unwrap()
                    .drop_var(name_var, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::HasVar => {
                let name_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let result = script.lock().unwrap().get_var(name_var, locals).is_ok();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::AddStr => {
                let var_name = self.args[0].clone();
                let other_var = self.args[1].clone();

                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;
                let other_var: String = if let Variable::List(VarType::Char, Some(list)) = other_var
                {
                    let mut bytes = Vec::new();
                    for ele in list {
                        bytes.push(ele.as_char().map_err(|f| (f, self.clone()))?);
                    }
                    String::from_utf8(bytes)
                        .or(Err(ScriptError::StringUTF8Error))
                        .map_err(|f| (f, self.clone()))?
                } else if let Variable::String(_, Some(string)) = other_var {
                    string
                } else if let Variable::Char(_, Some(value)) = other_var {
                    String::from_utf8(vec![value])
                        .or(Err(ScriptError::StringUTF8Error))
                        .map_err(|f| (f, self.clone()))?
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var_name.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_str()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        var_name.clone(),
                        Variable::from_str(Some(var.clone() + &other_var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Write => {
                let name_var = self.args[0].clone();
                let stream_var = self.args[1].clone();

                let text = script
                    .lock()
                    .unwrap()
                    .get_var(name_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;
                let text: Vec<u8> = if let Variable::List(VarType::Char, Some(list)) = text {
                    let mut bytes = Vec::new();
                    for ele in list {
                        bytes.push(ele.as_char().map_err(|f| (f, self.clone()))?);
                    }
                    bytes
                } else if let Variable::String(_, Some(string)) = text {
                    string.as_bytes().to_vec()
                } else if let Variable::Char(_, Some(value)) = text {
                    vec![value]
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                let stream = script
                    .lock()
                    .unwrap()
                    .get_var(stream_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_out_stream()
                    .map_err(|f| (f, self.clone()))?;
                stream.lock().unwrap().write_all(&text).unwrap();
            }
            CommandType::UseFunc => {
                let func_name = self.args[0].clone();
                let result_name = self.args[1].clone();
                let args_names = self.args[2..].to_vec();

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                let mut args = Vec::new();
                for name in args_names {
                    args.push(
                        script
                            .lock()
                            .unwrap()
                            .get_var(name, locals)
                            .map_err(|f| (f, self.clone()))?,
                    );
                }

                func.execute(script.clone(), result_name, args, false)?;
            }
            CommandType::Return => {
                return Ok(());
            }
            CommandType::For => {
                let func_name = self.args[0].clone();
                let start_index = script
                    .lock()
                    .unwrap()
                    .get_var(self.args[1].clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;
                let end_index = script
                    .lock()
                    .unwrap()
                    .get_var(self.args[2].clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                for index in start_index..=end_index {
                    func.execute(
                        script.clone(),
                        "null".to_string(),
                        vec![Variable::from_int(Some(index))],
                        false,
                    )?;
                }
            }
            CommandType::ToString => {
                let source_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let source_var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = source_var.to_string().map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_str(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ToChars => {
                let source_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let source_var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = source_var
                    .as_str()
                    .map_err(|f| (f, self.clone()))?
                    .as_bytes()
                    .iter()
                    .map(|f| Variable::from_char(Some(*f)))
                    .collect();
                let result =
                    Variable::from_list(Some(result), VarType::List(Box::new(VarType::Char)));

                script
                    .lock()
                    .unwrap()
                    .set_var(result_var, result, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ToInteger => {
                let source_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let source_var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = source_var
                    .as_str()
                    .map_err(|f| (f, self.clone()))?
                    .parse::<isize>()
                    .or(Err(ScriptError::ParseVarError))
                    .map_err(|f| (f, self.clone()))?;
                let result = Variable::from_int(Some(result));

                script
                    .lock()
                    .unwrap()
                    .set_var(result_var, result, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ToFloat => {
                let source_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let source_var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = source_var
                    .as_str()
                    .map_err(|f| (f, self.clone()))?
                    .parse::<f64>()
                    .or(Err(ScriptError::ParseVarError))
                    .map_err(|f| (f, self.clone()))?;
                let result = Variable::from_float(Some(result));

                script
                    .lock()
                    .unwrap()
                    .set_var(result_var, result, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ToBool => {
                let source_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let source_var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = if let Variable::List(_, Some(value)) = source_var {
                    !value.is_empty()
                } else if let Variable::String(_, Some(value)) = source_var {
                    value == "true" || value == "1"
                } else if let Variable::Char(_, Some(value)) = source_var {
                    value != 0
                } else if let Variable::Integer(_, Some(value)) = source_var {
                    value != 0
                } else if let Variable::Float(_, Some(value)) = source_var {
                    value != 0.0
                } else if let Variable::Bool(_, Some(value)) = source_var {
                    value
                } else if let Variable::Map(_, Some(value)) = source_var {
                    !value.is_empty()
                } else if let Variable::Optional(_, Some(value)) = source_var {
                    value.is_some()
                } else if let Variable::Null(_) = source_var {
                    false
                } else if let Variable::OutStream(_, Some(_)) = source_var {
                    true
                } else if let Variable::InStream(_, Some(_)) = source_var {
                    true
                } else {
                    false
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ToChar => {
                let source_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let source_var = script
                    .lock()
                    .unwrap()
                    .get_var(source_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = if let Variable::String(_, Some(value)) = source_var {
                    value.as_bytes()[0]
                } else if let Variable::Char(_, Some(value)) = source_var {
                    value
                } else if let Variable::Integer(_, Some(value)) = source_var {
                    value as u8
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_char(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::GetSymbol => {
                let str_var = self.args[0].clone();
                let index_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let str_var = script
                    .lock()
                    .unwrap()
                    .get_var(str_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let index_var = script
                    .lock()
                    .unwrap()
                    .get_var(index_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let index = index_var.as_int().map_err(|f| (f, self.clone()))?;

                let result = if let Variable::String(_, Some(value)) = str_var {
                    value.as_bytes()[index as usize]
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_char(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::GetItem => {
                let list_var = self.args[0].clone();
                let index_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let list_var = script
                    .lock()
                    .unwrap()
                    .get_var(list_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let index_var = script
                    .lock()
                    .unwrap()
                    .get_var(index_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let index = index_var.as_int().map_err(|f| (f, self.clone()))?;

                let result = if let Variable::List(_, Some(value)) = list_var {
                    value[index as usize].clone()
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(result_var, result, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::GetValue => {
                let map_var = self.args[0].clone();
                let key_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let map_var = script
                    .lock()
                    .unwrap()
                    .get_var(map_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let key_var = script
                    .lock()
                    .unwrap()
                    .get_var(key_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = if let Variable::Map(_, Some(value)) = map_var {
                    value[&key_var].clone()
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(result_var, result, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ListSize => {
                let list_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let list_var = script
                    .lock()
                    .unwrap()
                    .get_var(list_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let list_size = list_var.as_list().map_err(|f| (f, self.clone()))?.len();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_int(Some(list_size as isize)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::MapSize => {
                let map_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let map_var = script
                    .lock()
                    .unwrap()
                    .get_var(map_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let map_size = map_var.as_list().map_err(|f| (f, self.clone()))?.len();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_int(Some(map_size as isize)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::StringSize => {
                let string_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let string_var = script
                    .lock()
                    .unwrap()
                    .get_var(string_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let string_size = string_var.as_list().map_err(|f| (f, self.clone()))?.len();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_int(Some(string_size as isize)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ForMap => {
                let func_name = self.args[0].clone();
                let map_var = self.args[1].clone();

                let map_var = script
                    .lock()
                    .unwrap()
                    .get_var(map_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let map_var = map_var.as_map().map_err(|f| (f, self.clone()))?;

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                for (k, v) in map_var {
                    func.execute(script.clone(), "null".to_string(), vec![k, v], false)?;
                }
            }
            CommandType::ForList => {
                let func_name = self.args[0].clone();
                let list_var = self.args[1].clone();

                let list_var = script
                    .lock()
                    .unwrap()
                    .get_var(list_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let list_var = list_var.as_list().map_err(|f| (f, self.clone()))?;

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                for i in list_var {
                    func.execute(script.clone(), "null".to_string(), vec![i], false)?;
                }
            }
            CommandType::ForString => {
                let func_name = self.args[0].clone();
                let string_var = self.args[1].clone();

                let string_var = script
                    .lock()
                    .unwrap()
                    .get_var(string_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let string_var = string_var.as_str().map_err(|f| (f, self.clone()))?;

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                for c in string_var.as_bytes() {
                    func.execute(
                        script.clone(),
                        "null".to_string(),
                        vec![Variable::from_char(Some(*c))],
                        false,
                    )?;
                }
            }
            CommandType::While => {
                let func_name = self.args[0].clone();

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?
                    .clone();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        "while".to_string(),
                        Variable::from_bool(Some(true)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;

                loop {
                    func.execute(script.clone(), "while".to_string(), vec![], false)?;

                    let condition = script
                        .lock()
                        .unwrap()
                        .get_var("while".to_string(), locals)
                        .map_err(|f| (f, self.clone()))?
                        .as_bool()
                        .map_err(|f| (f, self.clone()))?;

                    if !condition {
                        break;
                    }
                }
            }
            CommandType::Equals => {
                let var = self.args[0].clone();
                let other_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(var == other_var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::More => {
                let var = self.args[0].clone();
                let other_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = if let Variable::Float(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 > v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 > v2 as f64
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 > v2 as f64
                    } else {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                } else if let Variable::Integer(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 as f64 > v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 > v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 > v2 as isize
                    } else {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                } else if let Variable::Char(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 as f64 > v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 as isize > v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 > v2
                    } else {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Less => {
                let var = self.args[0].clone();
                let other_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = if let Variable::Float(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 < v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 < v2 as f64
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 < v2 as f64
                    } else {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                } else if let Variable::Integer(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        (v1 as f64) < v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 < v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 < v2 as isize
                    } else {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                } else if let Variable::Char(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        (v1 as f64) < v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        (v1 as isize) < v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 < v2
                    } else {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                } else {
                    return Err((ScriptError::TypeMismatchError, self.clone()));
                };

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::And => {
                let var = self.args[0].clone();
                let other_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_bool()
                    .map_err(|f| (f, self.clone()))?;
                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_bool()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(var && other_var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Or => {
                let var = self.args[0].clone();
                let other_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_bool()
                    .map_err(|f| (f, self.clone()))?;
                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_bool()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(var || other_var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Not => {
                let var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_bool()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(!var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::If => {
                let bool_var = self.args[0].clone();
                let func_name = self.args[1].clone();

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                let bool_var = script
                    .lock()
                    .unwrap()
                    .get_var(bool_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_bool()
                    .map_err(|f| (f, self.clone()))?;

                if bool_var {
                    func.execute(script.clone(), "null".to_string(), vec![], false)?;
                }
            }
            CommandType::HasStr => {
                let string_var = self.args[0].clone();
                let substring = self.args[1].clone();
                let result_var = self.args[2].clone();

                let string_var = script
                    .lock()
                    .unwrap()
                    .get_var(string_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_str()
                    .map_err(|f| (f, self.clone()))?;
                let substring = script
                    .lock()
                    .unwrap()
                    .get_var(substring, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_str()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(string_var.contains(&substring))),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::HasItem => {
                let list_var = self.args[0].clone();
                let item_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let list_var = script
                    .lock()
                    .unwrap()
                    .get_var(list_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_list()
                    .map_err(|f| (f, self.clone()))?;
                let item_var = script
                    .lock()
                    .unwrap()
                    .get_var(item_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(list_var.contains(&item_var))),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::HasEntry => {
                let map_var = self.args[0].clone();
                let key_var = self.args[1].clone();
                let value_var = self.args[2].clone();
                let result_var = self.args[3].clone();

                let map_var = script
                    .lock()
                    .unwrap()
                    .get_var(map_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_map()
                    .map_err(|f| (f, self.clone()))?;
                let key_var = script
                    .lock()
                    .unwrap()
                    .get_var(key_var, locals)
                    .map_err(|f| (f, self.clone()))?;
                let value_var = script
                    .lock()
                    .unwrap()
                    .get_var(value_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let mut has = false;

                for (k, v) in map_var {
                    if k == key_var && v == value_var {
                        has = true;
                        break;
                    }
                }

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(has)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::HasKey => {
                let map_var = self.args[0].clone();
                let key_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let map_var = script
                    .lock()
                    .unwrap()
                    .get_var(map_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_map()
                    .map_err(|f| (f, self.clone()))?;
                let key_var = script
                    .lock()
                    .unwrap()
                    .get_var(key_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let mut has = false;

                for (k, _) in map_var {
                    if k == key_var {
                        has = true;
                        break;
                    }
                }

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(has)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::HasValue => {
                let map_var = self.args[0].clone();
                let value_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let map_var = script
                    .lock()
                    .unwrap()
                    .get_var(map_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_map()
                    .map_err(|f| (f, self.clone()))?;
                let value_var = script
                    .lock()
                    .unwrap()
                    .get_var(value_var, locals)
                    .map_err(|f| (f, self.clone()))?;

                let mut has = false;

                for (_, v) in map_var {
                    if v == value_var {
                        has = true;
                        break;
                    }
                }

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(has)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::HasOptional => {
                let optional_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let optional_var = script
                    .lock()
                    .unwrap()
                    .get_var(optional_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_option()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_bool(Some(optional_var.is_some())),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::UnpackOptional => {
                let optional_var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let optional_var = script
                    .lock()
                    .unwrap()
                    .get_var(optional_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_option()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        optional_var
                            .ok_or(ScriptError::ParseVarError)
                            .map_err(|f| (f, self.clone()))?
                            .as_mut()
                            .clone(),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Sleep => {
                let time_var = self.args[0].clone();

                let time_var = match script
                    .lock()
                    .unwrap()
                    .get_var(time_var, locals)
                    .map_err(|f| (f, self.clone()))?
                {
                    Variable::Integer(_, Some(v)) => Duration::from_millis(v as u64),
                    Variable::Float(_, Some(v)) => Duration::from_millis(v as u64),
                    _ => {
                        return Err((ScriptError::TypeMismatchError, self.clone()));
                    }
                };

                thread::sleep(time_var);
            }
            CommandType::AddInt => {
                let var_name = self.args[0].clone();
                let other_var = self.args[1].clone();

                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;
                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var_name.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        var_name,
                        Variable::from_int(Some(var + other_var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::AddFloat => {
                let var_name = self.args[0].clone();
                let other_var = self.args[1].clone();

                let other_var = script
                    .lock()
                    .unwrap()
                    .get_var(other_var, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_float()
                    .map_err(|f| (f, self.clone()))?;
                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var_name.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_float()
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        var_name,
                        Variable::from_float(Some(var + other_var)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::SubStr => {
                let str_var_name = self.args[0].clone();
                let start_index = self.args[1].clone();
                let end_index = self.args[1].clone();

                let str_var = script
                    .lock()
                    .unwrap()
                    .get_var(str_var_name.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_str()
                    .map_err(|f| (f, self.clone()))?;
                let start_index = script
                    .lock()
                    .unwrap()
                    .get_var(start_index, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))? as usize;
                let end_index = script
                    .lock()
                    .unwrap()
                    .get_var(end_index, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))? as usize;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        str_var_name,
                        Variable::from_str(Some(str_var[start_index..end_index].to_string())),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::SubList => {
                let list_var_name = self.args[0].clone();
                let start_index = self.args[1].clone();
                let end_index = self.args[1].clone();

                let list_var = script
                    .lock()
                    .unwrap()
                    .get_var(list_var_name.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;
                let start_index = script
                    .lock()
                    .unwrap()
                    .get_var(start_index, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))? as usize;
                let end_index = script
                    .lock()
                    .unwrap()
                    .get_var(end_index, locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))? as usize;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        list_var_name,
                        Variable::from_list(
                            Some(
                                list_var.as_list().map_err(|f| (f, self.clone()))?
                                    [start_index..end_index]
                                    .to_vec(),
                            ),
                            list_var.get_type(),
                        ),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Read => {
                let name_var = self.args[0].clone();
                let size_var = self.args[1].clone();
                let stream_var = self.args[2].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(name_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;
                let size_var = script
                    .lock()
                    .unwrap()
                    .get_var(size_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;
                let stream = script
                    .lock()
                    .unwrap()
                    .get_var(stream_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_in_stream()
                    .map_err(|f| (f, self.clone()))?;

                let mut buffer: Vec<u8> = Vec::with_capacity(size_var as usize);
                stream.lock().unwrap().read_exact(&mut buffer).unwrap();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        name_var,
                        match var {
                            Variable::List(VarType::Char, _) => Variable::from_list(
                                Some(
                                    buffer
                                        .iter()
                                        .map(|f| Variable::from_char(Some(*f)))
                                        .collect(),
                                ),
                                VarType::List(Box::new(VarType::Char)),
                            ),
                            Variable::String(_, _) => Variable::from_str(Some(
                                String::from_utf8(buffer)
                                    .or(Err(ScriptError::StringUTF8Error))
                                    .map_err(|f| (f, self.clone()))?,
                            )),
                            _ => {
                                return Err((ScriptError::TypeMismatchError, self.clone()));
                            }
                        },
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::ReadAll => {
                let name_var = self.args[0].clone();
                let stream_var = self.args[1].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(name_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;
                let stream = script
                    .lock()
                    .unwrap()
                    .get_var(stream_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_in_stream()
                    .map_err(|f| (f, self.clone()))?;

                let mut buffer: Vec<u8> = Vec::new();
                stream.lock().unwrap().read_to_end(&mut buffer).unwrap();

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        name_var,
                        match var {
                            Variable::List(VarType::Char, _) => Variable::from_list(
                                Some(
                                    buffer
                                        .iter()
                                        .map(|f| Variable::from_char(Some(*f)))
                                        .collect(),
                                ),
                                VarType::List(Box::new(VarType::Char)),
                            ),
                            Variable::String(_, _) => Variable::from_str(Some(
                                String::from_utf8(buffer)
                                    .or(Err(ScriptError::StringUTF8Error))
                                    .map_err(|f| (f, self.clone()))?,
                            )),
                            _ => {
                                return Err((ScriptError::TypeMismatchError, self.clone()));
                            }
                        },
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::PackOptional => {
                let var = self.args[0].clone();
                let result_var = self.args[1].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;

                let result = Variable::from_optional(Some(Some(var.clone())), var.get_type());

                script
                    .lock()
                    .unwrap()
                    .set_var(result_var, result, global, false, locals)
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::NoneOptional => {
                let var_name = self.args[0].clone();

                let var = script
                    .lock()
                    .unwrap()
                    .get_var(var_name.clone(), locals)
                    .map_err(|f| (f, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        var_name,
                        Variable::from_optional(
                            Some(None),
                            var.get_option_type().map_err(|f| (f, self.clone()))?,
                        ),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::NewThread => {
                let func_name = self.args[0].clone();

                let func = script
                    .lock()
                    .unwrap()
                    .get_function(func_name)
                    .map_err(|f| (f, self.clone()))?;

                let local_script = script.clone();
                thread::spawn(move || {
                    match func.execute(local_script, "null".to_string(), vec![], false) {
                        Ok(_) => {}
                        Err((e, c)) => {
                            println!("error ({:?}) command: {:?}", e, c);
                        }
                    };
                });
            }
            CommandType::Random => {
                let min_var = self.args[0].clone();
                let max_var = self.args[1].clone();
                let result_var = self.args[2].clone();

                let min_var = script
                    .lock()
                    .unwrap()
                    .get_var(min_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;

                let max_var = script
                    .lock()
                    .unwrap()
                    .get_var(max_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_int()
                    .map_err(|f| (f, self.clone()))?;

                let result = rand::thread_rng().gen_range(min_var..=max_var);

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        result_var,
                        Variable::from_int(Some(result)),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::Import => {
                let script_path_var = self.args[0].clone();

                // TODO: write logic
            }
            CommandType::ImportText => {
                let script_text_var = self.args[0].clone();

                // TODO: write logic
            }
            CommandType::OpenFileIn => {
                let path_var = self.args[0].clone();
                let stream_var = self.args[1].clone();

                let path_var = script
                    .lock()
                    .unwrap()
                    .get_var(path_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_str()
                    .map_err(|f| (f, self.clone()))?;

                let result =
                    fs::read(path_var).map_err(|_| (ScriptError::FileReadError, self.clone()))?;

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        stream_var,
                        Variable::from_in_stream(Some(Arc::new(Mutex::new(
                            ByteBuffer::from_bytes(&result),
                        )))),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::OpenFileOut => {
                let path_var = self.args[0].clone();
                let stream_var = self.args[1].clone();

                let path_var = script
                    .lock()
                    .unwrap()
                    .get_var(path_var.clone(), locals)
                    .map_err(|f| (f, self.clone()))?
                    .as_str()
                    .map_err(|f| (f, self.clone()))?;

                let bytes = fs::read(path_var.clone())
                    .map_err(|_| (ScriptError::FileWriteError, self.clone()))?;
                let result = FileOutStream::new(path_var, bytes);

                script
                    .lock()
                    .unwrap()
                    .set_var(
                        stream_var,
                        Variable::from_out_stream(Some(Arc::new(Mutex::new(result)))),
                        global,
                        false,
                        locals,
                    )
                    .map_err(|f| (f, self.clone()))?;
            }
            CommandType::OpenTcpConnection => {
                let addr_var = self.args[0].clone();
                let port_var = self.args[1].clone();
                let in_stream = self.args[2].clone();
                let out_stream = self.args[3].clone();

                // TODO: write logic
            }
            CommandType::OpenTcpListener => {
                let addr_var = self.args[0].clone();
                let port_var = self.args[1].clone();
                let accept_func = self.args[2].clone();

                // TODO: write logic
            }
            _ => {}
        }

        Ok(())
    }
}
