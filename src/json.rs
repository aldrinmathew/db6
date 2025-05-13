use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Index, IndexMut},
    str::FromStr,
};

pub enum Json {
    Number(JsonNumber),
    String(String),
    List(Vec<Json>),
    Object(JsonObject),
    Bool(bool),
    Null,
    None,
}

impl FromStr for Json {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Json::parse(s.as_bytes())
    }
}

pub enum JsonNumber {
    Int(i64),
    Float(f64),
}

impl Display for JsonNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonNumber::Int(num) => num.fmt(f),
            JsonNumber::Float(num) => num.fmt(f),
        }
    }
}

pub struct JsonObject {
    map: HashMap<String, Json>,
    none: Box<Json>,
}

impl JsonObject {
    pub fn new() -> JsonObject {
        JsonObject {
            map: HashMap::<String, Json>::new(),
            none: Box::<Json>::new(Json::None),
        }
    }

    pub fn from_map(val: HashMap<String, Json>) -> JsonObject {
        JsonObject {
            map: val,
            none: Box::<Json>::new(Json::None),
        }
    }
}

impl Display for JsonObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{ ")?;
        let mut one_valid_value = false;
        for (key, value) in &self.map {
            if !matches!(value, Json::None) {
                if one_valid_value {
                    f.write_str(", ")?;
                }
                one_valid_value = true;
                f.write_str("\"")?;
                key.fmt(f)?;
                f.write_str("\" : ")?;
                value.fmt(f)?;
            }
        }
        f.write_str(" }")?;
        Ok(())
    }
}

impl Index<String> for JsonObject {
    type Output = Json;

    fn index(&self, index: String) -> &Self::Output {
        if self.map.contains_key(&index) {
            return &self.map[&index];
        } else {
            return self.none.as_ref();
        }
    }
}

impl IndexMut<String> for JsonObject {
    fn index_mut(&mut self, index: String) -> &mut Self::Output {
        if !self.map.contains_key(&index) {
            self.map.insert(index.clone(), Json::None);
        }
        return self.map.get_mut(&index).unwrap();
    }
}

enum Token {
    CurlyOpen,
    CurlyClose,
    BracketOpen,
    BracketClose,
    Comma,
    Colon,
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Null,
}

impl Json {
    fn tokenise(data: &[u8]) -> Result<Vec<Token>, String> {
        let mut res = Vec::<Token>::new();
        let mut cur = 0usize;
        while cur < data.len() {
            match data[cur] {
                b'0'..=b'9' => {
                    let mut found_decimal = false;
                    let mut num_str = String::new();
                    while data[cur].is_ascii_digit() || (!found_decimal && data[cur] == b'.') {
                        if data[cur] == b'.' {
                            found_decimal = true;
                        }
                        num_str += &(data[cur] as char).to_string();
                        cur += 1;
                    }
                    if found_decimal {
                        match num_str.parse::<f64>() {
                            Ok(fl_num) => {
                                res.push(Token::Float(fl_num));
                            }
                            Err(err) => {
                                return Err("Failed to parse the floating point number "
                                    .to_string()
                                    + &num_str
                                    + ". The error is "
                                    + &err.to_string());
                            }
                        };
                    } else {
                        match num_str.parse::<i64>() {
                            Ok(int_num) => {
                                res.push(Token::Int(int_num));
                            }
                            Err(err) => {
                                return Err("Failed to parse the integer ".to_string()
                                    + &num_str
                                    + ". The error is "
                                    + &err.to_string());
                            }
                        }
                    }
                }
                b'{' => {
                    res.push(Token::CurlyOpen);
                    cur += 1;
                }
                b'}' => {
                    res.push(Token::CurlyClose);
                    cur += 1;
                }
                b'[' => {
                    res.push(Token::BracketOpen);
                    cur += 1;
                }
                b']' => {
                    res.push(Token::BracketClose);
                    cur += 1;
                }
                b':' => {
                    res.push(Token::Colon);
                    cur += 1;
                }
                b',' => {
                    res.push(Token::Comma);
                    cur += 1;
                }
                b' ' | b'\t' | b'\n' => {
                    cur += 1;
                }
                b'a'..=b'z' => {
                    let mut ident = String::new();
                    while data[cur].is_ascii_lowercase() {
                        ident += &(data[cur] as char).to_string();
                        cur += 1;
                    }
                    if ident == "true" {
                        res.push(Token::Bool(true));
                    } else if ident == "false" {
                        res.push(Token::Bool(false));
                    } else if ident == "null" {
                        res.push(Token::Null);
                    }
                }
                b'"' => {
                    let mut content = String::new();
                    let mut escape = false;
                    cur += 1;
                    while cur < data.len() && (escape || data[cur] != b'"') {
                        if !escape && data[cur] == b'\\' {
                            escape = true;
                        } else if escape {
                            if data[cur] == b'"' {
                                content += &(b'"' as char).to_string();
                            } else if data[cur] == b'\\' {
                                content += &(b'\\' as char).to_string();
                            } else if data[cur] == b'/' {
                                content += &(b'/' as char).to_string();
                            } else if data[cur] == b'b' {
                                content += &(b'\x08' as char).to_string();
                            } else if data[cur] == b'f' {
                                content += &(b'\x0c' as char).to_string();
                            } else if data[cur] == b'n' {
                                content += &(b'\n' as char).to_string();
                            } else if data[cur] == b'r' {
                                content += &(b'\r' as char).to_string();
                            } else if data[cur] == b't' {
                                content += &(b'\t' as char).to_string();
                            } else if data[cur] == b'u' {
                                cur += 1;
                                let mut uni_str = String::new();
                                for i in 0..4 {
                                    if (cur + i) < data.len() {
                                        if data[cur + i].is_ascii_hexdigit() {
                                            uni_str += &(data[cur + i] as char).to_string();
                                        } else {
                                            return Err(
                                                "Expected 4 hex digits after \\u for the unicode character, but found the character ".to_string()
                                                    + &(data[cur + i] as char).to_string()
                                                    + " instead",
                                            );
                                        }
                                    } else {
                                        return Err("Expected 4 characters to be present after \\u for the unicode character, but the JSON representation ended".to_string());
                                    }
                                }
                                match u32::from_str_radix(uni_str.as_str(), 16) {
                                    Ok(code) => match char::from_u32(code) {
                                        Some(char_val) => {
                                            content += &char_val.to_string();
                                        }
                                        None => {
                                            return Err("Failed to convert the provided unicode codepoint \\u".to_string() + &uni_str + " to a unicode scalar value");
                                        }
                                    },
                                    Err(err) => {
                                        return Err(
                                            "Failed to parse the unicode code point here: \\u"
                                                .to_string()
                                                + &uni_str
                                                + ". The error is "
                                                + &err.to_string(),
                                        );
                                    }
                                }
                                cur += 3;
                            } else {
                                return Err("Invalid escape sequence \\".to_string()
                                    + &(data[cur] as char).to_string()
                                    + " found in JSON");
                            }
                            escape = false;
                        } else {
                            content += &(data[cur] as char).to_string();
                        }
                        cur += 1;
                    }
                    if cur == data.len() {
                        return Err("Could not find \" to end the string value".to_string());
                    } else {
                        cur += 1;
                    }
                    res.push(Token::String(content));
                }
                _ => {
                    return Err("Invalid character found in the JSON: ".to_string()
                        + &(data[cur] as char).to_string());
                }
            }
        }
        return Ok(res);
    }

    fn parse_value<'a>(data: &'a Vec<Token>, ind: usize) -> Result<(Json, usize), String> {
        if ind >= data.len() {
            return Err(
                "Expected to find a JSON value, but the JSON representation ended before that"
                    .to_string(),
            );
        }
        match &data[ind] {
            Token::Bool(val) => Ok((Json::Bool(*val), ind)),
            Token::Null => Ok((Json::Null, ind)),
            Token::String(val) => Ok((Json::String(val.clone()), ind)),
            Token::Int(val) => Ok((Json::Number(JsonNumber::Int(*val)), ind)),
            Token::Float(val) => Ok((Json::Number(JsonNumber::Float(*val)), ind)),
            Token::CurlyOpen => {
                let mut vals_map = HashMap::<String, Json>::new();
                if ind + 1 >= data.len() {
                    return Err("Found { first in the JSON, and expected key-value pairs after it, but the JSON representation ended".to_string());
                }
                let mut cur = ind + 1usize;
                if !matches!(data[cur], Token::String(_)) {
                    return Err(
                        "Expected a string value for the key of the field, after {".to_string()
                    );
                }
                'object_loop: while let Token::String(key) = &data[cur] {
                    if cur + 1 >= data.len() || !matches!(data[cur + 1], Token::Colon) {
                        return Err("Expected : after the key string `".to_string()
                            + &key
                            + &"`, before the value of the field".to_string());
                    }
                    if cur + 2 >= data.len() {
                        return Err(
                            "Expected a value after : for the value of the field with key `"
                                .to_string()
                                + &key
                                + &"`".to_string(),
                        );
                    }
                    match Self::parse_value(data, cur + 2) {
                        Ok(value) => {
                            vals_map.insert(key.clone(), value.0);
                            cur = value.1;
                        }
                        Err(err) => {
                            return Err("Error while parsing a value of the field with key `"
                                .to_string()
                                + &key
                                + &"`. The error is ".to_string()
                                + &err);
                        }
                    }
                    if cur + 1 >= data.len() {
                        return Err("Expected either a , or a } after the key-value pair, but the JSON ended".to_string());
                    }
                    if matches!(data[cur + 1], Token::Comma) {
                        cur += 1;
                        if cur + 1 >= data.len() {
                            return Err("Expected a string after the , for the key of the next field. Trailing commas are not allowed in JSON".to_string());
                        }
                        cur += 1;
                    } else if matches!(data[cur + 1], Token::CurlyClose) {
                        cur += 1;
                        break 'object_loop;
                    } else {
                        return Err(
                            "Expected either a , or a } after the key-value pair, but found an invalid symbol".to_string()
                        );
                    }
                }
                Ok((Json::Object(JsonObject::from_map(vals_map)), cur))
            }
            Token::BracketOpen => {
                let mut list = Vec::<Json>::new();
                if ind + 1 >= data.len() {
                    return Err("Expected either values to be present after [ for the array, or for the array to end with a ], but the JSON representation ended before that".to_string());
                }
                let mut cur = ind + 1usize;
                'array_loop: while !matches!(data[cur], Token::BracketClose) {
                    match Self::parse_value(data, cur) {
                        Ok(val) => {
                            list.push(val.0);
                            cur = val.1;
                            if cur + 1 >= data.len() {
                                return Err("Expected either , after the value or ] to end the array, but the JSON representation ended before that".to_string());
                            }
                            if matches!(data[cur + 1], Token::Comma) {
                                if cur + 2 >= data.len() {
                                    return Err("Expected a value to be present after , in the array, but the JSON representation ended before that".to_string());
                                }
                                if matches!(data[cur + 2], Token::BracketClose) {
                                    return Err("Trailing commas are not supported in arrays. Found ] immediately after a ,".to_string());
                                }
                                cur += 1;
                            } else if matches!(data[cur + 1], Token::BracketClose) {
                                cur += 1;
                                break 'array_loop;
                            } else {
                                return Err("Expected either , or ] after the array value, but found an invalid symbol".to_string());
                            }
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
                Ok((Json::List(list), cur))
            }
            _ => {
                return Err("Invalid symbol found in JSON".to_string());
            }
        }
    }

    pub fn parse(data: &[u8]) -> Result<Json, String> {
        match Self::tokenise(data) {
            Ok(tokens) => {
                if tokens.len() == 0 {
                    return Err(
                        "Could not parse a valid JSON value as the string representation is empty"
                            .to_string(),
                    );
                } else {
                    match Self::parse_value(&tokens, 0) {
                        Ok(val) => {
                            if val.1 == tokens.len() - 1 {
                                return Ok(val.0);
                            } else {
                                return Err(format!(
                                    "Found the value {} first in the JSON, but the JSON representation does not end after that",
                                    val.0
                                ));
                            }
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
            }
            Err(err) => {
                return Err(err.to_string());
            }
        }
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Json::Number(num) => num.fmt(f),
            Json::String(string) => {
                "\"".fmt(f)?;
                string.fmt(f)?;
                "\"".fmt(f)?;
                Ok(())
            }
            Json::List(list) => {
                f.write_str("[")?;
                for i in 0..list.len() {
                    list[i].fmt(f)?;
                    if i != (list.len()) {
                        f.write_str(", ")?;
                    }
                }
                f.write_str("]")?;
                Ok(())
            }
            Json::Bool(val) => val.fmt(f),
            Json::Object(obj) => obj.fmt(f),
            Json::Null => f.write_str("null"),
            Json::None => Ok(()),
        }
    }
}
