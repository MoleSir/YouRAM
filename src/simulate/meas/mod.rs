mod voltageat;
mod delay;

pub use voltageat::*;
pub use delay::*;

use std::num::ParseFloatError;
use regex::Regex;
use reda_unit::Number;

#[derive(Debug, thiserror::Error)]
pub enum MeasError {
    #[error("meas '{0}' not found")]
    NoMeasResultFound(String),

    #[error("parse value '{0}' failed for '{1}'")]
    ParseValue(String, ParseFloatError),

    #[error("meas '{0}''s value not found")]
    NoMeasValueFound(String),
}

pub trait Meas {
    fn name(&self) -> &str; 
    fn write_command(&self, out: &mut dyn std::io::Write) -> std::io::Result<()>;
    fn get_result(&self, context: &str) -> Result<Number, MeasError> {
        let pattern = format!(r"{}\s*=\s*-?\d+\.?\d*[eE]?[-+]?\d+", regex::escape(&self.name()));
        let re = Regex::new(&pattern).unwrap();

        let mat = match re.find(context) {
            Some(m) => m,
            None => return Err(MeasError::NoMeasResultFound(self.name().to_string())),
        };
        // "Vout = -1.2345e-3"
        let name_value = mat.as_str();

        let eq_pos = name_value.find('=').unwrap();
        let after_eq = &name_value[eq_pos + 1..].trim_start();

        let val = match after_eq.split_whitespace().next() {
            Some(num_str) => match num_str.parse::<f64>() {
                Ok(v) => v,
                Err(e) => return Err(MeasError::ParseValue(num_str.into(), e))
            },
            None => return Err(MeasError::NoMeasValueFound(self.name().to_string())),
        };

        Ok(Number::from_f64(val))
    }
}
