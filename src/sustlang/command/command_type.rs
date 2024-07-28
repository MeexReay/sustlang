use super::super::script::ScriptError;

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
    /// Название: TO_CHARS \
    /// Параметры: `source_var`, `result_var`
    ToChars,

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

    /// Скопировать предмет из списка `str_var` по индексу `index_var` и записать в `result_var`
    ///
    /// Название: GET_ITEM \
    /// Параметры: `list_var`, `index_var`, `result_var`
    GetItem,

    /// Скопировать предмет из мапы `map_var` по ключу `key_var` и записать в `result_var`
    ///
    /// Название: GET_VALUE \
    /// Параметры: `map_var`, key_var`, `result_var`
    GetValue,

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

    /// Если `var` равен `true`, то результат `false`, иначе `true`, записать результат в `result_var`
    ///
    /// Название: NOT \
    /// Параметры: `var`, `result_var`
    Not,

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

    /// Достать данные из `optional_var` и установить в `result_var`
    ///
    /// Название: UNPACK_OPTIONAL \
    /// Параметры: `optional_var`, `result_var`
    UnpackOptional,

    /// Упаковать `var` в `optional` и установить в `result_var`
    ///
    /// Название: PACK_OPTIONAL \
    /// Параметры: `var`, `result_var`
    PackOptional,

    /// Установить пустой `optional` в `var`
    ///
    /// Название: NONE_OPTIONAL \
    /// Параметры: `var`
    NoneOptional,

    /// Получить размер списка и записать в переменную `result_var` типа `int`
    ///
    /// Название: LIST_SIZE \
    /// Параметры: `list_var`, `result_var`
    ListSize,

    /// Получить размер строки и записать в переменную `result_var` типа `int`
    ///
    /// Название: MAP_SIZE \
    /// Параметры: `map_var`, `result_var`
    MapSize,

    /// Получить размер мапы и записать в переменную `result_var` типа `int`
    ///
    /// Название: STRING_SIZE \
    /// Параметры: `string_var`, `result_var`
    StringSize,

    /// Функция `func` вызывается для каждого символа строки `string_var`
    ///
    /// Название: FOR_STRING \
    /// Параметры: `func(char)`, `string_var`
    ForString,

    /// Импортировать код из скрипта по пути (путь должен быть с расширением файла) (путь это переменная)
    ///
    /// Название: IMPORT \
    /// Параметры: `script_path`
    Import,

    /// Импортировать код из текста переменной в скрипт
    ///
    /// Название: IMPORT_TEXT \
    /// Параметры: `script_text_var`
    ImportText,

    /// Получить рандомное число от `min_var: int` до `max_var: int` включительно и записать в `result_var: int`
    ///
    /// Название: RANDOM \
    /// Параметры: `min_var`, `max_var`, `result_var`
    Random,
}

impl CommandType {
    pub fn from_name(name: &str) -> Result<CommandType, ScriptError> {
        match name {
            "INIT_VAR" => Ok(CommandType::InitVar),
            "SET_VAR" => Ok(CommandType::SetVar),
            "TEMP_VAR" => Ok(CommandType::TempVar),
            "MOVE_VAR" => Ok(CommandType::MoveVar),
            "COPY_VAR" => Ok(CommandType::CopyVar),
            "DROP_VAR" => Ok(CommandType::DropVar),
            "HAS_VAR" => Ok(CommandType::HasVar),
            "TO_STRING" => Ok(CommandType::ToString),
            "TO_CHARS" => Ok(CommandType::ToChars),
            "TO_INTEGER" => Ok(CommandType::ToInteger),
            "TO_FLOAT" => Ok(CommandType::ToFloat),
            "TO_CHAR" => Ok(CommandType::ToChar),
            "TO_BOOL" => Ok(CommandType::ToBool),
            "GET_SYMBOL" => Ok(CommandType::GetSymbol),
            "GET_ITEM" => Ok(CommandType::GetItem),
            "GET_VALUE" => Ok(CommandType::GetValue),
            "ADD_INT" => Ok(CommandType::AddInt),
            "ADD_FLOAT" => Ok(CommandType::AddFloat),
            "ADD_STR" => Ok(CommandType::AddStr),
            "SUB_STR" => Ok(CommandType::SubStr),
            "SUB_LIST" => Ok(CommandType::SubList),
            "LIST_SIZE" => Ok(CommandType::ListSize),
            "MAP_SIZE" => Ok(CommandType::MapSize),
            "STRING_SIZE" => Ok(CommandType::StringSize),
            "WRITE" => Ok(CommandType::Write),
            "READ" => Ok(CommandType::Read),
            "READ_ALL" => Ok(CommandType::ReadAll),
            "FOR" => Ok(CommandType::For),
            "FOR_MAP" => Ok(CommandType::ForMap),
            "FOR_LIST" => Ok(CommandType::ForList),
            "FOR_STRING" => Ok(CommandType::ForString),
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
            "NOT" => Ok(CommandType::Not),
            "IF" => Ok(CommandType::If),
            "HAS_STR" => Ok(CommandType::HasStr),
            "HAS_ITEM" => Ok(CommandType::HasItem),
            "HAS_ENTRY" => Ok(CommandType::HasEntry),
            "HAS_KEY" => Ok(CommandType::HasKey),
            "HAS_VALUE" => Ok(CommandType::HasValue),
            "HAS_OPTIONAL" => Ok(CommandType::HasOptional),
            "UNPACK_OPTIONAL" => Ok(CommandType::UnpackOptional),
            "PACK_OPTIONAL" => Ok(CommandType::PackOptional),
            "NONE_OPTIONAL" => Ok(CommandType::NoneOptional),
            "IMPORT_TEXT" => Ok(CommandType::ImportText),
            "IMPORT" => Ok(CommandType::Import),
            "RANDOM" => Ok(CommandType::Random),
            _ => Err(ScriptError::CommandUnknownError),
        }
    }
}
