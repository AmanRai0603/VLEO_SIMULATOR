//! A very small JSON writer.
//!
//! Hand-rolled rather than pulled in, for one reason worth stating: the shapes
//! this daemon emits are generated from the same tables the engine walks, so a
//! serialisation library would be a second description of them. Forty lines
//! that do nothing clever are cheaper to review than a derive whose output
//! nobody reads.
//!
//! Bulk numeric output does not come through here. A field or a sweep is
//! written as a flat array of numbers, because serialising a field costs more
//! than computing it.

pub struct Json(pub String);

impl Json {
    pub fn new() -> Json {
        Json(String::new())
    }
    pub fn raw(&mut self, s: &str) -> &mut Self {
        self.0.push_str(s);
        self
    }
    pub fn str_field(&mut self, k: &str, v: &str) -> &mut Self {
        self.comma();
        self.0.push('"');
        self.0.push_str(k);
        self.0.push_str("\":");
        self.push_string(v);
        self
    }
    pub fn num_field(&mut self, k: &str, v: f64) -> &mut Self {
        self.comma();
        self.0.push('"');
        self.0.push_str(k);
        self.0.push_str("\":");
        self.0.push_str(&num(v));
        self
    }
    pub fn bool_field(&mut self, k: &str, v: bool) -> &mut Self {
        self.comma();
        self.0.push('"');
        self.0.push_str(k);
        self.0.push_str("\":");
        self.0.push_str(if v { "true" } else { "false" });
        self
    }
    /// Open an object in an array context, emitting the separating comma. Kept
    /// beside `close_obj` because a writer with only half the pair is a writer
    /// somebody will finish badly.
    #[allow(dead_code)]
    pub fn open_obj(&mut self) -> &mut Self {
        self.comma();
        self.0.push('{');
        self
    }
    pub fn close_obj(&mut self) -> &mut Self {
        self.0.push('}');
        self
    }
    pub fn key(&mut self, k: &str) -> &mut Self {
        self.comma();
        self.0.push('"');
        self.0.push_str(k);
        self.0.push_str("\":");
        self
    }
    pub fn open_arr(&mut self) -> &mut Self {
        self.0.push('[');
        self
    }
    pub fn close_arr(&mut self) -> &mut Self {
        self.0.push(']');
        self
    }
    fn comma(&mut self) {
        if let Some(c) = self.0.chars().last() {
            if c != '{' && c != '[' && c != ':' && !self.0.is_empty() {
                self.0.push(',');
            }
        }
    }
    pub fn push_string(&mut self, v: &str) {
        self.0.push('"');
        for c in v.chars() {
            match c {
                '"' => self.0.push_str("\\\""),
                '\\' => self.0.push_str("\\\\"),
                '\n' => self.0.push_str("\\n"),
                '\r' => self.0.push_str("\\r"),
                '\t' => self.0.push_str("\\t"),
                c if (c as u32) < 0x20 => self.0.push_str(&format!("\\u{:04x}", c as u32)),
                c => self.0.push(c),
            }
        }
        self.0.push('"');
    }
}

/// One escaped JSON string, for a message built by hand.
///
/// The escaping already exists on `Json::push_string`; this reaches it without a
/// second copy of the rules, which is where an escaping bug would hide.
pub fn string(v: &str) -> String {
    let mut j = Json::new();
    j.push_string(v);
    j.0
}

impl Default for Json {
    fn default() -> Self {
        Json::new()
    }
}

/// JSON has no infinity and no NaN. A value that is not a number is a fault,
/// and a fault is reported as a fault rather than smuggled through as `null`.
pub fn num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:?}")
    } else {
        "null".to_string()
    }
}
