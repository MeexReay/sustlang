use std::{
    char::EscapeDebug,
    collections::HashMap,
    error::Error,
    fmt::Display,
    hash::Hash,
    io::{Read, Write},
    ptr::hash,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub enum ScriptError {
    ParseVarError,
    TypeUnknownError,
    CommandUnknownError(usize),
    CommandArgsInvalidError(usize),
    UnknownVarError,
    TypeMismatchError,
}

impl Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("some error ez")
    }
}
impl Error for ScriptError {}

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
    Null,
}

impl VarType {
    pub fn from_name(name: &str) -> Result<VarType, ScriptError> {
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

            return Ok(VarType::Map(key_type, val_type));
        }
        if name.starts_with("list[") {
            let value_type = name[5..name.len() - 1].to_string();
            let value_type = Box::new(VarType::from_name(&value_type)?);
            return Ok(VarType::List(value_type));
        }
        if name.starts_with("optional[") {
            let value_type = name[9..name.len() - 1].to_string();
            let value_type = Box::new(VarType::from_name(&value_type)?);
            return Ok(VarType::Optional(value_type));
        }

        match name {
            "bool" => Ok(VarType::Bool),
            "string" => Ok(VarType::String),
            "integer" => Ok(VarType::Integer),
            "float" => Ok(VarType::Float),
            "char" => Ok(VarType::Char),
            "in_stream" => Ok(VarType::InStream),
            "out_stream" => Ok(VarType::OutStream),
            "null" => Ok(VarType::Null),
            _ => Err(ScriptError::TypeUnknownError),
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

    pub fn empty_var(var_type: VarType) -> Result<Variable, ScriptError> {
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

#[derive(PartialEq, Clone, Debug, Copy, Hash)]
pub enum CommandType {
    /// Инициализировать переменную `name_var` с типом `type_var`
    ///
    /// Название: INIT_VAR \
    /// Параметры: `type_var`, `name_var`
    InitVar,

    /// Установить значение переменной в `name_var`
    ///
    /// Название: SET_VAR \
    /// Параметры: `name_var`, `value_var`
    SetVar,

    /// Переменная `name_var` инициализируется с типом `type_var` и присваивается `value_var`, переменная дропается через одну команду
    ///
    /// Название: TEMP_VAR \
    /// Параметры: `type_var`, `name_var`, `value_var`
    TempVar,

    /// Переместить значение переменной с `source_var` в `target_var`
    ///
    /// Название: MOVE_VAR \
    /// Параметры: `source_var`, `target_var`
    MoveVar,

    /// Скопировать значение переменной с `source_var` в `target_var`
    ///
    /// Название: COPY_VAR \
    /// Параметры: `source_var`, `target_var`
    CopyVar,

    /// Дропнуть переменную `name_var`
    ///
    /// Название: DROP_VAR \
    /// Параметры: `name_var`
    DropVar,

    /// В переменную `result_var` записывается `bool` существует ли переменная `name_var`
    ///
    /// Название: HAS_VAR \
    /// Параметры: `name_var`, `result_var`
    HasVar,

    /// Скопировать значение переменной с `source_var` в `result_var`, переводя в `string`
    ///
    /// Название: TO_STRING \
    /// Параметры: `source_var`, `result_var`
    ToString,

    /// Скопировать строку `str_var` в `result_var`, переводя в `list[char]`
    ///
    /// Название: TO_BYTES \
    /// Параметры: `source_var`, `result_var`
    ToBytes,

    /// Скопировать строку `source_var` (тип переменной: `string`/`integer`) в `result_var`, переводя в `char`
    ///
    /// Название: TO_CHAR \
    /// Параметры: `source_var`, `result_var`
    ToChar,

    /// Скопировать строку `source_var` (тип переменной: `string`/`char`) в `result_var`, переводя в `integer`
    ///
    /// Название: TO_INTEGER \
    /// Параметры: `source_var`, `result_var`
    ToInteger,

    /// Скопировать строку `source_var` в `result_var`, переводя в `float`
    ///
    /// Название: TO_FLOAT \
    /// Параметры: `source_var`, `result_var`
    ToFloat,

    /// Скопировать строку `source_var` (тип переменной: `string`/`integer`) в `result_var`, переводя в `bool`
    ///
    /// Название: TO_BOOL \
    /// Параметры: `source_var`, `result_var`
    ToBool,

    /// Скопировать символ из строки `str_var` по индексу `index_var` и записать в `result_var`
    ///
    /// Название: GET_SYMBOL \
    /// Параметры: `str_var`, `index_var`, `result_var`
    GetSymbol,

    /// Прибавить к числу `var` значение `other_var`
    ///
    /// Название: ADD_INT \
    /// Параметры: `var`, `other_var`
    AddInt,

    /// Прибавить к числу `var` значение `other_var`
    ///
    /// Название: ADD_FLOAT \
    /// Параметры: `var`, `other_var`
    AddFloat,

    /// Прибавить к числу `var` значение `other_var`
    ///
    /// Название: ADD_STR \
    /// Параметры: `var`, `other_var`
    AddStr,

    /// Сделать подстроку из строки `str_var` и сохранить туда же
    ///
    /// Название: SUB_STR \
    /// Параметры: `str_var`, `start_index`, `end_index`
    SubStr,

    /// Сделать подсписок из списка `list_var` и сохранить туда же
    ///
    /// Название: SUB_LIST \
    /// Параметры: `list_var`, `start_index`, `end_index`
    SubList,

    /// Получить размер списка и записать в переменную `result_var` типа `int`
    ///
    /// Название: LIST_SIZE \
    /// Параметры: `list_var`, `result_var`
    ListSize,

    /// Вывести переменную `name_var` в `stream_var`
    ///
    /// Название: WRITE \
    /// Параметры: `name_var`, `stream_var`
    Write,

    /// Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `list[char]`
    ///
    /// Название: READ \
    /// Параметры: `name_var`, `size_var`, `stream_var`
    Read,

    /// Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `list[char]`
    ///
    /// Название: READ_ALL \
    /// Параметры: `name_var`, `stream_var`
    ReadAll,

    /// Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `string`
    ///
    /// Название: READ_STR \
    /// Параметры: `name_var`, `size_var`, `stream_var`
    ReadStr,

    /// Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `string`
    ///
    /// Название: READ_STR_ALL \
    /// Параметры: `name_var`, `stream_var`
    ReadStrAll,

    /// Функция `func` (с единственным аргументом с типом `int`) вызывается с `start_index` до `end_index` включительно, `start_index` и `end_index` это названия переменных
    ///
    /// Название: FOR \
    /// Параметры: `func(int)`, `start_index`, `end_index`
    For,

    /// Функция `func` вызывается для каждого `key`, `value` переменной `map_var`
    ///
    /// Название: FOR_MAP \
    /// Параметры: `func(any, any)`, `map_var`
    ForMap,

    /// Функция `func` вызывается для каждого предмета переменной `list_var`
    ///
    /// Название: FOR_LIST \
    /// Параметры: `func(any)`, `list_var`
    ForList,

    /// Функция `func` (с результатом `bool`) вызывается, пока функция выдает `true`
    ///
    /// Название: WHILE \
    /// Параметры: `func -> bool`
    While,

    /// Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для чтения и записать стрим для чтения в переменную `stream_var`
    ///
    /// Название: OPEN_FILE_IN \
    /// Параметры: `path_var`, `stream_var`
    OpenFileIn,

    /// Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для записи и записать стрим для записи в переменную `stream_var`
    ///
    /// Название: OPEN_FILE_OUT \
    /// Параметры: `path_var`, `stream_var`
    OpenFileOut,

    /// Подключиться по `addr_var:port_var` (`addr_var: string`, `port_var: int`, `in_stream: in_stream`, `out_stream: out_stream` - переменные) и записать стримы для чтения и записи в `in_stream` и `out_stream`
    ///
    /// Название: OPEN_TCP_CONNECTION \
    /// Параметры: `addr_var`, `port_var`, `in_stream`, `out_stream`
    OpenTcpConnection,

    /// Ожидание подключений с `addr_var:port_var` (`addr_var: string`, `port_var: int` - переменные), при подключениях вызывается функция `accept_func`
    ///
    /// Название: OPEN_TCP_LISTENER \
    /// Параметры: `addr_var`, `port_var`, `accept_func(string,int,in_stream,out_stream)`
    OpenTcpListener,

    /// Ждать миллисекунд из переменной `time_var` (тип переменной: int)
    ///
    /// Название: SLEEP \
    /// Параметры: `time_var`
    Sleep,

    /// Вызвать функцию `func` в новом потоке
    ///
    /// Название: NEW_THREAD \
    /// Параметры: `func`
    NewThread,

    /// Функция `func` вызывается с переданными аргументами и устанавливает результат в переменную `result_var`
    ///
    /// Название: USE_FUNC \
    /// Параметры: `func_name`, `result_var`, `[arg_var1] ... [arg_varN]`
    UseFunc,

    /// Создать функцию с типом результата `result_type`, названием `func_name` и аргументами `[arg_name_1 arg_type] ... [arg_name_N arg_type]`. Установить результат переменной можно изменив переменную `result` внутри функции. Все команды после этой и до `FUNC_END` будут командами функции. Функции внутри функций не могут быть.
    ///
    /// Название: FUNC \
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
    /// Название: EQUALS \
    /// Параметры: `var`, `other_var`, `result_var`
    Equals,

    /// Узнать, больше ли в `var` чем в `other_var` записать результат в `result_var`
    ///
    /// Название: MORE \
    /// Параметры: `var`, `other_var`, `result_var`
    More,

    /// Узнать, меньше ли в `var` чем в `other_var` записать результат в `result_var`
    ///
    /// Название: LESS \
    /// Параметры: `var`, `other_var`, `result_var`
    Less,

    /// Если `var` и `other_var` равны `true`, то результат `true`, иначе `false`, записать результат в `result_var`
    ///
    /// Название: AND \
    /// Параметры: `var`, `other_var`, `result_var`
    And,

    /// Если `var` или `other_var` равен `true`, то результат `true`, иначе `false`, записать результат в `result_var`
    ///
    /// Название: OR \
    /// Параметры: `var`, `other_var`, `result_var`
    Or,

    /// Если `var` равен `true` то вызвать функцию `func`
    ///
    /// Название: IF \
    /// Параметры: `bool_var`, `func`
    If,

    /// Узнать, имеет ли строка `var` в себе подстроку `substring` и записать результат в `result_var`
    ///
    /// Название: HAS_STR \
    /// Параметры: `string_var`, `substring`, `result_var`
    HasStr,

    /// Узнать, имеет ли список `list_var` значение `item_var` и записать результат в `result_var`
    ///
    /// Название: HAS_ITEM \
    /// Параметры: `list_var`, `item_var`, `result_var`
    HasItem,

    /// Узнать, имеет ли мап `map_var` поле с ключом `key_var` и значением `value_var` и записать результат в `result_var`
    ///
    /// Название: HAS_ENTRY \
    /// Параметры: `map_var`, `key_var`, `value_var`, `result_var`
    HasEntry,

    /// Узнать, имеет ли мап `map_var` поле с ключом `key_var` и записать результат в `result_var`
    ///
    /// Название: HAS_KEY \
    /// Параметры: `map_var`, `key_var`, `result_var`
    HasKey,

    /// Узнать, имеет ли мап `map_var` поле с значением `value_var` и записать результат в `result_var`
    ///
    /// Название: HAS_VALUE \
    /// Параметры: `map_var`, `value_var`, `result_var`
    HasValue,

    /// Узнать, имеет ли данные опшнл `optional_var` и записать результат в `result_var`
    ///
    /// Название: HAS_OPTIONAL \
    /// Параметры: `optional_var`, `result_var`
    HasOptional,
}

impl CommandType {
    pub fn from_name(name: &str, line: usize) -> Result<CommandType, ScriptError> {
        match name {
            "INIT_VAR" => Ok(CommandType::InitVar),
            "SET_VAR" => Ok(CommandType::SetVar),
            "TEMP_VAR" => Ok(CommandType::TempVar),
            "MOVE_VAR" => Ok(CommandType::MoveVar),
            "COPY_VAR" => Ok(CommandType::CopyVar),
            "DROP_VAR" => Ok(CommandType::DropVar),
            "HAS_VAR" => Ok(CommandType::HasVar),
            "TO_STRING" => Ok(CommandType::ToString),
            "TO_BYTES" => Ok(CommandType::ToBytes),
            "TO_INTEGER" => Ok(CommandType::ToInteger),
            "TO_FLOAT" => Ok(CommandType::ToFloat),
            "TO_CHAR" => Ok(CommandType::ToChar),
            "TO_BOOL" => Ok(CommandType::ToBool),
            "GET_SYMBOL" => Ok(CommandType::GetSymbol),
            "ADD_INT" => Ok(CommandType::AddInt),
            "ADD_FLOAT" => Ok(CommandType::AddFloat),
            "ADD_STR" => Ok(CommandType::AddStr),
            "SUB_STR" => Ok(CommandType::SubStr),
            "SUB_LIST" => Ok(CommandType::SubList),
            "LIST_SIZE" => Ok(CommandType::ListSize),
            "WRITE" => Ok(CommandType::Write),
            "READ" => Ok(CommandType::Read),
            "READ_ALL" => Ok(CommandType::ReadAll),
            "READ_STR" => Ok(CommandType::ReadStr),
            "READ_STR_ALL" => Ok(CommandType::ReadStrAll),
            "FOR" => Ok(CommandType::For),
            "FOR_MAP" => Ok(CommandType::ForMap),
            "FOR_LIST" => Ok(CommandType::ForList),
            "WHILE" => Ok(CommandType::While),
            "OPEN_FILE_IN" => Ok(CommandType::OpenFileIn),
            "OPEN_FILE_OUT" => Ok(CommandType::OpenFileOut),
            "OPEN_TCP_CONNECTION" => Ok(CommandType::OpenTcpConnection),
            "OPEN_TCP_LISTENER" => Ok(CommandType::OpenTcpListener),
            "SLEEP" => Ok(CommandType::Sleep),
            "NEW_THREAD" => Ok(CommandType::NewThread),
            "USE_FUNC" => Ok(CommandType::UseFunc),
            "FUNC" => Ok(CommandType::Func),
            "FUNC_END" => Ok(CommandType::FuncEnd),
            "RETURN" => Ok(CommandType::Return),
            "EQUALS" => Ok(CommandType::Equals),
            "MORE" => Ok(CommandType::More),
            "LESS" => Ok(CommandType::Less),
            "AND" => Ok(CommandType::And),
            "OR" => Ok(CommandType::Or),
            "IF" => Ok(CommandType::If),
            "HAS_STR" => Ok(CommandType::HasStr),
            "HAS_ITEM" => Ok(CommandType::HasItem),
            "HAS_ENTRY" => Ok(CommandType::HasEntry),
            "HAS_KEY" => Ok(CommandType::HasKey),
            "HAS_VALUE" => Ok(CommandType::HasValue),
            "HAS_OPTIONAL" => Ok(CommandType::HasOptional),
            _ => Err(ScriptError::CommandUnknownError(line)),
        }
    }
}

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
        .map(|s| s.trim_end_matches(" ").to_string())
        .collect()
}

fn parse_commands(lines: Vec<String>) -> Result<Vec<Command>, ScriptError> {
    let mut commands = Vec::new();
    let mut line_num = 0;

    for line in lines {
        line_num += 1;

        if line.trim().is_empty() {
            continue;
        }

        let params: Vec<String> = line.split(" ").map(|v| v.to_string()).collect();

        let command_type = CommandType::from_name(&params[0], line_num)?;

        let args = if params.is_empty() {
            Vec::new()
        } else {
            params[1..].to_vec()
        };

        commands.push(Command::new(command_type, line_num, args))
    }

    Ok(commands)
}

fn cut_funcs(commands: &mut Vec<Command>) -> Result<Vec<Function>, ScriptError> {
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
                    let result_type = VarType::from_name(&command.args[0])?;
                    let mut parameters = HashMap::new();

                    let mut param_key: Option<String> = None;
                    for i in &command.args[2..] {
                        match &param_key {
                            Some(key) => {
                                parameters.insert(key.to_string(), VarType::from_name(i)?);
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
    commands: Vec<Command>,
    functions: Vec<Function>,
}

impl Script {
    pub fn parse(text: String) -> Result<Script, ScriptError> {
        let lines = prepare_script(text);
        let mut commands = parse_commands(lines)?;
        let functions = cut_funcs(&mut commands)?;
        Ok(Script {
            commands,
            functions,
        })
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
            &mut HashMap::new(),
        )?;
        self.set_var(
            String::from("cout"),
            Variable::from_out_stream(Some(Arc::new(Mutex::new(cout)))),
            true,
            &mut HashMap::new(),
        )?;
        self.set_var(
            String::from("cin"),
            Variable::from_in_stream(Some(Arc::new(Mutex::new(cin)))),
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
        locals: &mut HashMap<String, Variable>,
    ) -> Result<(), ScriptError> {
        let mut var: Option<&mut Variable> = None;
        let parts: Vec<&str> = name.split('.').collect();

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
    ) -> Result<(), ScriptError> {
        let mut temp_vars: Vec<String> = Vec::new();

        for command in commands {
            match command.command_type {
                CommandType::InitVar => {
                    let type_var = command.args[0].clone();
                    let type_var = VarType::from_name(&type_var)?;
                    let name_var = command.args[1].clone();

                    self.set_var(name_var, Variable::empty_var(type_var)?, global, locals)?;
                }
                CommandType::SetVar => {
                    let name_var = command.args[0].clone();
                    let value_var = command.args[1..].join(" ");

                    let type_var = self
                        .get_var(name_var.clone(), &mut locals.clone())?
                        .get_type();
                    let var = Variable::parse_var(type_var, value_var)?;

                    self.set_var(name_var, var, global, locals)?;
                }
                CommandType::TempVar => {
                    let type_var = command.args[0].clone();
                    let name_var = command.args[1].clone();
                    let value_var = command.args[2..].join(" ");

                    self.set_var(
                        name_var.clone(),
                        Variable::parse_var(VarType::from_name(&type_var)?, value_var)?,
                        global,
                        locals,
                    )?;

                    temp_vars.push(name_var);

                    continue;
                }
                CommandType::MoveVar => {
                    let source_var = command.args[0].clone();
                    let target_var = command.args[1].clone();

                    let var = self.get_var(source_var.clone(), locals)?;

                    self.set_var(target_var, var, global, locals)?;
                    self.drop_var(source_var, locals)?;
                }
                CommandType::CopyVar => {
                    let source_var = command.args[0].clone();
                    let target_var = command.args[1].clone();

                    let var = self.get_var(source_var.clone(), locals)?;

                    self.set_var(target_var, var, global, locals)?;
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
                        locals,
                    )?;
                }
                CommandType::AddStr => {
                    let var_name = command.args[0].clone();
                    let other_var = command.args[1].clone();

                    let other_var = self.get_var(other_var.clone(), locals)?;
                    let other_var: String = if let Variable::List(VarType::Char, Some(list)) =
                        other_var
                    {
                        let mut bytes = Vec::new();
                        for ele in list {
                            bytes.push(ele.as_char()?);
                        }
                        String::from_utf8(bytes).or(Err(ScriptError::TypeMismatchError))?
                    } else if let Variable::String(_, Some(string)) = other_var {
                        string
                    } else if let Variable::Char(_, Some(value)) = other_var {
                        String::from_utf8(vec![value]).or(Err(ScriptError::TypeMismatchError))?
                    } else {
                        return Err(ScriptError::TypeMismatchError);
                    };

                    let var = self.get_var(var_name.clone(), locals)?.as_str()?;

                    self.set_var(
                        var_name,
                        Variable::from_str(Some(var + &other_var)),
                        global,
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
                CommandType::ToString => {}
                CommandType::ToBytes => {}
                CommandType::ToInteger => {}
                CommandType::ToFloat => {}
                CommandType::ToBool => {}
                CommandType::ToChar => {}
                CommandType::GetSymbol => {}
                CommandType::AddInt => {}
                CommandType::AddFloat => {}
                CommandType::SubStr => {}
                CommandType::SubList => {}
                CommandType::ListSize => {}
                CommandType::Read => {}
                CommandType::ReadAll => {}
                CommandType::ReadStr => {}
                CommandType::ReadStrAll => {}
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

        Ok(())
    }

    pub fn exec_function(
        &mut self,
        function: Function,
        result_var: String,
        args: Vec<Variable>,
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

        self.exec_commands(function.commands, false, &mut locals)?;

        if result_var != "null" {
            self.variables
                .insert(result_var, locals.get("result").unwrap().clone());
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), ScriptError> {
        self.exec_commands(self.commands.clone(), true, &mut HashMap::new())
    }
}
