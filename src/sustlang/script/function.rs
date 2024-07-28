use super::super::command::{Command, CommandType};
use super::super::other::Pohuy;
use super::super::var::{VarType, Variable};
use super::{RunningScript, ScriptError};

use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub name: String,
    pub result_type: VarType,
    pub parameters: HashMap<String, VarType>,
    pub commands: Vec<Command>,
}

impl Function {
    pub fn new(
        name: String,
        result_type: VarType,
        parameters: HashMap<String, VarType>,
        commands: Vec<Command>,
    ) -> Function {
        Function {
            name,
            result_type,
            parameters,
            commands,
        }
    }

    pub fn execute(
        &self,
        script: &mut RunningScript,
        result_var: String,
        args: Vec<Variable>,
        is_global: bool,
    ) -> Result<(), (ScriptError, Command)> {
        let mut locals: HashMap<String, Variable> = HashMap::new();
        let mut index = 0;
        for (k, _) in self.parameters.clone() {
            locals.insert(k, args[index].clone());
            index += 1;
        }
        locals.insert(
            "result".to_string(),
            Variable::empty_var(self.result_type.clone()).unwrap(),
        );

        let mut temp_vars: Vec<String> = Vec::new();

        for command in self.commands.clone() {
            if let CommandType::Return = command.command_type {
                return Ok(());
            }

            command.execute(script, is_global, &mut locals, &mut temp_vars)?;

            if let CommandType::TempVar = command.command_type {
                continue;
            }

            for ele in temp_vars.clone() {
                script
                    .drop_var(ele, &mut locals)
                    .map_err(|f| (f, command.clone()))
                    .pohuy();
            }
        }

        if result_var != "null" {
            script
                .set_var(
                    result_var,
                    locals.get("result").unwrap().clone(),
                    is_global,
                    false,
                    &mut locals,
                )
                .unwrap();
        }

        Ok(())
    }
}
