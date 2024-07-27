# Sust

язык саст LOL

код хеллоу ворлда:

```
INIT_VAR string text
SET_VAR text Hello World!
WRITE text cout
DROP_VAR text
```


минимальный код для хеллоу ворлда:

```
TEMP_VAR string text Hello World!
WRITE text cout
```

## Команды

| Команда                  | Параметры  | Описание    |
|--------------------------|------------|-------------|
| `INIT_VAR`               | `type_var`, `name_var` | Инициализировать переменную `name_var` с типом `type_var` |
| `SET_VAR`                | `name_var`, `value_var` | Установить значение переменной в `name_var` |
| `TEMP_VAR`               | `type_var`, `name_var`, `value_var` | Переменная `name_var` инициализируется с типом `type_var` и присваивается `value_var`, переменная дропается после первого же использования |
| `MOVE_VAR`               | `source_var`, `target_var` | Переместить значение переменной с `source_var` в `target_var` |
| `COPY_VAR`               | `source_var`, `target_var` | Скопировать значение переменной с `source_var` в `target_var` |
| `DROP_VAR`               | `name_var` | Дропнуть переменную `name_var` |
| `HAS_VAR`                | `name_var`, `result_var` | В переменную `result_var` записывается `bool` существует ли переменная `name_var` |
| `TO_STRING`              | `source_var`, `target_var` | Скопировать значение переменной с `source_var` в `target_var`, переводя в строку |
| `ADD_INT`                | `int_var1`, `int_var2` | Прибавить к числу `int_var1` значение `int_var2` |
| `ADD_FLOAT`              | `float_var1`, `float_var2` | Прибавить к числу `float_var1` значение `float_var2` |
| `ADD_STR`                | `str_var`, `value_var` | Прибавить к строке `str_var` значение `value_var` (может быть типа `string/char/list[char]`) |
| `SUB_STR`                | `str_var`, `start_index`, `end_index` | Сделать подстроку из строки `str_var` и сохранить туда же |
| `SUB_LIST`               | `list_var`, `start_index`, `end_index` | Сделать подсписок из списка `list_var` и сохранить туда же |
| `LIST_SIZE`              | `list_var`, `result_var` | Получить размер списка и записать в переменную `result_var` типа `int` |
| `WRITE`                  | `name_var`, `stream_var` | Вывести переменную `name_var` в `stream_var` |
| `READ`                   | `name_var`, `size_var`, `stream_var` | Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `list[char]` |
| `READ_ALL`               | `name_var`, `stream_var` | Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `list[char]` |
| `READ_STR`               | `name_var`, `size_var`, `stream_var` | Прочитать с `stream_var` ровно `size_var` байтов в переменную `name_var` типа `string` |
| `READ_STR_ALL`           | `name_var`, `stream_var` | Прочитать с `stream_var` все имеющиеся байты в переменную `name_var` типа `string` |
| `FOR`                    | `func(int)`, `start_index`, `end_index` | Функция `func` (с единственным аргументом с типом `int`) вызывается с `start_index` до `end_index` включительно, `start_index` и `end_index` это названия переменных |
| `FOR_MAP`                | `func(any, any)`, `map_var` | Функция `func` вызывается для каждого `key`, `value` переменной `map_var` |
| `FOR_LIST`               | `func(any)`, `list_var` | Функция `func` вызывается для каждого предмета переменной `list_var` |
| `WHILE`                  | `func -> bool` | Функция `func` (с результатом `bool`) вызывается, пока функция выдает `true` |
| `USE_FUNC`               | `func`, `result_var`, `[arg_var1] ... [arg_varN]` | Функция `func` вызывается с переданными аргументами и устанавливает результат в переменную `result_var` |
| `OPEN_FILE_IN`           | `path_var`, `stream_var` | Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для чтения и записать стрим для чтения в переменную `stream_var` |
| `OPEN_FILE_OUT`          | `path_var`, `stream_var` | Открыть файл по пути `path_var` (`path_var`, `stream_var` - переменные) для записи и записать стрим для записи в переменную `stream_var` |
| `OPEN_TCP_CONNECTION`    | `addr_var`, `port_var`, `in_stream`, `out_stream` | Подключиться по `addr_var:port_var` (`addr_var: string`, `port_var: int`, `in_stream: in_stream`, `out_stream: out_stream` - переменные) и записать стримы для чтения и записи в `in_stream` и `out_stream` |
| `OPEN_TCP_LISTENER`      | `addr_var`, `port_var`, `accept_func(string,int,in_stream,out_stream)` | Ожидание подключений с `addr_var:port_var` (`addr_var: string`, `port_var: int` - переменные), при подключениях вызывается функция `accept_func` |
| `HAS_OPTIONAL`           | `optional_var`, `result_var` | Узнать, имеет ли данные опшнл `optional_var` и записать результат в `result_var: bool` |
| `WHEN_OPTIONAL`          | `optional_var`, `func(option[any])` | Когда опшнл `optional_var` имеет данные, вызывается `func` |
| `SLEEP`          | `time_var` | Ждать миллисекунд из переменной `time_var` (тип переменной: int) |

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
