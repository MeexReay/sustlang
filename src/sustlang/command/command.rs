#[derive(PartialEq, Clone, Debug)]
pub struct Command {
    command_type: CommandType,
    args: Vec<String>,
    line: usize,
}

impl Command {
    pub fn new(command_type: CommandType, line: usize, args: Vec<String>) -> Command {
        Command {
            command_type,
            args,
            line,
        }
    }

    fn execute(
        &self,
        global: bool,
        locals: &mut HashMap<String, Variable>,
        globals: &mut HashMap<String, Variable>,
        temp_vars: &mut Vec<String>,
    ) -> Result<(), ScriptError> {
        match command.command_type {
            CommandType::InitVar => {
                let type_var = command.args[0].clone();
                let type_var = VarType::from_name(&type_var)?;
                let name_var = command.args[1].clone();

                self.set_var(
                    name_var,
                    Variable::empty_var(type_var)?,
                    global,
                    true,
                    locals,
                )?;
            }
            CommandType::SetVar => {
                let name_var = command.args[0].clone();
                let value_var = command.args[1..].join(" ");

                let type_var = self
                    .get_var(name_var.clone(), &mut locals.clone())?
                    .get_type();
                let var = Variable::parse_var(type_var, value_var)?;

                self.set_var(name_var, var, global, false, locals)?;
            }
            CommandType::TempVar => {
                let type_var = command.args[0].clone();
                let name_var = command.args[1].clone();
                let value_var = command.args[2..].join(" ");

                self.set_var(
                    name_var.clone(),
                    Variable::parse_var(VarType::from_name(&type_var)?, value_var)?,
                    global,
                    true,
                    locals,
                )?;

                temp_vars.push(name_var);
            }
            CommandType::MoveVar => {
                let source_var = command.args[0].clone();
                let target_var = command.args[1].clone();

                let var = self.get_var(source_var.clone(), locals)?;

                self.set_var(target_var, var, global, false, locals)?;
                self.drop_var(source_var, locals)?;
            }
            CommandType::CopyVar => {
                let source_var = command.args[0].clone();
                let target_var = command.args[1].clone();

                let var = self.get_var(source_var.clone(), locals)?;

                self.set_var(target_var, var, global, false, locals)?;
            }
            CommandType::DropVar => {
                let name_var = command.args[0].clone();

                self.drop_var(name_var, locals)?;
            }
            CommandType::HasVar => {
                let name_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let result = self.get_var(name_var, locals).is_ok();

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::AddStr => {
                let var_name = command.args[0].clone();
                let other_var = command.args[1].clone();

                let other_var = self.get_var(other_var.clone(), locals)?;
                let other_var: String = if let Variable::List(VarType::Char, Some(list)) = other_var
                {
                    let mut bytes = Vec::new();
                    for ele in list {
                        bytes.push(ele.as_char()?);
                    }
                    String::from_utf8(bytes).or(Err(ScriptError::StringUTF8Error))?
                } else if let Variable::String(_, Some(string)) = other_var {
                    string
                } else if let Variable::Char(_, Some(value)) = other_var {
                    String::from_utf8(vec![value]).or(Err(ScriptError::StringUTF8Error))?
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                let var = self.get_var(var_name.clone(), locals)?.as_str()?;

                self.set_var(
                    var_name,
                    Variable::from_str(Some(var + &other_var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::Write => {
                let name_var = command.args[0].clone();
                let stream_var = command.args[1].clone();

                let text = self.get_var(name_var.clone(), locals)?;
                let text: Vec<u8> = if let Variable::List(VarType::Char, Some(list)) = text {
                    let mut bytes = Vec::new();
                    for ele in list {
                        bytes.push(ele.as_char()?);
                    }
                    bytes
                } else if let Variable::String(_, Some(string)) = text {
                    string.as_bytes().to_vec()
                } else if let Variable::Char(_, Some(value)) = text {
                    vec![value]
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                let stream = self.get_var(stream_var.clone(), locals)?.as_out_stream()?;
                stream.lock().unwrap().write_all(&text).unwrap();
            }
            CommandType::UseFunc => {
                let func_name = command.args[0].clone();
                let result_name = command.args[1].clone();
                let args_names = command.args[2..].to_vec();

                let func = self.get_function(func_name).unwrap();

                let mut args = Vec::new();
                for name in args_names {
                    args.push(self.get_var(name, locals)?);
                }

                self.exec_function(func, result_name, args)?;
            }
            CommandType::Return => {
                return Ok(());
            }
            CommandType::For => {
                let func_name = command.args[0].clone();
                let start_index = self.get_var(command.args[1].clone(), locals)?.as_int()?;
                let end_index = self.get_var(command.args[2].clone(), locals)?.as_int()?;

                let func = self.get_function(func_name).unwrap();

                for index in start_index..=end_index {
                    self.exec_function(
                        func.clone(),
                        "null".to_string(),
                        vec![Variable::from_int(Some(index))],
                    )?;
                }
            }
            CommandType::ToString => {
                let source_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let source_var = self.get_var(source_var, locals)?;

                let result = source_var.to_string()?;

                self.set_var(
                    result_var,
                    Variable::from_str(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::ToChars => {
                let source_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let source_var = self.get_var(source_var, locals)?;

                let result = source_var
                    .as_str()?
                    .as_bytes()
                    .iter()
                    .map(|f| Variable::from_char(Some(*f)))
                    .collect();
                let result =
                    Variable::from_list(Some(result), VarType::List(Box::new(VarType::Char)));

                self.set_var(result_var, result, global, false, locals)?;
            }
            CommandType::ToInteger => {
                let source_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let source_var = self.get_var(source_var, locals)?;

                let result = source_var
                    .as_str()?
                    .parse::<isize>()
                    .or(Err(ScriptError::ParseVarError))?;
                let result = Variable::from_int(Some(result));

                self.set_var(result_var, result, global, false, locals)?;
            }
            CommandType::ToFloat => {
                let source_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let source_var = self.get_var(source_var, locals)?;

                let result = source_var
                    .as_str()?
                    .parse::<f64>()
                    .or(Err(ScriptError::ParseVarError))?;
                let result = Variable::from_float(Some(result));

                self.set_var(result_var, result, global, false, locals)?;
            }
            CommandType::ToBool => {
                let source_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let source_var = self.get_var(source_var, locals)?;

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

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::ToChar => {
                let source_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let source_var = self.get_var(source_var, locals)?;

                let result = if let Variable::String(_, Some(value)) = source_var {
                    value.as_bytes()[0]
                } else if let Variable::Char(_, Some(value)) = source_var {
                    value
                } else if let Variable::Integer(_, Some(value)) = source_var {
                    value as u8
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                self.set_var(
                    result_var,
                    Variable::from_char(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::GetSymbol => {
                let str_var = command.args[0].clone();
                let index_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let str_var = self.get_var(str_var, locals)?;
                let index_var = self.get_var(index_var, locals)?;

                let index = index_var.as_int()?;

                let result = if let Variable::String(_, Some(value)) = str_var {
                    value.as_bytes()[index as usize]
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                self.set_var(
                    result_var,
                    Variable::from_char(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::GetItem => {
                let list_var = command.args[0].clone();
                let index_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let list_var = self.get_var(list_var, locals)?;
                let index_var = self.get_var(index_var, locals)?;

                let index = index_var.as_int()?;

                let result = if let Variable::List(_, Some(value)) = list_var {
                    value[index as usize].clone()
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                self.set_var(result_var, result, global, false, locals)?;
            }
            CommandType::GetValue => {
                let map_var = command.args[0].clone();
                let key_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let map_var = self.get_var(map_var, locals)?;
                let key_var = self.get_var(key_var, locals)?;

                let result = if let Variable::Map(_, Some(value)) = map_var {
                    value[&key_var].clone()
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                self.set_var(result_var, result, global, false, locals)?;
            }
            CommandType::ListSize => {
                let list_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let list_var = self.get_var(list_var, locals)?;
                let list_size = list_var.as_list()?.len();

                self.set_var(
                    result_var,
                    Variable::from_int(Some(list_size as isize)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::MapSize => {
                let map_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let map_var = self.get_var(map_var, locals)?;
                let map_size = map_var.as_list()?.len();

                self.set_var(
                    result_var,
                    Variable::from_int(Some(map_size as isize)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::StringSize => {
                let string_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let string_var = self.get_var(string_var, locals)?;
                let string_size = string_var.as_list()?.len();

                self.set_var(
                    result_var,
                    Variable::from_int(Some(string_size as isize)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::ForMap => {
                let func_name = command.args[0].clone();
                let map_var = command.args[1].clone();

                let map_var = self.get_var(map_var, locals)?;
                let map_var = map_var.as_map()?;

                let func = self.get_function(func_name).unwrap();

                for (k, v) in map_var {
                    self.exec_function(func.clone(), "null".to_string(), vec![k, v])?;
                }
            }
            CommandType::ForList => {
                let func_name = command.args[0].clone();
                let list_var = command.args[1].clone();

                let list_var = self.get_var(list_var, locals)?;
                let list_var = list_var.as_list()?;

                let func = self.get_function(func_name).unwrap();

                for i in list_var {
                    self.exec_function(func.clone(), "null".to_string(), vec![i])?;
                }
            }
            CommandType::ForString => {
                let func_name = command.args[0].clone();
                let string_var = command.args[1].clone();

                let string_var = self.get_var(string_var, locals)?;
                let string_var = string_var.as_str()?;

                let func = self.get_function(func_name).unwrap();

                for c in string_var.as_bytes() {
                    self.exec_function(
                        func.clone(),
                        "null".to_string(),
                        vec![Variable::from_char(Some(*c))],
                    )?;
                }
            }
            CommandType::While => {
                let func_name = command.args[0].clone();

                let func = self.get_function(func_name).unwrap();

                self.set_var(
                    "while".to_string(),
                    Variable::from_bool(Some(true)),
                    global,
                    false,
                    locals,
                )?;

                while self.get_var("while".to_string(), locals)?.as_bool()? {
                    self.exec_function(func.clone(), "while".to_string(), vec![])?;
                }
            }
            CommandType::Equals => {
                let var = command.args[0].clone();
                let other_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let var = self.get_var(var, locals)?;
                let other_var = self.get_var(other_var, locals)?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(var == other_var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::More => {
                let var = command.args[0].clone();
                let other_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let var = self.get_var(var, locals)?;
                let other_var = self.get_var(other_var, locals)?;

                let result = if let Variable::Float(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 > v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 > v2 as f64
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 > v2 as f64
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    }
                } else if let Variable::Integer(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 as f64 > v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 > v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 > v2 as isize
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    }
                } else if let Variable::Char(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 as f64 > v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 as isize > v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 > v2
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    }
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::Less => {
                let var = command.args[0].clone();
                let other_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let var = self.get_var(var, locals)?;
                let other_var = self.get_var(other_var, locals)?;

                let result = if let Variable::Float(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        v1 < v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 < v2 as f64
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 < v2 as f64
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    }
                } else if let Variable::Integer(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        (v1 as f64) < v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        v1 < v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 < v2 as isize
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    }
                } else if let Variable::Char(_, Some(v1)) = var {
                    if let Variable::Float(_, Some(v2)) = other_var {
                        (v1 as f64) < v2
                    } else if let Variable::Integer(_, Some(v2)) = other_var {
                        (v1 as isize) < v2
                    } else if let Variable::Char(_, Some(v2)) = other_var {
                        v1 < v2
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    }
                } else {
                    return Err(ScriptError::TypeMismatchError);
                };

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(result)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::And => {
                let var = command.args[0].clone();
                let other_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let var = self.get_var(var, locals)?.as_bool()?;
                let other_var = self.get_var(other_var, locals)?.as_bool()?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(var && other_var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::Or => {
                let var = command.args[0].clone();
                let other_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let var = self.get_var(var, locals)?.as_bool()?;
                let other_var = self.get_var(other_var, locals)?.as_bool()?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(var || other_var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::Not => {
                let var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let var = self.get_var(var, locals)?.as_bool()?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(!var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::If => {
                let bool_var = command.args[0].clone();
                let func_name = command.args[1].clone();

                let func = self.get_function(func_name).unwrap();

                let bool_var = self.get_var(bool_var, locals)?.as_bool()?;

                if bool_var {
                    self.exec_function(func, "null".to_string(), vec![])?;
                }
            }
            CommandType::HasStr => {
                let string_var = command.args[0].clone();
                let substring = command.args[1].clone();
                let result_var = command.args[2].clone();

                let string_var = self.get_var(string_var, locals)?.as_str()?;
                let substring = self.get_var(substring, locals)?.as_str()?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(string_var.contains(&substring))),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::HasItem => {
                let list_var = command.args[0].clone();
                let item_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let list_var = self.get_var(list_var, locals)?.as_list()?;
                let item_var = self.get_var(item_var, locals)?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(list_var.contains(&item_var))),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::HasEntry => {
                let map_var = command.args[0].clone();
                let key_var = command.args[1].clone();
                let value_var = command.args[2].clone();
                let result_var = command.args[3].clone();

                let map_var = self.get_var(map_var, locals)?.as_map()?;
                let key_var = self.get_var(key_var, locals)?;
                let value_var = self.get_var(value_var, locals)?;

                let mut has = false;

                for (k, v) in map_var {
                    if k == key_var && v == value_var {
                        has = true;
                        break;
                    }
                }

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(has)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::HasKey => {
                let map_var = command.args[0].clone();
                let key_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let map_var = self.get_var(map_var, locals)?.as_map()?;
                let key_var = self.get_var(key_var, locals)?;

                let mut has = false;

                for (k, _) in map_var {
                    if k == key_var {
                        has = true;
                        break;
                    }
                }

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(has)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::HasValue => {
                let map_var = command.args[0].clone();
                let value_var = command.args[1].clone();
                let result_var = command.args[2].clone();

                let map_var = self.get_var(map_var, locals)?.as_map()?;
                let value_var = self.get_var(value_var, locals)?;

                let mut has = false;

                for (_, v) in map_var {
                    if v == value_var {
                        has = true;
                        break;
                    }
                }

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(has)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::HasOptional => {
                let optional_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let optional_var = self.get_var(optional_var, locals)?.as_option()?;

                self.set_var(
                    result_var,
                    Variable::from_bool(Some(optional_var.is_some())),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::UnpackOptional => {
                let optional_var = command.args[0].clone();
                let result_var = command.args[1].clone();

                let optional_var = self.get_var(optional_var, locals)?.as_option()?;

                self.set_var(
                    result_var,
                    optional_var
                        .ok_or(ScriptError::ParseVarError)?
                        .as_mut()
                        .clone(),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::Sleep => {
                let time_var = command.args[0].clone();

                let time_var = match self.get_var(time_var, locals)? {
                    Variable::Integer(_, Some(v)) => Duration::from_millis(v as u64),
                    Variable::Float(_, Some(v)) => Duration::from_millis(v as u64),
                    _ => {
                        return Err(ScriptError::TypeMismatchError);
                    }
                };

                thread::sleep(time_var);
            }
            CommandType::AddInt => {
                let var_name = command.args[0].clone();
                let other_var = command.args[1].clone();

                let other_var = self.get_var(other_var, locals)?.as_int()?;
                let var = self.get_var(var_name.clone(), locals)?.as_int()?;

                self.set_var(
                    var_name,
                    Variable::from_int(Some(var + other_var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::AddFloat => {
                let var_name = command.args[0].clone();
                let other_var = command.args[1].clone();

                let other_var = self.get_var(other_var, locals)?.as_float()?;
                let var = self.get_var(var_name.clone(), locals)?.as_float()?;

                self.set_var(
                    var_name,
                    Variable::from_float(Some(var + other_var)),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::SubStr => {
                let str_var_name = command.args[0].clone();
                let start_index = command.args[1].clone();
                let end_index = command.args[1].clone();

                let str_var = self.get_var(str_var_name.clone(), locals)?.as_str()?;
                let start_index = self.get_var(start_index, locals)?.as_int()? as usize;
                let end_index = self.get_var(end_index, locals)?.as_int()? as usize;

                self.set_var(
                    str_var_name,
                    Variable::from_str(Some(str_var[start_index..end_index].to_string())),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::SubList => {
                let list_var_name = command.args[0].clone();
                let start_index = command.args[1].clone();
                let end_index = command.args[1].clone();

                let list_var = self.get_var(list_var_name.clone(), locals)?;
                let start_index = self.get_var(start_index, locals)?.as_int()? as usize;
                let end_index = self.get_var(end_index, locals)?.as_int()? as usize;

                self.set_var(
                    list_var_name,
                    Variable::from_list(
                        Some(list_var.as_list()?[start_index..end_index].to_vec()),
                        list_var.get_type(),
                    ),
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::Read => {
                let name_var = command.args[0].clone();
                let size_var = command.args[1].clone();
                let stream_var = command.args[2].clone();

                let var = self.get_var(name_var.clone(), locals)?;
                let size_var = self.get_var(size_var.clone(), locals)?.as_int()?;
                let stream = self.get_var(stream_var.clone(), locals)?.as_in_stream()?;

                let mut buffer: Vec<u8> = Vec::with_capacity(size_var as usize);
                stream.lock().unwrap().read_exact(&mut buffer).unwrap();

                self.set_var(
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
                            String::from_utf8(buffer).or(Err(ScriptError::StringUTF8Error))?,
                        )),
                        _ => {
                            return Err(ScriptError::TypeMismatchError);
                        }
                    },
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::ReadAll => {
                let name_var = command.args[0].clone();
                let stream_var = command.args[1].clone();

                let var = self.get_var(name_var.clone(), locals)?;
                let stream = self.get_var(stream_var.clone(), locals)?.as_in_stream()?;

                let mut buffer: Vec<u8> = Vec::new();
                stream.lock().unwrap().read_to_end(&mut buffer).unwrap();

                self.set_var(
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
                            String::from_utf8(buffer).or(Err(ScriptError::StringUTF8Error))?,
                        )),
                        _ => {
                            return Err(ScriptError::TypeMismatchError);
                        }
                    },
                    global,
                    false,
                    locals,
                )?;
            }
            CommandType::OpenFileIn => {
                let path_var = command.args[0].clone();
                let stream_var = command.args[1].clone();

                // TODO: write logic
            }
            CommandType::OpenFileOut => {
                let path_var = command.args[0].clone();
                let stream_var = command.args[1].clone();

                // TODO: write logic
            }
            CommandType::OpenTcpConnection => {
                let addr_var = command.args[0].clone();
                let port_var = command.args[1].clone();
                let in_stream = command.args[2].clone();
                let out_stream = command.args[3].clone();

                // TODO: write logic
            }
            CommandType::OpenTcpListener => {
                let addr_var = command.args[0].clone();
                let port_var = command.args[1].clone();
                let accept_func = command.args[2].clone();

                // TODO: write logic
            }
            CommandType::NewThread => {
                let func_name = command.args[0].clone();

                // TODO: write logic
            }
            _ => {}
        }

        Ok(())
    }
}
