#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    name: String,
    result_type: VarType,
    parameters: HashMap<String, VarType>,
    commands: Vec<Command>,
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

    pub fn execute_function(
        &self,
        function: Function,
        result_var: String,
        args: Vec<Variable>,
        globals: &mut HashMap<String, Variable>,
        is_global: bool,
    ) -> Result<(), ScriptError> {
        let mut locals: HashMap<String, Variable> = HashMap::new();
        let mut index = 0;
        for (k, _) in function.parameters {
            locals.insert(k, args[index].clone());
            index += 1;
        }
        locals.insert(
            "result".to_string(),
            Variable::empty_var(function.result_type)?,
        );

        let mut temp_vars: Vec<String> = Vec::new();

        for command in function.commands {
            if let CommandType::Return = command.command_type {
                return Ok(());
            }

            self.exec_command(command.clone(), false, &mut locals, &mut temp_vars)?;

            if let CommandType::TempVar = command.command_type {
                continue;
            }

            for ele in temp_vars.clone() {
                self.variables.remove(&ele);
            }
        }

        if result_var != "null" {
            self.variables
                .insert(result_var, locals.get("result").unwrap().clone());
        }

        Ok(())
    }
}
