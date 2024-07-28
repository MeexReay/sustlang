use super::super::command::Command;
use super::super::script::{Function, Script, ScriptError};
use super::super::var::{VarType, Variable};

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct RunningScript {
    main_function: Function,
    functions: Vec<Function>,
    variables: HashMap<String, Variable>,
}

unsafe impl Sync for RunningScript {}
unsafe impl Send for RunningScript {}

impl RunningScript {
    pub fn new(script: Script) -> RunningScript {
        RunningScript {
            functions: script.functions,
            variables: HashMap::new(),
            main_function: Function::new(
                "main".to_string(),
                VarType::Null,
                HashMap::new(),
                script.commands,
            ),
        }
    }

    pub fn set_standard_vars(
        &mut self,
        args: Vec<String>,
        cout: Box<dyn Write>,
        cin: Box<dyn Read>,
    ) -> Result<(), ScriptError> {
        self.set_var(
            String::from("args"),
            Variable::from_list(
                Some(
                    args.iter()
                        .map(|s| Variable::from_str(Some(s.to_string())))
                        .collect(),
                ),
                VarType::String,
            ),
            true,
            true,
            &mut HashMap::new(),
        )?;
        self.set_var(
            String::from("cout"),
            Variable::from_out_stream(Some(Arc::new(Mutex::new(cout)))),
            true,
            true,
            &mut HashMap::new(),
        )?;
        self.set_var(
            String::from("cin"),
            Variable::from_in_stream(Some(Arc::new(Mutex::new(cin)))),
            true,
            true,
            &mut HashMap::new(),
        )?;

        Ok(())
    }

    pub fn get_var(
        &mut self,
        name: String,
        locals: &mut HashMap<String, Variable>,
    ) -> Result<Variable, ScriptError> {
        let mut var: Option<Variable> = None;

        for part in name.split('.') {
            var = match &var {
                Some(v) => match v {
                    Variable::List(_, Some(list)) => {
                        let index: usize = part.parse().map_err(|_| ScriptError::ParseVarError)?;
                        Some(list.get(index).ok_or(ScriptError::UnknownVarError)?.clone())
                    }
                    Variable::Map(map_type, Some(map)) => {
                        let key_var = Variable::parse_var(map_type.clone(), part.to_string())?;
                        map.get(&key_var).cloned()
                    }
                    _ => return Err(ScriptError::TypeMismatchError),
                },
                None => locals
                    .get(part)
                    .or_else(|| self.variables.get(part))
                    .cloned(),
            };
        }

        var.ok_or(ScriptError::UnknownVarError)
    }

    pub fn drop_var(
        &mut self,
        name: String,
        locals: &mut HashMap<String, Variable>,
    ) -> Result<(), ScriptError> {
        let mut var: Option<&mut Variable> = None;
        let parts: Vec<&str> = name.split('.').collect();

        if parts.len() == 1 {
            if locals.remove(&name).is_some() || self.variables.remove(&name).is_some() {
                return Ok(());
            } else {
                return Err(ScriptError::UnknownVarError);
            }
        }

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                match &mut var {
                    Some(v) => match v {
                        Variable::List(_, list) => match list {
                            Some(list) => {
                                let index: usize =
                                    part.parse().map_err(|_| ScriptError::ParseVarError)?;
                                if index < list.len() {
                                    list.remove(index);
                                    return Ok(());
                                } else {
                                    return Err(ScriptError::UnknownVarError);
                                }
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        Variable::Map(map_type, map) => match map {
                            Some(map) => {
                                let key_var =
                                    Variable::parse_var(map_type.clone(), part.to_string())?;
                                if map.remove(&key_var).is_some() {
                                    return Ok(());
                                } else {
                                    return Err(ScriptError::UnknownVarError);
                                }
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        _ => return Err(ScriptError::TypeMismatchError),
                    },
                    None => return Err(ScriptError::UnknownVarError),
                }
            } else {
                var = match var {
                    Some(v) => match v {
                        Variable::List(_, list) => match list {
                            Some(list) => {
                                let index: usize =
                                    part.parse().map_err(|_| ScriptError::ParseVarError)?;
                                Some(list.get_mut(index).ok_or(ScriptError::UnknownVarError)?)
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        Variable::Map(map_type, map) => match map {
                            Some(map) => {
                                let key_var =
                                    Variable::parse_var(map_type.clone(), part.to_string())?;
                                map.get_mut(&key_var)
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        _ => return Err(ScriptError::TypeMismatchError),
                    },
                    None => locals
                        .get_mut(*part)
                        .or_else(|| self.variables.get_mut(*part)),
                };
            }
        }

        Err(ScriptError::UnknownVarError)
    }

    pub fn set_var(
        &mut self,
        name: String,
        value: Variable,
        global: bool,
        init: bool,
        locals: &mut HashMap<String, Variable>,
    ) -> Result<(), ScriptError> {
        let var_type = value.get_type();
        let mut var: Option<&mut Variable> = None;
        let parts: Vec<&str> = (&name).split('.').collect();

        let global = global
            || (self.variables.contains_key(parts[0]) && !locals.contains_key(parts[0]) && !init);

        if parts.len() == 1 {
            if global {
                self.variables.insert(name, value);
            } else {
                locals.insert(name.clone(), value.clone());
            }
            return Ok(());
        }

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                match &mut var {
                    Some(v) => match v {
                        Variable::List(_, list) => match list {
                            Some(list) => {
                                let index: usize =
                                    part.parse().map_err(|_| ScriptError::ParseVarError)?;
                                if index < list.len() {
                                    list[index] = value;
                                    return Ok(());
                                } else {
                                    return Err(ScriptError::UnknownVarError);
                                }
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        Variable::Map(map_type, map) => match map {
                            Some(map) => {
                                let key_var =
                                    Variable::parse_var(map_type.clone(), part.to_string())?;
                                map.insert(key_var, value);
                                return Ok(());
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        _ => return Err(ScriptError::TypeMismatchError),
                    },
                    None => return Err(ScriptError::UnknownVarError),
                }
            } else {
                var = match var {
                    Some(v) => match v {
                        Variable::List(_, list) => match list {
                            Some(list) => {
                                let index: usize =
                                    part.parse().map_err(|_| ScriptError::ParseVarError)?;
                                Some(list.get_mut(index).ok_or(ScriptError::UnknownVarError)?)
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        Variable::Map(map_type, map) => match map {
                            Some(map) => {
                                let key_var =
                                    Variable::parse_var(map_type.clone(), part.to_string())?;
                                map.get_mut(&key_var)
                            }
                            None => return Err(ScriptError::UnknownVarError),
                        },
                        _ => return Err(ScriptError::TypeMismatchError),
                    },
                    None => {
                        if global {
                            self.variables.get_mut(*part)
                        } else {
                            locals.get_mut(*part)
                        }
                    }
                }
            }
        }

        Err(ScriptError::UnknownVarError)
    }

    pub fn get_function(&self, name: String) -> Result<Function, ScriptError> {
        for func in &self.functions {
            if func.name == name {
                return Ok(func.clone());
            }
        }
        Err(ScriptError::FunctionUnknownError)
    }

    pub fn run(self) -> Result<(), (ScriptError, Command)> {
        let main_function = self.main_function.clone();

        main_function.execute(
            Arc::new(Mutex::new(self)),
            "null".to_string(),
            Vec::new(),
            true,
        )
    }
}
