#[derive(Debug)]
pub enum ScriptError {
    ParseVarError,
    TypeUnknownError,
    CommandUnknownError,
    CommandArgsInvalidError,
    UnknownVarError,
    TypeMismatchError,
    VarNotInitedError,
    StringUTF8Error,
    VarInitedError,
    FunctionUnknownError,
}

impl Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("some error ez")
    }
}
impl Error for ScriptError {}
