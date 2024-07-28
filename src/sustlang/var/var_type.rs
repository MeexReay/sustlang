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
            "b" => Ok(VarType::Bool),
            "string" => Ok(VarType::String),
            "str" => Ok(VarType::String),
            "s" => Ok(VarType::String),
            "integer" => Ok(VarType::Integer),
            "int" => Ok(VarType::Integer),
            "i" => Ok(VarType::Integer),
            "float" => Ok(VarType::Float),
            "f" => Ok(VarType::Float),
            "char" => Ok(VarType::Char),
            "c" => Ok(VarType::Char),
            "in_stream" => Ok(VarType::InStream),
            "in" => Ok(VarType::InStream),
            "out_stream" => Ok(VarType::OutStream),
            "out" => Ok(VarType::OutStream),
            "null" => Ok(VarType::Null),
            _ => Err(ScriptError::TypeUnknownError),
        }
    }
}
