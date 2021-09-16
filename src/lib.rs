use pikchr_sys::{pikchr, PIKCHR_DARK_MODE, PIKCHR_PLAINTEXT_ERRORS};
use std::ffi::{CStr, CString, NulError};
use std::str::FromStr;
use thiserror::Error;

bitflags::bitflags! {
    /// Flags to configure the render behaviour.
    ///
    /// Note that `PIKCHR_PLAINTEXT_ERRORS` can't be switched off because errors
    /// are handled by pikt.
    #[derive(Default)]
    pub struct Flags: u32 {
        // const PLAINTEXT_ERRORS = PIKCHR_PLAINTEXT_ERRORS;
        const DARK_MODE = PIKCHR_DARK_MODE;
    }
}

/// Represents the set of options the renderer can take.
///
/// Use the [`OptionsBuilder`] to construct it.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    flags: Flags,
    width: u32,
    height: u32,
    class: String,
}

impl Options {
    pub fn flags(&self) -> Flags {
        self.flags
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn class(&self) -> &str {
        &self.class
    }
}

#[derive(Debug, Clone)]
pub struct OptionsBuilder {
    flags: Flags,
    width: u32,
    height: u32,
    class: String,
}

impl Default for OptionsBuilder {
    fn default() -> Self {
        Self {
            flags: Flags::empty(),
            width: 0,
            height: 0,
            class: "pikchr".to_string(),
        }
    }
}

impl OptionsBuilder {
    pub fn flags(&mut self, flags: Flags) {
        self.flags = flags;
    }

    pub fn width(&mut self, width: u32) {
        self.width = width;
    }

    pub fn height(&mut self, height: u32) {
        self.height = height;
    }

    /// Replaces the entire value for `class`. See [`OptionsBuilder.classes`] to append a list of
    /// values.
    ///
    /// By default it already has the value `pikchr`.
    pub fn class(&mut self, class: &str) {
        self.class = class.to_string();
    }

    pub fn classes(&mut self, values: &[&str]) {
        let s = values.join(" ");
        self.class.push_str(" ");
        self.class.push_str(&s);
    }

    /// Builds the set of options.
    ///
    /// ## Example
    ///
    /// ```
    /// use pikt::OptionsBuilder;
    ///
    /// let mut builder = OptionsBuilder::default();
    /// builder.width(300);
    /// builder.height(150);
    /// builder.classes(&vec!["foo", "bar"]);
    /// let options = builder.build();
    ///
    /// assert_eq!(options.width(), 300);
    /// assert_eq!(options.class(), "pikchr foo bar");
    /// ```
    pub fn build(self) -> Options {
        Options {
            flags: self.flags,
            width: self.width,
            height: self.height,
            class: self.class,
        }
    }
}

/// Renders the given pikchr markup as SVG.
///
/// Use [`render_with`] if you want to change the default options.
///
/// ## Example
///
/// ```
/// use pikt::render;
///
/// let markup = r#"
/// circle "1"
/// move
/// circle "2"
/// arrow from first circle.end to last circle.start
/// "#;
/// let svg = render(markup);
///
/// assert!(svg.is_ok());
/// ```
pub fn render(input: &str) -> Result<String, PiktError> {
    let options = OptionsBuilder::default().build();
    render_with(input, options)
}

/// Renders the given pikchr markup as SVG with the given configuration.
///
/// ```
/// use pikt::{render_with, OptionsBuilder, Flags};
///
/// let markup = r#"
/// circle "1"
/// move
/// circle "2"
/// arrow from first circle.end to last circle.start
/// "#;
/// let mut opt_builder = OptionsBuilder::default();
/// opt_builder.flags(Flags::DARK_MODE);
/// opt_builder.classes(&["foo", "bar"]);
/// let options = opt_builder.build();
/// let svg = render_with(markup, options);
///
/// assert!(svg.is_ok());
/// ```
///
/// ## Errors
///
/// It can fail either because the given input has an unexpected NUL terminator or for any of the
/// errors the native pikchr library handles. See [`PiktError`].
pub fn render_with(input: &str, options: Options) -> Result<String, PiktError> {
    use libc::free;
    use std::os::raw::*;

    let mut width: c_int = options.width() as i32;
    let mut height: c_int = options.height() as i32;
    let class = CString::new(options.class())?;
    let input = CString::new(input)?;

    let res: *mut c_char = unsafe {
        pikchr(
            input.as_ptr() as *const c_char,
            class.as_ptr() as *const c_char,
            options.flags().bits() | PIKCHR_PLAINTEXT_ERRORS,
            &mut width as *mut c_int,
            &mut height as *mut c_int,
        )
    };

    let cstr = unsafe { CStr::from_ptr(res) };
    let output = String::from_utf8_lossy(cstr.to_bytes()).into_owned();

    unsafe { free(res as *mut c_void) };

    if width < 0 {
        let err = PiktError::from_str(&output).unwrap();
        return Err(err);
    }

    Ok(output)
}

#[derive(Error, Debug, PartialEq)]
pub enum PiktError {
    /// Raised when the given input has a nul byte.
    #[error("incompatible input. Nul bytes are not allowed.")]
    IncompatibleInput(#[from] NulError),
    #[error("parser stack overflow")]
    ParserStackOverflow,
    #[error("out of memory")]
    OutOfMemory,
    #[error("line {0}, column {1}: division by zero")]
    DivisionByZero(usize, usize),
    #[error("line {0}, column {1}: syntax error")]
    SyntaxError(usize, usize),
    #[error("line {0}, column {1}: arc geometry error")]
    ArcGeometryError(usize, usize),
    #[error("line {0}, column {1}: unknown object")]
    UnknownObject(usize, usize),
    #[error("line {0}, column {1}: unknown object type")]
    UnknownObjectType(usize, usize),
    #[error("line {0}, column {1}: value already set")]
    ValueAlreadySet(usize, usize),
    #[error("line {0}, column {1}: value already fixed by prior constraints")]
    ValueAlreadyFixed(usize, usize),
    #[error("line {0}, column {1}: use with line-oriented objects only")]
    OnlyWithLineOrientedObject(usize, usize),
    #[error("line {0}, column {1}: no prior path points")]
    NoPriorPathPoints(usize, usize),
    #[error("line {0}, column {1}: headings should be between 0 and 360")]
    HeadingOutOfBounds(usize, usize),
    #[error("line {0}, column {1}: use `at` to position this object")]
    MissingAt(usize, usize),
    #[error("line {0}, column {1}: use `from` and `to` to position this object")]
    MissingFromTo(usize, usize),
    #[error("line {0}, column {1}: polygon is closed")]
    ClosedPolygon(usize, usize),
    #[error("line {0}, column {1}: line start position already fixed")]
    StartLineAlreadyFixed(usize, usize),
    #[error("line {0}, column {1}: need at least 3 vertexes in order to close the polygon")]
    TooFewVertexes(usize, usize),
    #[error("line {0}, column {1}: location fixed by prior `at`")]
    PositionAlreadyFixedByAt(usize, usize),
    #[error("line {0}, column {1}: too many text terms")]
    AttributeTooManyTerms(usize, usize),
    #[error("line {0}, column {1}: no text to fit to")]
    AttributeMissingText(usize, usize),
    #[error("line {0}, column {1}: unknown color name")]
    UnknownColorName(usize, usize),
    #[error("line {0}, column {1}: unknown variable")]
    UnknownVariable(usize, usize),
    #[error("line {0}, column {1}: the maximum ordinal is `1000th`")]
    OrdinalOutOfBounds(usize, usize),
    #[error("line {0}, column {1}: no prior objects of the same type")]
    MissingPriorObjectType(usize, usize),
    #[error("line {0}, column {1}: object is not a line")]
    NotALine(usize, usize),
    #[error("line {0}, column {1}: unknown vertex")]
    VertexUnknown(usize, usize),
    #[error("line {0}, column {1}: negative sqrt")]
    NegativeSqrt(usize, usize),
    #[error("line {0}, column {1}: too many macro arguments - max 9")]
    MacroTooManyArguments(usize, usize),
    #[error("line {0}, column {1}: unterminated macro argument list")]
    MacroUnterminatedArgumentList(usize, usize),
    #[error("line {0}, column {1}: token is too long - max length 50000 bytes")]
    TokenTooLong(usize, usize),
    #[error("line {0}, column {1}: unknown token")]
    TokenUnknown(usize, usize),
    #[error("line {0}, column {1}: macros nested too deep")]
    MacroTooDeep(usize, usize),
    #[error("line {0}, column {1}: recursive macro definition")]
    MacroRecursive(usize, usize),

    /// Raised when the given pikchr input cannot be parsed by Pikchr for an unknown reason.
    #[error("line {1}, column {2}: {0}")]
    Other(String, usize, usize),
}

impl FromStr for PiktError {
    type Err = PiktError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PiktError::*;

        if s.contains("parser stack overflow") {
            return Ok(ParserStackOverflow);
        }
        if s.contains("Out of memory") {
            return Ok(OutOfMemory);
        }

        let line_padding = 12;
        let mut line_num = 0;
        let mut col_num = 0;
        let mut lines = s.lines();
        let mut message = "unknown error";

        while let Some(line) = lines.next() {
            // markup lines are formatted like:
            //
            // /*    1 */  circle "1"
            if line.starts_with("/*") {
                line_num = line_num + 1;
            }

            // caret lines always end with a caret. multiple carets are ignored.
            if line.ends_with('^') {
                col_num = line.len() + 1 - line_padding;
            }

            // the last line always follow a pattern like:
            //
            // ERROR: <error message>
            if line.starts_with("ERROR:") {
                if let Some((_, msg)) = line.split_once(' ') {
                    message = msg
                };
            }
        }

        let err = match message {
            "division by zero" => DivisionByZero(line_num, col_num),
            "syntax error" => SyntaxError(line_num, col_num),
            "arc geometry error" => ArcGeometryError(line_num, col_num),
            "unknown object type" => UnknownObjectType(line_num, col_num),
            "no such object" => UnknownObject(line_num, col_num),
            "value is already set" => ValueAlreadySet(line_num, col_num),
            "value already fixed by prior constraints" => ValueAlreadyFixed(line_num, col_num),
            "use with line-oriented objects only" => OnlyWithLineOrientedObject(line_num, col_num),
            "no prior path points" => NoPriorPathPoints(line_num, col_num),
            "too many path elements" => NoPriorPathPoints(line_num, col_num),
            "headings should be between 0 and 360" => HeadingOutOfBounds(line_num, col_num),
            "use \"at\" to position this object" => MissingAt(line_num, col_num),
            "use \"from\" and \"to\" to position this object" => MissingFromTo(line_num, col_num),
            "polygon is closed" => ClosedPolygon(line_num, col_num),
            "need at least 3 vertexes in order to close the polygon" => {
                TooFewVertexes(line_num, col_num)
            }
            "line start location already fixed" => StartLineAlreadyFixed(line_num, col_num),
            "location fixed by prior \"at\"" => PositionAlreadyFixedByAt(line_num, col_num),
            "too many text terms" => AttributeTooManyTerms(line_num, col_num),
            "no text to fit to" => AttributeMissingText(line_num, col_num),
            "not a known color name" => UnknownColorName(line_num, col_num),
            "no such variable" => UnknownVariable(line_num, col_num),
            "value too big - max '1000th'" => OrdinalOutOfBounds(line_num, col_num),
            "no prior objects of the same type" => MissingPriorObjectType(line_num, col_num),
            "object is not a line" => NotALine(line_num, col_num),
            "no such vertex" => VertexUnknown(line_num, col_num),
            "sqrt of negative value" => NegativeSqrt(line_num, col_num),
            "too many macro arguments - max 9" => MacroTooManyArguments(line_num, col_num),
            "unterminated macro argument list" => MacroUnterminatedArgumentList(line_num, col_num),
            "token is too long - max length 50000 bytes" => TokenTooLong(line_num, col_num),
            "unrecognized token" => TokenUnknown(line_num, col_num),
            "macros nested too deep" => MacroTooDeep(line_num, col_num),
            "recursive macro definition" => MacroRecursive(line_num, col_num),
            msg => Other(msg.to_string(), line_num, col_num),
        };

        Ok(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_box() -> Result<(), PiktError> {
        let source = "box \"pikchr\"";
        let expected = "<svg xmlns='http://www.w3.org/2000/svg' class=\"pikchr\" viewBox=\"0 0 112.32 76.32\">\n<path d=\"M2,74L110,74L110,2L2,2Z\"  style=\"fill:none;stroke-width:2.16;stroke:rgb(0,0,0);\" />\n<text x=\"56\" y=\"38\" text-anchor=\"middle\" fill=\"rgb(0,0,0)\" dominant-baseline=\"central\">pikchr</text>\n</svg>\n";

        let actual = render(source)?;

        assert_eq!(&actual, expected);

        Ok(())
    }

    #[test]
    fn input_with_nul() {
        let source = "box \"pikchr\"\0";

        let actual = render(source);

        assert!(actual.is_err(), "expected a nul pointer error");
    }

    #[test]
    fn malformed_input() {
        let source = "box 'pikchr'";

        let actual = render(source);

        assert_eq!(
            actual.expect_err("expected unknown token"),
            PiktError::TokenUnknown(1, 5)
        );
    }

    #[test]
    fn division_by_zero() {
        let source = r#"box "pikchr"
        arrow from first box to (0/0, 0)
        "#;

        let actual = render(source);

        assert_eq!(
            actual.expect_err("expected div by zero err"),
            PiktError::DivisionByZero(2, 36)
        );
    }

    #[test]
    fn syntax_error() {
        let source = r#"circ "1""#;

        let actual = render(source);

        assert_eq!(
            actual.expect_err("expected syntax error"),
            PiktError::SyntaxError(1, 8)
        );
    }

    #[test]
    fn unknown_object() {
        let source = r#"arrow from A to B"#;

        let actual = render(source);

        assert_eq!(
            actual.expect_err("expected unknown object"),
            PiktError::UnknownObject(1, 12)
        );
    }

    #[test]
    fn box_dark_mode() -> Result<(), PiktError> {
        let source = "box \"pikchr\"";
        let expected = "<svg xmlns='http://www.w3.org/2000/svg' class=\"pikchr\" viewBox=\"0 0 112.32 76.32\">\n<path d=\"M2,74L110,74L110,2L2,2Z\"  style=\"fill:none;stroke-width:2.16;stroke:rgb(255,255,255);\" />\n<text x=\"56\" y=\"38\" text-anchor=\"middle\" fill=\"rgb(255,255,255)\" dominant-baseline=\"central\">pikchr</text>\n</svg>\n";
        let mut flags = Flags::default();
        flags.insert(Flags::DARK_MODE);

        let mut builder = OptionsBuilder::default();
        builder.flags(flags);
        let options = builder.build();

        let actual = render_with(source, options)?;

        assert_eq!(&actual, expected);

        Ok(())
    }
}
