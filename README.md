# Sust

язык саст LOL

код хеллоу ворлда:

```
INIT_VAR string text       # init text var
SET_VAR text Hello World!  # set text to var
TEMP_VAR char br 10        # create temp var with line break
ADD_STR text br            # add line break to text
WRITE text cout            # write text to console
DROP_VAR text              # drop text var
```


минимальный код для хеллоу ворлда:

```
TEMP_VAR string text Hello World!
WRITE text cout
```

## Команды

### Переменные

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `INIT_VAR`               | `type_var`, `name_var` | Инициализировать переменную `name_var` с типом `type_var` |
| `SET_VAR`                | `name_var`, `value_var` | Установить значение переменной в `name_var` |
| `TEMP_VAR`               | `type_var`, `name_var`, `value_var` | Переменная `name_var` инициализируется с типом `type_var` и присваивается `value_var`, переменная дропается через одну команду |
| `MOVE_VAR`               | `source_var`, `target_var` | Переместить значение переменной с `source_var` в `target_var` |
| `COPY_VAR`               | `source_var`, `target_var` | Скопировать значение переменной с `source_var` в `target_var` |
| `DROP_VAR`               | `name_var` | Дропнуть переменную `name_var` |
| `HAS_VAR`                | `name_var`, `result_var` | В переменную `result_var` записывается `bool` существует ли переменная `name_var` |

### Преобразование переменных

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `TO_STRING`              | `source_var`, `result_var` | Скопировать значение переменной с `source_var` в `result_var`, переводя в `string` |
| `TO_CHARS`               | `str_var`, `result_var` | Скопировать строку `str_var` в `result_var`, переводя в `list[char]` |
| `TO_INTEGER`             | `source_var`, `result_var` | Скопировать строку `source_var` (тип переменной: `string`/`char`) в `result_var`, переводя в `integer` |
| `TO_CHAR`             | `source_var`, `result_var` | Скопировать строку `source_var` (тип переменной: `string`/`integer`) в `result_var`, переводя в `char` |
| `TO_BOOL`             | `source_var`, `result_var` | Скопировать строку `source_var` (тип переменной: `string`/`integer`) в `result_var`, переводя в `bool` |
| `TO_FLOAT`               | `source_var`, `result_var` | Скопировать строку `source_var` в `result_var`, переводя в `float` |
| `GET_SYMBOL`             | `str_var`, `index_var`, `result_var` | Скопировать символ из строки `str_var` по индексу `index_var` и записать в `result_var` |
| `GET_ITEM`               | `list_var`, `index_var`, `result_var` | Скопировать предмет из списка `str_var` по индексу `index_var` и записать в `result_var` |
| `GET_VALUE`              | `map_var`, `key_var`, `result_var` | Скопировать предмет из мапы `map_var` по ключу `key_var` и записать в `result_var` |
| `ADD_INT`                | `int_var1`, `int_var2` | Прибавить к числу `int_var1` значение `int_var2` |
| `ADD_FLOAT`              | `float_var1`, `float_var2` | Прибавить к числу `float_var1` значение `float_var2` |
| `ADD_STR`                | `str_var`, `value_var` | Прибавить к строке `str_var` значение `value_var` (может быть типа `string/char/list[char]`) |
| `SUB_STR`                | `str_var`, `start_index`, `end_index` | Сделать подстроку из строки `str_var` и сохранить туда же |
| `SUB_LIST`               | `list_var`, `start_index`, `end_index` | Сделать подсписок из списка `list_var` и сохранить туда же |
| `UNPACK_OPTIONAL`        | `optional_var`, `result_var` | Достать данные из `optional_var` и установить в `result_var` |
| `LIST_SIZE`              | `list_var`, `result_var` | Получить размер списка и записать в переменную `result_var` типа `int` |
| `STRING_SIZE`            | `string_var`, `result_var` | Получить размер строки и записать в переменную `result_var` типа `int` |
| `MAP_SIZE`            | `map_var`, `result_var` | Получить размер мапы и записать в переменную `result_var` типа `int` |

### Циклы

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `FOR`                    | `func(int)`, `start_index`, `end_index` | Функция `func` (с единственным аргументом с типом `int`) вызывается с `start_index` до `end_index` включительно, `start_index` и `end_index` это названия переменных |
| `FOR_MAP`                | `func(any, any)`, `map_var` | Функция `func` вызывается для каждого `key`, `value` переменной `map_var` |
| `FOR_LIST`               | `func(any)`, `list_var` | Функция `func` вызывается для каждого предмета переменной `list_var` |
| `FOR_STRING`             | `func(char)`, `string_var` | Функция `func` вызывается для каждого символа строки `string_var` |
| `WHILE`                  | `func -> bool` | Функция `func` (с результатом `bool`) вызывается, пока функция выдает `true` |

### Работа со стримами

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `WRITE`                  | `name_var`, `stream_var` | Вывести переменную `name_var` в `stream_var` |
| `READ`                   | `name_var`, `size_var`, `stream_var` | Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `list[char]`/`string` |
| `READ_ALL`               | `name_var`, `stream_var` | Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `list[char]`/`string` |
| `OPEN_FILE_IN`           | `path_var`, `stream_var` | Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для чтения и записать стрим для чтения в переменную `stream_var` |
| `OPEN_FILE_OUT`          | `path_var`, `stream_var` | Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для записи и записать стрим для записи в переменную `stream_var` |
| `OPEN_TCP_CONNECTION`    | `addr_var`, `port_var`, `in_stream`, `out_stream` | Подключиться по `addr_var:port_var` (`addr_var: string`, `port_var: int`, `in_stream: in_stream`, `out_stream: out_stream` - переменные) и записать стримы для чтения и записи в `in_stream` и `out_stream` |
| `OPEN_TCP_LISTENER`      | `addr_var`, `port_var`, `accept_func(string,int,in_stream,out_stream)` | Ожидание подключений с `addr_var:port_var` (`addr_var: string`, `port_var: int` - переменные), при подключениях вызывается функция `accept_func` |

### Логические операции

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `EQUALS`                 | `var`, `other_var`, `result_var` | Узнать, равен ли `var` и `other_var` записать результат в `result_var` |
| `MORE`                   | `var`, `other_var`, `result_var` | Узнать, больше ли в `var` чем в `other_var` записать результат в `result_var` |
| `LESS`                   | `var`, `other_var`, `result_var` | Узнать, меньше ли в `var` чем в `other_var` записать результат в `result_var` |
| `AND`                    | `var`, `other_var`, `result_var` | Если `var` и `other_var` равны `true`, то результат `true`, иначе `false`, записать результат в `result_var` |
| `OR`                     | `var`, `other_var`, `result_var` | Если `var` или `other_var` равен `true`, то результат `true`, иначе `false`, записать результат в `result_var`  |
| `NOT`                    | `var`, `result_var` | Если `var` равен `true`, то результат `false`, иначе `true`, записать результат в `result_var`  |
| `IF`                     | `bool_var`, `func` | Если `var` равен `true` то вызвать функцию `func` |
| `HAS_STR`                | `string_var`, `substring`, `result_var` | Узнать, имеет ли строка `var` в себе подстроку `substring` и записать результат в `result_var` |
| `HAS_ITEM`               | `list_var`, `item_var`, `result_var` | Узнать, имеет ли список `list_var` значение `item_var` и записать результат в `result_var` |
| `HAS_ENTRY`              | `map_var`, `key_var`, `value_var`, `result_var` | Узнать, имеет ли мап `map_var` поле с ключом `key_var` и значением `value_var` и записать результат в `result_var` |
| `HAS_KEY`                | `map_var`, `key_var`, `result_var` | Узнать, имеет ли мап `map_var` поле с ключом `key_var` и записать результат в `result_var` |
| `HAS_VALUE`              | `map_var`, `value_var`, `result_var` | Узнать, имеет ли мап `map_var` поле с значением `value_var` и записать результат в `result_var` |
| `HAS_OPTIONAL`           | `optional_var`, `result_var` | Узнать, имеет ли данные опшнл `optional_var` и записать результат в `result_var` |


### Функции

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `USE_FUNC`               | `func_name`, `result_var`, `[arg_var1] ... [arg_varN]` | Функция `func` вызывается с переданными аргументами и устанавливает результат в переменную `result_var` |
| `FUNC`                   | `result_type`, `func_name`, `[arg_name_1 arg_type] ... [arg_name_N arg_type]` | Создать функцию с типом результата `result_type`, названием `func_name` и аргументами `[arg_name_1 arg_type] ... [arg_name_N arg_type]`. Установить результат переменной можно изменив переменную `result` внутри функции. Все команды после этой и до `FUNC_END` будут командами функции. Функции внутри функций не могут быть. |
| `RETURN`                 |            | Досрочно выйти из функции, также работает как выход из скрипта |
| `FUNC_END`               |            | Маркер, что команды функции тут заканчиваются |

### Прочее

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `SLEEP`                  | `time_var` | Ждать миллисекунд из переменной `time_var` (тип переменной: int) |
| `NEW_THREAD`             | `func`     | Вызвать функцию `func` в новом потоке |

## Типы переменных

| Type                   | Example Command                   | Example Value            |
|------------------------|------------------------------------|--------------------------|
| `bool`                 | `SET_VAR var true`                 | `true` / `false`         |
| `string`               | `SET_VAR var some_text`            | `some_text`              |
| `integer`              | `SET_VAR var 123`                  | `123`                    |
| `float`                | `SET_VAR var 14.48`                | `14.48`                  |
| `char`                 | `SET_VAR var 255`                  | `0 - 255`                |
| `list[type]`           | `SET_VAR var.0 value`              | `value`                  |
| `map[key_type,value_type]` | `SET_VAR var.key value`         | `value`                  |
| `optional[type]`       | `SET_VAR var (value)`              | `(value)` / `null`       |
| `in_stream`            | `OPEN_FILE_IN path var`            |                          |
| `out_stream`           | `OPEN_FILE_OUT path var`           |                          |

## Стандартные переменные

| Переменная | Описание                             | Тип         |
|------------|--------------------------------------|-------------|
| `args`     | Аргументы при вызове программы       | `list[string]` |
| `cout`     | Вывод консоли                        | `out_stream` |
| `cin`      | Ввод консоли                         | `in_stream`  |

## Создание функций

example code:

```
FUNC result_type func_name arg_name type arg_name2 type # Создание функции
SET_VAR result value  # Установить результат функции (value изменить, это просто пример)
RETURN                # Для досрочного выхода из функции (тут это не надо, функция и так закончится, это просто пример)
FUNC_END
```
