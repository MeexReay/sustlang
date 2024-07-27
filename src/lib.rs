use std::{
    collections::HashMap,
    hash::Hash,
    io::{Read, Write},
    ptr::hash,
    sync::{Arc, Mutex},
};

#[derive(PartialEq, Clone, Debug, Hash)]
pub enum VarType {
    Bool,
    String,
    Integer,
    Float,
    Char,
    List(Box<VarType>),
    Map(Box<VarType>, Box<VarType>),
    Optional(Box<VarType>),
    InStream,
    OutStream,
}

impl VarType {
    pub fn from_name(name: &str) -> Option<VarType> {
        if name.starts_with("map[") {
            let value_type = name[9..name.len() - 1].to_string();

            let mut key_type = String::new();
            let mut val_type = String::new();

            let mut val_tree = 0;
            let mut val_stat = 0;
            for char in value_type.chars() {
                if val_stat == 0 {
                    key_type.push(char);
                } else if val_stat == 1 {
                    val_type.push(char);
                }
                if char == ',' && val_tree == 0 {
                    val_stat += 1;
                }
                if char == '[' {
                    val_tree += 1;
                }
                if char == ']' {
                    val_tree -= 1;
                }
            }

            let key_type = Box::new(VarType::from_name(&key_type)?);
            let val_type = Box::new(VarType::from_name(&val_type)?);

            return Some(VarType::Map(key_type, val_type));
        }
        if name.starts_with("list[") {
            let value_type = name[5..name.len() - 1].to_string();
            let value_type = Box::new(VarType::from_name(&value_type)?);
            return Some(VarType::List(value_type));
        }
        if name.starts_with("optional[") {
            let value_type = name[9..name.len() - 1].to_string();
            let value_type = Box::new(VarType::from_name(&value_type)?);
            return Some(VarType::Optional(value_type));
        }

        match name {
            "bool" => Some(VarType::Bool),
            "string" => Some(VarType::String),
            "integer" => Some(VarType::Integer),
            "float" => Some(VarType::Float),
            "char" => Some(VarType::Char),
            "in_stream" => Some(VarType::InStream),
            "out_stream" => Some(VarType::OutStream),
            _ => None,
        }
    }
}

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
}

impl Variable {
    pub fn empty_var(var_type: VarType) -> Option<Variable> {
        match var_type {
            VarType::Bool => Some(Variable::Bool(VarType::Bool, None)),
            VarType::String => Some(Variable::String(VarType::String, None)),
            VarType::Integer => Some(Variable::Integer(VarType::Integer, None)),
            VarType::Float => Some(Variable::Float(VarType::Float, None)),
            VarType::Char => Some(Variable::Char(VarType::Char, None)),
            VarType::Optional(optional_type) => {
                Some(Variable::Optional(VarType::Optional(optional_type), None))
            }
            VarType::List(value_type) => Some(Variable::List(VarType::List(value_type), None)),
            VarType::Map(key_type, value_type) => {
                Some(Variable::Map(VarType::Map(key_type, value_type), None))
            }
            VarType::InStream => Some(Variable::InStream(VarType::InStream, None)),
            VarType::OutStream => Some(Variable::OutStream(VarType::OutStream, None)),
        }
    }

    pub fn parse_var(var_type: VarType, text: String) -> Option<Variable> {
        match var_type {
            VarType::Bool => Some(Variable::Bool(
                VarType::Bool,
                Some(match text.as_str() {
                    "true" => true,
                    "false" => false,
                    "1" => true,
                    "0" => false,
                    _ => {
                        return None;
                    }
                }),
            )),
            VarType::String => Some(Variable::String(VarType::String, Some(text))),
            VarType::Integer => Some(Variable::Integer(
                VarType::Integer,
                Some(match text.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return None;
                    }
                }),
            )),
            VarType::Float => Some(Variable::Float(
                VarType::Float,
                Some(match text.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return None;
                    }
                }),
            )),
            VarType::Char => Some(Variable::Char(
                VarType::Char,
                Some(match text.parse() {
                    Ok(i) => i,
                    Err(_) => {
                        return None;
                    }
                }),
            )),
            VarType::Optional(optional_type) => {
                if text.starts_with("[") && text.ends_with("]") {
                    let text = text[1..text.len() - 1].to_string();
                    Some(Variable::Optional(
                        VarType::Optional(optional_type.clone()),
                        match Self::parse_var(optional_type.clone().as_mut().clone(), text) {
                            Some(i) => Some(Some(Box::new(i))),
                            None => None,
                        },
                    ))
                } else if text.as_str() == "none" {
                    Some(Variable::Optional(
                        VarType::Optional(optional_type),
                        Some(None),
                    ))
                } else {
                    None
                }
            }
            _ => None,
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

#[derive(PartialEq, Clone, Debug, Copy, Hash)]
pub enum CommandType {
    InitVar,
    SetVar,
    TempVar,
    MoveVar,
    CopyVar,
    DropVar,
    HasVar,
    ToString,
    AddInt,
    AddFloat,
    SubStr,
    SubList,
    ListSize,
    Write,
    Read,
    ReadAll,
    ReadStr,
    ReadStrAll,
    For,
    ForMap,
    ForList,
    While,
    UseFunc,
    OpenFileIn,
    OpenFileOut,
    OpenTcpConnection,
    OpenTcpListener,
    HasOptional,
    WhenOptional,
    Sleep,
    Func,
    FuncEnd,
}

impl CommandType {
    pub fn from_name(name: &str) -> Option<CommandType> {
        match name {
            "INIT_VAR" => Some(CommandType::InitVar),
            "SET_VAR" => Some(CommandType::SetVar),
            "TEMP_VAR" => Some(CommandType::TempVar),
            "MOVE_VAR" => Some(CommandType::MoveVar),
            "COPY_VAR" => Some(CommandType::CopyVar),
            "DROP_VAR" => Some(CommandType::DropVar),
            "HAS_VAR" => Some(CommandType::HasVar),
            "TO_STRING" => Some(CommandType::ToString),
            "ADD_INT" => Some(CommandType::AddInt),
            "ADD_FLOAT" => Some(CommandType::AddFloat),
            "SUB_STR" => Some(CommandType::SubStr),
            "SUB_LIST" => Some(CommandType::SubList),
            "LIST_SIZE" => Some(CommandType::ListSize),
            "WRITE" => Some(CommandType::Write),
            "READ" => Some(CommandType::Read),
            "READ_ALL" => Some(CommandType::ReadAll),
            "READ_STR" => Some(CommandType::ReadStr),
            "READ_STR_ALL" => Some(CommandType::ReadStrAll),
            "FOR" => Some(CommandType::For),
            "FOR_MAP" => Some(CommandType::ForMap),
            "FOR_LIST" => Some(CommandType::ForList),
            "WHILE" => Some(CommandType::While),
            "USE_FUNC" => Some(CommandType::UseFunc),
            "OPEN_FILE_IN" => Some(CommandType::OpenFileIn),
            "OPEN_FILE_OUT" => Some(CommandType::OpenFileOut),
            "OPEN_TCP_CONNECTION" => Some(CommandType::OpenTcpConnection),
            "OPEN_TCP_LISTENER" => Some(CommandType::OpenTcpListener),
            "HAS_OPTIONAL" => Some(CommandType::HasOptional),
            "WHEN_OPTIONAL" => Some(CommandType::WhenOptional),
            "SLEEP" => Some(CommandType::Sleep),
            "FUNC" => Some(CommandType::Func),
            "FUNC_END" => Some(CommandType::FuncEnd),
            _ => None,
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Command {
    command_type: CommandType,
    args: Vec<String>,
}

impl Command {
    pub fn new(command_type: CommandType, args: Vec<String>) -> Command {
        Command { command_type, args }
    }
}

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
}

fn prepare_script(text: String) -> Vec<String> {
    text.lines()
        .map(|s| match s.split_once("#") {
            Some(s) => s.0,
            None => s,
        })
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn parse_commands(lines: Vec<String>) -> Vec<Command> {
    let mut commands = Vec::new();

    for line in lines {
        let params: Vec<String> = line.split(" ").map(|v| v.to_string()).collect();

        let command_type = match CommandType::from_name(&params[0]) {
            Some(i) => i,
            None => {
                continue;
            }
        };

        let args = if params.is_empty() {
            Vec::new()
        } else {
            params[1..].to_vec()
        };

        commands.push(Command::new(command_type, args))
    }

    commands
}

fn cut_funcs(commands: &mut Vec<Command>) -> Vec<Function> {
    let mut functions: Vec<Function> = Vec::new();

    let mut now_func = None;

    let mut index = 0;
    for command in commands.clone() {
        match now_func.clone() {
            Some(mut func) => {
                if command.command_type == CommandType::FuncEnd {
                    commands.remove(index);
                    index -= 1;

                    functions.push(func);
                    now_func = None;
                } else {
                    commands.remove(index);
                    index -= 1;

                    func.commands.push(command);
                }
            }
            None => {
                if command.command_type == CommandType::Func {
                    commands.remove(index);
                    index -= 1;

                    let name = command.args[0].clone();
                    let result_type = match VarType::from_name(&command.args[1]) {
                        Some(i) => i,
                        None => {
                            continue;
                        }
                    };
                    let mut parameters = HashMap::new();

                    let mut param_key: Option<String> = None;
                    for i in &command.args[2..] {
                        match &param_key {
                            Some(key) => {
                                parameters.insert(
                                    key.to_string(),
                                    match VarType::from_name(i) {
                                        Some(i) => i,
                                        None => {
                                            continue;
                                        }
                                    },
                                );
                                param_key = None;
                            }
                            None => {
                                param_key = Some(i.to_string());
                            }
                        }
                    }

                    now_func = Some(Function::new(name, result_type, parameters, Vec::new()))
                }
            }
        }

        index += 1;
    }

    functions
}

pub struct Script {
    commands: Vec<Command>,
    functions: Vec<Function>,
}

impl Script {
    pub fn parse(text: String) -> Script {
        let lines = prepare_script(text);
        let mut commands = parse_commands(lines);
        let functions = cut_funcs(&mut commands);
        Script {
            commands,
            functions,
        }
    }
}

pub struct RunningScript {
    commands: Vec<Command>,
    functions: Vec<Function>,
    variables: HashMap<String, Variable>,
}

impl RunningScript {
    pub fn new(script: Script) -> RunningScript {
        RunningScript {
            commands: script.commands,
            functions: script.functions,
            variables: HashMap::new(),
        }
    }

    pub fn set_standard_vars(
        &mut self,
        args: Vec<String>,
        cout: Box<dyn Write>,
        cin: Box<dyn Read>,
    ) {
        self.variables.insert(
            String::from("args"),
            Variable::List(
                VarType::List(Box::new(VarType::String)),
                Some(
                    args.iter()
                        .map(|s| Variable::String(VarType::String, Some(s.to_string())))
                        .collect(),
                ),
            ),
        );
        self.variables.insert(
            String::from("cout"),
            Variable::OutStream(VarType::OutStream, Some(Arc::new(Mutex::new(cout)))),
        );
        self.variables.insert(
            String::from("cin"),
            Variable::InStream(VarType::InStream, Some(Arc::new(Mutex::new(cin)))),
        );
    }

    pub fn get_var(
        &mut self,
        name: String,
        locals: &mut HashMap<String, Variable>,
    ) -> Option<Variable> {
        let mut var: Option<Variable> = None;

        for part in name.split(".") {
            match &var {
                Some(v) => match v {
                    Variable::List(_, list) => match list {
                        Some(list) => {
                            let index: usize = part.parse().unwrap();
                            var = match list.get(index) {
                                Some(i) => Some(i.clone()),
                                None => {
                                    return None;
                                }
                            }
                        }
                        None => {
                            return None;
                        }
                    },
                    Variable::Map(map_type, map) => match map {
                        Some(map) => {
                            var = map
                                .get(
                                    &match Variable::parse_var(map_type.clone(), part.to_string()) {
                                        Some(i) => i,
                                        None => {
                                            return None;
                                        }
                                    },
                                )
                                .cloned();
                        }
                        None => {
                            return None;
                        }
                    },
                    _ => {}
                },
                None => match self.variables.get(part) {
                    Some(i) => var = Some(i.clone()),
                    None => match locals.get(part) {
                        Some(i) => var = Some(i.clone()),
                        None => {
                            return None;
                        }
                    },
                },
            }
        }

        var
    }

    pub fn exec_command(
        &mut self,
        command: Command,
        global: bool,
        locals: &mut HashMap<String, Variable>,
    ) {
        match command.command_type {
            CommandType::InitVar => {
                // TODO: for hello world
            }
            CommandType::SetVar => {
                // TODO: for hello world
            }
            CommandType::TempVar => {
                // TODO: for hello world
            }
            CommandType::MoveVar => {}
            CommandType::CopyVar => {}
            CommandType::DropVar => {}
            CommandType::HasVar => {}
            CommandType::ToString => {}
            CommandType::AddInt => {}
            CommandType::AddFloat => {}
            CommandType::SubStr => {}
            CommandType::SubList => {}
            CommandType::ListSize => {}
            CommandType::Write => {
                // TODO: for hello world
            }
            CommandType::Read => {}
            CommandType::ReadAll => {}
            CommandType::ReadStr => {}
            CommandType::ReadStrAll => {}
            CommandType::For => {}
            CommandType::ForMap => {}
            CommandType::ForList => {}
            CommandType::While => {}
            CommandType::UseFunc => {}
            CommandType::OpenFileIn => {}
            CommandType::OpenFileOut => {}
            CommandType::OpenTcpConnection => {}
            CommandType::OpenTcpListener => {}
            CommandType::HasOptional => {}
            CommandType::WhenOptional => {}
            CommandType::Sleep => {}
            _ => {}
        }
    }

    pub fn exec_function(&mut self, function: Function, result_var: String, args: Vec<Variable>) {
        let mut locals: HashMap<String, Variable> = HashMap::new();
        let mut index = 0;
        for (k, _) in function.parameters {
            locals.insert(k, args[index].clone());
            index += 1;
        }
        locals.insert(
            "result".to_string(),
            Variable::empty_var(function.result_type).unwrap(),
        );

        for command in function.commands.clone() {
            self.exec_command(command, false, &mut locals);
        }

        self.variables
            .insert(result_var, locals.get("result").unwrap().clone());
    }

    pub fn run(&mut self) {
        for command in self.commands.clone() {
            self.exec_command(command, true, &mut HashMap::new());
        }
    }
}
