/// Filters operating on numbers
use std::collections::HashMap;

use serde_json::value::{Value, to_value};
use humansize::{FileSize, file_size_opts};

use errors::Result;


/// Returns a suffix if the value is greater or equal than 2. Suffix defaults to `s`
pub fn pluralize(value: Value, mut args: HashMap<String, Value>) -> Result<Value> {
    let num = try_get_value!("pluralize", "value", f64, value);
    let suffix = match args.remove("suffix") {
        Some(val) => try_get_value!("pluralize", "suffix", String, val),
        None => "s".to_string(),
    };

    if num >= 2.0 {
        Ok(to_value(&suffix).unwrap())
    } else {
        Ok(to_value(&"").unwrap())
    }
}

/// Returns a rounded number using the `method` arg and `precision` given.
/// `method` defaults to `common` which will round to the nearest number.
/// `ceil` and `floor` are also available as method.
/// `precision` defaults to `0`, meaning it will round to an integer
pub fn round(value: Value, mut args: HashMap<String, Value>) -> Result<Value> {
    let num = try_get_value!("round", "value", f64, value);
    let method = match args.remove("method") {
        Some(val) => try_get_value!("round", "method", String, val),
        None => "common".to_string(),
    };
    let precision = match args.remove("precision") {
        Some(val) => try_get_value!("round", "precision", i32, val),
        None => 0,
    };
    let multiplier = if precision == 0 { 1.0 } else { 10.0_f64.powi(precision) } ;

    match method.as_ref() {
        "common" => Ok(to_value((multiplier * num).round() / multiplier).unwrap()),
        "ceil" => Ok(to_value((multiplier * num).ceil() / multiplier).unwrap()),
        "floor" => Ok(to_value((multiplier * num).floor() / multiplier).unwrap()),
        _ => bail!(
                "Filter `round` received an incorrect value for arg `method`: got `{:?}`, \
                only common, ceil and floor are allowed",
                method
            )
    }
}


/// Returns a human-readable file size (i.e. '110 MB') from an integer
pub fn filesizeformat(value: Value, _: HashMap<String, Value>) -> Result<Value> {
    let num = try_get_value!("filesizeformat", "value", i64, value);
    match num
        .file_size(file_size_opts::CONVENTIONAL)
        .or_else(|_| Err(format!("Filter `filesizeformat` was called on a negative number: {}", num).into())) {
        Ok(r) => Ok(to_value(r).unwrap()),
        Err(e) => Err(e)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use serde_json::value::to_value;
    use super::*;

    #[test]
    fn test_pluralize_single() {
        let result = pluralize(to_value(1).unwrap(), HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value("").unwrap());
    }

    #[test]
    fn test_pluralize_multiple() {
        let result = pluralize(to_value(2).unwrap(), HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value("s").unwrap());
    }

    #[test]
    fn test_pluralize_multiple_custom_suffix() {
        let mut args = HashMap::new();
        args.insert("suffix".to_string(), to_value("es").unwrap());
        let result = pluralize(to_value(2).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value("es").unwrap());
    }

    #[test]
    fn test_round_default() {
        let result = round(to_value(2.1).unwrap(), HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(2.0).unwrap());
    }

    #[test]
    fn test_round_default_precision() {
        let mut args = HashMap::new();
        args.insert("precision".to_string(), to_value(2).unwrap());
        let result = round(to_value(3.15159265359).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(3.15).unwrap());
    }

    #[test]
    fn test_round_ceil() {
        let mut args = HashMap::new();
        args.insert("method".to_string(), to_value("ceil").unwrap());
        let result = round(to_value(2.1).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(3.0).unwrap());
    }

    #[test]
    fn test_round_ceil_precision() {
        let mut args = HashMap::new();
        args.insert("method".to_string(), to_value("ceil").unwrap());
        args.insert("precision".to_string(), to_value(1).unwrap());
        let result = round(to_value(2.11).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(2.2).unwrap());
    }

    #[test]
    fn test_round_floor() {
        let mut args = HashMap::new();
        args.insert("method".to_string(), to_value("floor").unwrap());
        let result = round(to_value(2.1).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(2.0).unwrap());
    }

    #[test]
    fn test_round_floor_precision() {
        let mut args = HashMap::new();
        args.insert("method".to_string(), to_value("floor").unwrap());
        args.insert("precision".to_string(), to_value(1).unwrap());
        let result = round(to_value(2.91).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(2.9).unwrap());
    }

    #[test]
    fn test_filesizeformat() {
        let args = HashMap::new();
        let result = filesizeformat(to_value(123456789).unwrap(), args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value("117.74 MB").unwrap());
    }
}
