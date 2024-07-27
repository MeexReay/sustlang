use std::{
    char::EscapeDebug,
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
        }
    }

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
    /// Инициализировать переменную `name_var` с типом `type_var`
    ///
    /// Название: INIT_VAR
    /// Параметры: `type_var`, `name_var`
    InitVar,

    /// Установить значение переменной в `name_var`
    ///
    /// Название: SET_VAR
    /// Параметры: `name_var`, `value_var`
    SetVar,

    /// Переменная `name_var` инициализируется с типом `type_var` и присваивается `value_var`, переменная дропается через одну команду
    ///
    /// Название: TEMP_VAR
    /// Параметры: `type_var`, `name_var`, `value_var`
    TempVar,

    /// Переместить значение переменной с `source_var` в `target_var`
    ///
    /// Название: MOVE_VAR
    /// Параметры: `source_var`, `target_var`
    MoveVar,

    /// Скопировать значение переменной с `source_var` в `target_var`
    ///
    /// Название: COPY_VAR
    /// Параметры: `source_var`, `target_var`
    CopyVar,

    /// Дропнуть переменную `name_var`
    ///
    /// Название: DROP_VAR
    /// Параметры: `name_var`
    DropVar,

    /// В переменную `result_var` записывается `bool` существует ли переменная `name_var`
    ///
    /// Название: HAS_VAR
    /// Параметры: `name_var`, `result_var`
    HasVar,

    /// Скопировать значение переменной с `source_var` в `target_var`, переводя в строку
    ///
    /// Название: TO_STRING
    /// Параметры: `source_var`, `target_var`
    ToString,

    /// Прибавить к числу `var` значение `other_var`
    ///
    /// Название: ADD_INT
    /// Параметры: `var`, `other_var`
    AddInt,

    /// Прибавить к числу `var` значение `other_var`
    ///
    /// Название: ADD_FLOAT
    /// Параметры: `var`, `other_var`
    AddFloat,

    /// Прибавить к числу `var` значение `other_var`
    ///
    /// Название: ADD_STR
    /// Параметры: `var`, `other_var`
    AddStr,

    /// Сделать подстроку из строки `str_var` и сохранить туда же
    ///
    /// Название: SUB_STR
    /// Параметры: `str_var`, `start_index`, `end_index`
    SubStr,

    /// Сделать подсписок из списка `list_var` и сохранить туда же
    ///
    /// Название: SUB_LIST
    /// Параметры: `list_var`, `start_index`, `end_index`
    SubList,

    /// Получить размер списка и записать в переменную `result_var` типа `int`
    ///
    /// Название: LIST_SIZE
    /// Параметры: `list_var`, `result_var`
    ListSize,

    /// Вывести переменную `name_var` в `stream_var`
    ///
    /// Название: WRITE
    /// Параметры: `name_var`, `stream_var`
    Write,

    /// Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `list[char]`
    ///
    /// Название: READ
    /// Параметры: `name_var`, `size_var`, `stream_var`
    Read,

    /// Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `list[char]`
    ///
    /// Название: READ_ALL
    /// Параметры: `name_var`, `stream_var`
    ReadAll,

    /// Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `string`
    ///
    /// Название: READ_STR
    /// Параметры: `name_var`, `size_var`, `stream_var`
    ReadStr,

    /// Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `string`
    ///
    /// Название: READ_STR_ALL
    /// Параметры: `name_var`, `stream_var`
    ReadStrAll,

    /// Функция `func` (с единственным аргументом с типом `int`) вызывается с `start_index` до `end_index` включительно, `start_index` и `end_index` это названия переменных
    ///
    /// Название: FOR
    /// Параметры: `func(int)`, `start_index`, `end_index`
    For,

    /// Функция `func` вызывается для каждого `key`, `value` переменной `map_var`
    ///
    /// Название: FOR_MAP
    /// Параметры: `func(any, any)`, `map_var`
    ForMap,

    /// Функция `func` вызывается для каждого предмета переменной `list_var`
    ///
    /// Название: FOR_LIST
    /// Параметры: `func(any)`, `list_var`
    ForList,

    /// Функция `func` (с результатом `bool`) вызывается, пока функция выдает `true`
    ///
    /// Название: WHILE
    /// Параметры: `func -> bool`
    While,

    /// Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для чтения и записать стрим для чтения в переменную `stream_var`
    ///
    /// Название: OPEN_FILE_IN
    /// Параметры: `path_var`, `stream_var`
    OpenFileIn,

    /// Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для записи и записать стрим для записи в переменную `stream_var`
    ///
    /// Название: OPEN_FILE_OUT
    /// Параметры: `path_var`, `stream_var`
    OpenFileOut,

    /// Подключиться по `addr_var:port_var` (`addr_var: string`, `port_var: int`, `in_stream: in_stream`, `out_stream: out_stream` - переменные) и записать стримы для чтения и записи в `in_stream` и `out_stream`
    ///
    /// Название: OPEN_TCP_CONNECTION
    /// Параметры: `addr_var`, `port_var`, `in_stream`, `out_stream`
    OpenTcpConnection,

    /// Ожидание подключений с `addr_var:port_var` (`addr_var: string`, `port_var: int` - переменные), при подключениях вызывается функция `accept_func`
    ///
    /// Название: OPEN_TCP_LISTENER
    /// Параметры: `addr_var`, `port_var`, `accept_func(string,int,in_stream,out_stream)`
    OpenTcpListener,

    /// Ждать миллисекунд из переменной `time_var` (тип переменной: int)
    ///
    /// Название: SLEEP
    /// Параметры: `time_var`
    Sleep,

    /// Вызвать функцию `func` в новом потоке
    ///
    /// Название: NEW_THREAD
    /// Параметры: `func`
    NewThread,

    /// Функция `func` вызывается с переданными аргументами и устанавливает результат в переменную `result_var`
    ///
    /// Название: USE_FUNC
    /// Параметры: `func_name`, `result_var`, `[arg_var1] ... [arg_varN]`
    UseFunc,

    /// Создать функцию с типом результата `result_type`, названием `func_name` и аргументами `[arg_name_1 arg_type] ... [arg_name_N arg_type]`. Установить результат переменной можно изменив переменную `result` внутри функции. Все команды после этой и до `FUNC_END` будут командами функции. Функции внутри функций не могут быть.
    ///
    /// Название: FUNC
    /// Параметры: `result_type`, `func_name`, `[arg_name_1 arg_type] ... [arg_name_N arg_type]`
    Func,

    /// Досрочно выйти из функции, также работает как выход из скрипта
    ///
    /// Название: RETURN
    Return,

    /// Маркер, что команды функции тут заканчиваются
    ///
    /// Название: FUNC_END
    FuncEnd,

    /// Узнать, равен ли `var` и `other_var` записать результат в `result_var`
    ///
    /// Название: EQUALS
    /// Параметры: `var`, `other_var`, `result_var`
    Equals,

    /// Узнать, больше ли в `var` чем в `other_var` записать результат в `result_var`
    ///
    /// Название: MORE
    /// Параметры: `var`, `other_var`, `result_var`
    More,

    /// Узнать, меньше ли в `var` чем в `other_var` записать результат в `result_var`
    ///
    /// Название: LESS
    /// Параметры: `var`, `other_var`, `result_var`
    Less,

    /// Если `var` и `other_var` равны `true`, то результат `true`, иначе `false`, записать результат в `result_var`
    ///
    /// Название: AND
    /// Параметры: `var`, `other_var`, `result_var`
    And,

    /// Если `var` или `other_var` равен `true`, то результат `true`, иначе `false`, записать результат в `result_var`
    ///
    /// Название: OR
    /// Параметры: `var`, `other_var`, `result_var`
    Or,

    /// Если `var` равен `true` то вызвать функцию `func`
    ///
    /// Название: IF
    /// Параметры: `bool_var`, `func`
    If,

    /// Узнать, имеет ли строка `var` в себе подстроку `substring` и записать результат в `result_var`
    ///
    /// Название: HAS_STR
    /// Параметры: `string_var`, `substring`, `result_var`
    HasStr,

    /// Узнать, имеет ли список `list_var` значение `item_var` и записать результат в `result_var`
    ///
    /// Название: HAS_ITEM
    /// Параметры: `list_var`, `item_var`, `result_var`
    HasItem,

    /// Узнать, имеет ли мап `map_var` поле с ключом `key_var` и значением `value_var` и записать результат в `result_var`
    ///
    /// Название: HAS_ENTRY
    /// Параметры: `map_var`, `key_var`, `value_var`, `result_var`
    HasEntry,

    /// Узнать, имеет ли мап `map_var` поле с ключом `key_var` и записать результат в `result_var`
    ///
    /// Название: HAS_KEY
    /// Параметры: `map_var`, `key_var`, `result_var`
    HasKey,

    /// Узнать, имеет ли мап `map_var` поле с значением `value_var` и записать результат в `result_var`
    ///
    /// Название: HAS_VALUE
    /// Параметры: `map_var`, `value_var`, `result_var`
    HasValue,

    /// Узнать, имеет ли данные опшнл `optional_var` и записать результат в `result_var`
    ///
    /// Название: HAS_OPTIONAL
    /// Параметры: `optional_var`, `result_var`
    HasOptional,
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
            "ADD_STR" => Some(CommandType::AddStr),
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
            "OPEN_FILE_IN" => Some(CommandType::OpenFileIn),
            "OPEN_FILE_OUT" => Some(CommandType::OpenFileOut),
            "OPEN_TCP_CONNECTION" => Some(CommandType::OpenTcpConnection),
            "OPEN_TCP_LISTENER" => Some(CommandType::OpenTcpListener),
            "SLEEP" => Some(CommandType::Sleep),
            "NEW_THREAD" => Some(CommandType::NewThread),
            "USE_FUNC" => Some(CommandType::UseFunc),
            "FUNC" => Some(CommandType::Func),
            "FUNC_END" => Some(CommandType::FuncEnd),
            "RETURN" => Some(CommandType::Return),
            "EQUALS" => Some(CommandType::Equals),
            "MORE" => Some(CommandType::More),
            "LESS" => Some(CommandType::Less),
            "AND" => Some(CommandType::And),
            "OR" => Some(CommandType::Or),
            "IF" => Some(CommandType::If),
            "HAS_STR" => Some(CommandType::HasStr),
            "HAS_ITEM" => Some(CommandType::HasItem),
            "HAS_ENTRY" => Some(CommandType::HasEntry),
            "HAS_KEY" => Some(CommandType::HasKey),
            "HAS_VALUE" => Some(CommandType::HasValue),
            "HAS_OPTIONAL" => Some(CommandType::HasOptional),
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
        .filter(|s| !s.trim_matches(' ').is_empty())
        .map(|s| s.trim_end_matches(" ").to_string())
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
                    let result_type = match VarType::from_name(&command.args[0]) {
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

                    now_func = Some(Function::new(name, result_type, parameters, Vec::new()));
                }
            }
        }
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

    pub fn get_function(&self, name: String) -> Option<Function> {
        for func in &self.functions {
            if func.name == name {
                return Some(func.clone());
            }
        }
        None
    }

    pub fn exec_commands(
        &mut self,
        commands: Vec<Command>,
        global: bool,
        locals: &mut HashMap<String, Variable>,
    ) {
        let mut temp_vars: Vec<String> = Vec::new();

        for command in commands {
            match command.command_type {
                CommandType::InitVar => {
                    let type_var = command.args[0].clone();
                    let type_var = VarType::from_name(&type_var).unwrap();
                    let name_var = command.args[1].clone();

                    self.variables
                        .insert(name_var, Variable::empty_var(type_var).unwrap());
                }
                CommandType::SetVar => {
                    let name_var = command.args[0].clone();
                    let value_var = command.args[1..].join(" ");

                    let type_var = self.get_var(name_var.clone(), locals).unwrap().get_type();
                    let var = Variable::parse_var(type_var, value_var).unwrap();

                    if self.variables.contains_key(&name_var) {
                        self.variables.insert(name_var, var);
                    } else if locals.contains_key(&name_var) {
                        locals.insert(name_var, var);
                    }
                }
                CommandType::TempVar => {
                    let type_var = command.args[0].clone();
                    let name_var = command.args[1].clone();
                    let value_var = command.args[2..].join(" ");

                    self.variables.insert(
                        name_var.clone(),
                        Variable::parse_var(VarType::from_name(&type_var).unwrap(), value_var)
                            .unwrap(),
                    );

                    temp_vars.push(name_var);

                    continue;
                }
                CommandType::MoveVar => {
                    let source_var = command.args[0].clone();
                    let target_var = command.args[1].clone();

                    let var = self.get_var(source_var.clone(), locals).unwrap();

                    if self.variables.contains_key(&source_var) {
                        self.variables.remove(&source_var);
                    } else if locals.contains_key(&source_var) {
                        locals.remove(&source_var);
                    }

                    if self.variables.contains_key(&target_var) {
                        self.variables.insert(target_var, var);
                    } else if locals.contains_key(&target_var) {
                        locals.insert(target_var, var);
                    }
                }
                CommandType::CopyVar => {
                    let source_var = command.args[0].clone();
                    let target_var = command.args[1].clone();

                    let var = self.get_var(source_var.clone(), locals).unwrap();

                    if self.variables.contains_key(&target_var) {
                        self.variables.insert(target_var, var);
                    } else if locals.contains_key(&target_var) {
                        locals.insert(target_var, var);
                    }
                }
                CommandType::DropVar => {
                    let name_var = command.args[0].clone();

                    if self.variables.contains_key(&name_var) {
                        self.variables.remove(&name_var);
                    } else if locals.contains_key(&name_var) {
                        locals.remove(&name_var);
                    }
                }
                CommandType::HasVar => {
                    let name_var = command.args[0].clone();
                    let result_var = command.args[1].clone();

                    let result =
                        self.variables.contains_key(&name_var) || locals.contains_key(&name_var);

                    if self.variables.contains_key(&result_var) {
                        self.variables
                            .insert(name_var, Variable::Bool(VarType::Bool, Some(result)));
                    } else if locals.contains_key(&result_var) {
                        locals.insert(name_var, Variable::Bool(VarType::Bool, Some(result)));
                    }
                }
                CommandType::AddStr => {
                    let var_name = command.args[0].clone();
                    let other_var = command.args[1].clone();

                    let other_var = self.get_var(other_var.clone(), locals).unwrap();
                    let other_var = match other_var {
                        Variable::String(_, s) => match s {
                            Some(s) => s,
                            None => {
                                continue;
                            }
                        },
                        Variable::Char(_, c) => match c {
                            Some(c) => String::from_utf8(vec![c]).unwrap(),
                            None => {
                                continue;
                            }
                        },
                        Variable::List(list_type, list) => match list_type {
                            VarType::List(mut list_type) => match list_type.as_mut() {
                                VarType::Char => String::from_utf8(
                                    match list {
                                        Some(i) => i,
                                        None => {
                                            continue;
                                        }
                                    }
                                    .iter()
                                    .map(|f| match f {
                                        Variable::Char(_, c) => match c {
                                            Some(c) => *c,
                                            None => 0u8,
                                        },
                                        _ => 0u8,
                                    })
                                    .collect(),
                                )
                                .unwrap(),
                                _ => {
                                    continue;
                                }
                            },
                            _ => {
                                continue;
                            }
                        },
                        _ => {
                            continue;
                        }
                    };

                    let var = self.get_var(var_name.clone(), locals).unwrap();
                    let var = match var {
                        Variable::String(t, s) => match s {
                            Some(s) => Variable::String(t, Some(s + &other_var)),
                            None => {
                                continue;
                            }
                        },
                        _ => {
                            continue;
                        }
                    };

                    if self.variables.contains_key(&var_name) {
                        self.variables.insert(var_name, var);
                    } else if locals.contains_key(&var_name) {
                        locals.insert(var_name, var);
                    }
                }
                CommandType::Write => {
                    let name_var = command.args[0].clone();
                    let stream_var = command.args[1].clone();

                    let text = self.get_var(name_var.clone(), locals).unwrap();
                    let text: Vec<u8> = match text {
                        Variable::List(list_type, list) => match list_type {
                            VarType::List(mut list_type) => match list_type.as_mut() {
                                VarType::Char => match list {
                                    Some(i) => i,
                                    None => {
                                        continue;
                                    }
                                }
                                .iter()
                                .map(|f| match f {
                                    Variable::Char(_, c) => match c {
                                        Some(c) => *c,
                                        None => 0u8,
                                    },
                                    _ => 0u8,
                                })
                                .collect(),
                                _ => {
                                    continue;
                                }
                            },
                            _ => {
                                continue;
                            }
                        },
                        Variable::String(_, text) => match text {
                            Some(i) => i.as_bytes().to_vec(),
                            None => {
                                continue;
                            }
                        },
                        _ => {
                            continue;
                        }
                    };
                    let stream = self.get_var(stream_var.clone(), locals).unwrap();

                    match stream {
                        Variable::OutStream(_, stream) => match stream {
                            Some(stream) => {
                                stream.lock().unwrap().write_all(&text).unwrap();
                            }
                            None => {}
                        },
                        _ => {}
                    }
                }
                CommandType::UseFunc => {
                    let func_name = command.args[0].clone();
                    let result_name = command.args[1].clone();
                    let args_names = command.args[2..].to_vec();

                    let func = self.get_function(func_name).unwrap();
                    let args: Vec<Variable> = args_names
                        .iter()
                        .map(|f| self.get_var(f.to_string(), locals).unwrap())
                        .collect();

                    self.exec_function(func, result_name, args)
                }
                CommandType::Return => {
                    return;
                }
                CommandType::ToString => {}
                CommandType::AddInt => {}
                CommandType::AddFloat => {}
                CommandType::SubStr => {}
                CommandType::SubList => {}
                CommandType::ListSize => {}
                CommandType::Read => {}
                CommandType::ReadAll => {}
                CommandType::ReadStr => {}
                CommandType::ReadStrAll => {}
                CommandType::For => {}
                CommandType::ForMap => {}
                CommandType::ForList => {}
                CommandType::While => {}
                CommandType::OpenFileIn => {}
                CommandType::OpenFileOut => {}
                CommandType::OpenTcpConnection => {}
                CommandType::OpenTcpListener => {}
                CommandType::Sleep => {}
                CommandType::NewThread => {}
                CommandType::Equals => {}
                CommandType::More => {}
                CommandType::Less => {}
                CommandType::And => {}
                CommandType::Or => {}
                CommandType::If => {}
                CommandType::HasStr => {}
                CommandType::HasItem => {}
                CommandType::HasEntry => {}
                CommandType::HasKey => {}
                CommandType::HasValue => {}
                CommandType::HasOptional => {}
                _ => {}
            }

            for ele in temp_vars.clone() {
                self.variables.remove(&ele);
            }
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

        self.exec_commands(function.commands, false, &mut locals);

        self.variables
            .insert(result_var, locals.get("result").unwrap().clone());
    }

    pub fn run(&mut self) {
        self.exec_commands(self.commands.clone(), true, &mut HashMap::new());
    }
}
