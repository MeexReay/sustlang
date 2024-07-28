use super::super::command::{Command, CommandType};
use super::super::script::{Function, ScriptError};
use super::super::var::VarType;

use std::collections::HashMap;

fn prepare_script(text: String) -> Vec<String> {
    text.lines()
        .map(|s| match s.split_once("#") {
            Some(s) => s.0,
            None => s,
        })
        .map(|s| {
            s.trim_end_matches(" ")
                .trim_end_matches("\t")
                .trim_start_matches(" ")
                .trim_start_matches("\t")
                .to_string()
        })
        .collect()
}

fn parse_commands(lines: Vec<String>) -> Result<Vec<Command>, (ScriptError, usize)> {
    let mut commands = Vec::new();
    let mut line_num = 0;

    for line in lines {
        line_num += 1;

        if line.trim().is_empty() {
            continue;
        }

        let params: Vec<String> = line.split(" ").map(|v| v.to_string()).collect();

        let command_type = CommandType::from_name(&params[0]).map_err(|f| (f, line_num))?;

        let args = if params.is_empty() {
            Vec::new()
        } else {
            params[1..].to_vec()
        };

        commands.push(Command::new(command_type, line_num, args))
    }

    Ok(commands)
}

fn cut_funcs(commands: &mut Vec<Command>) -> Result<Vec<Function>, (ScriptError, usize)> {
    let mut functions: Vec<Function> = Vec::new();

    let mut now_func: Option<Function> = None;

    let mut index = 0;
    for command in commands.clone() {
        index += 1;

        match now_func.clone() {
            Some(func) => {
                index -= 1;
                commands.remove(index);

                if let CommandType::FuncEnd = command.command_type {
                    functions.push(func.clone());
                    now_func = None;
                } else {
                    now_func.as_mut().unwrap().commands.push(command);
                }
            }
            None => {
                if let CommandType::Func = command.command_type {
                    index -= 1;
                    commands.remove(index);

                    let name = command.args[1].clone();
                    let result_type =
                        VarType::from_name(&command.args[0]).map_err(|f| (f, command.line))?;
                    let mut parameters = HashMap::new();

                    let mut param_key: Option<String> = None;
                    for i in &command.args[2..] {
                        match &param_key {
                            Some(key) => {
                                parameters.insert(
                                    key.to_string(),
                                    VarType::from_name(i).map_err(|f| (f, command.line))?,
                                );
                                param_key = None;
                            }
                            None => {
                                param_key = Some(i.to_string());
                            }
                        }
                    }

                    now_func = Some(Function::new(name, result_type, parameters, Vec::new()));
                }
            }
        }
    }

    Ok(functions)
}

pub struct Script {
    pub commands: Vec<Command>,
    pub functions: Vec<Function>,
}

impl Script {
    pub fn parse(text: String) -> Result<Script, (ScriptError, usize)> {
        let lines = prepare_script(text);
        let mut commands = parse_commands(lines)?;
        let functions = cut_funcs(&mut commands)?;
        Ok(Script {
            commands,
            functions,
        })
    }
}
