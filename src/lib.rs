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
        self.class.push(' ');
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
#[error("line {line}, column {column}: {reason}")]
pub struct PiktError {
    line: usize,
    column: usize,
    reason: PiktErrorReason,
}

#[derive(Error, Debug, PartialEq)]
pub enum PiktErrorReason {
    /// Raised when the given input has a nul byte.
    #[error("incompatible input. Nul bytes are not allowed.")]
    IncompatibleInput(NulError),
    #[error("parser stack overflow")]
    ParserStackOverflow,
    #[error("out of memory")]
    OutOfMemory,
    #[error("division by zero")]
    DivisionByZero,
    #[error("syntax error")]
    SyntaxError,
    #[error("arc geometry error")]
    ArcGeometryError,
    #[error("unknown object")]
    UnknownObject,
    #[error("unknown object type")]
    UnknownObjectType,
    #[error("value already set")]
    ValueAlreadySet,
    #[error("value already fixed by prior constraints")]
    ValueAlreadyFixed,
    #[error("use with line-oriented objects only")]
    OnlyWithLineOrientedObject,
    #[error("no prior path points")]
    NoPriorPathPoints,
    #[error("headings should be between 0 and 360")]
    HeadingOutOfBounds,
    #[error("use `at` to position this object")]
    MissingAt,
    #[error("use `from` and `to` to position this object")]
    MissingFromTo,
    #[error("polygon is closed")]
    ClosedPolygon,
    #[error("line start position already fixed")]
    StartLineAlreadyFixed,
    #[error("need at least 3 vertexes in order to close the polygon")]
    TooFewVertexes,
    #[error("location fixed by prior `at`")]
    PositionAlreadyFixedByAt,
    #[error("too many text terms")]
    AttributeTooManyTerms,
    #[error("no text to fit to")]
    AttributeMissingText,
    #[error("unknown color name")]
    UnknownColorName,
    #[error("unknown variable")]
    UnknownVariable,
    #[error("the maximum ordinal is `1000th`")]
    OrdinalOutOfBounds,
    #[error("no prior objects of the same type")]
    MissingPriorObjectType,
    #[error("object is not a line")]
    NotALine,
    #[error("unknown vertex")]
    VertexUnknown,
    #[error("negative sqrt")]
    NegativeSqrt,
    #[error("too many macro arguments - max 9")]
    MacroTooManyArguments,
    #[error("unterminated macro argument list")]
    MacroUnterminatedArgumentList,
    #[error("token is too long - max length 50000 bytes")]
    TokenTooLong,
    #[error("unknown token")]
    TokenUnknown,
    #[error("macros nested too deep")]
    MacroTooDeep,
    #[error("recursive macro definition")]
    MacroRecursive,

    /// Raised when the given pikchr input cannot be parsed by Pikchr for an unknown reason.
    #[error("other")]
    Other(String),
}

impl FromStr for PiktError {
    type Err = PiktError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PiktErrorReason::*;

        if s.contains("parser stack overflow") {
            return Ok(PiktError {
                line: 0,
                column: 0,
                reason: ParserStackOverflow,
            });
        }
        if s.contains("Out of memory") {
            return Ok(PiktError {
                line: 0,
                column: 0,
                reason: OutOfMemory,
            });
        }

        let line_padding = 12;
        let lines = s.lines();
        let mut message = "unknown error";
        let mut err = PiktError {
            line: 0,
            column: 0,
            reason: Other(message.to_string()),
        };

        for line in lines {
            // markup lines are formatted like:
            //
            // /*    1 */  circle "1"
            if line.starts_with("/*") {
                err.line += 1;
            }

            // caret lines always end with a caret. multiple carets are ignored.
            if line.ends_with('^') {
                err.column = line.len() + 1 - line_padding;
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

        err.reason = match message {
            "division by zero" => DivisionByZero,
            "syntax error" => SyntaxError,
            "arc geometry error" => ArcGeometryError,
            "unknown object type" => UnknownObjectType,
            "no such object" => UnknownObject,
            "value is already set" => ValueAlreadySet,
            "value already fixed by prior constraints" => ValueAlreadyFixed,
            "use with line-oriented objects only" => OnlyWithLineOrientedObject,
            "no prior path points" => NoPriorPathPoints,
            "too many path elements" => NoPriorPathPoints,
            "headings should be between 0 and 360" => HeadingOutOfBounds,
            "use \"at\" to position this object" => MissingAt,
            "use \"from\" and \"to\" to position this object" => MissingFromTo,
            "polygon is closed" => ClosedPolygon,
            "need at least 3 vertexes in order to close the polygon" => TooFewVertexes,
            "line start location already fixed" => StartLineAlreadyFixed,
            "location fixed by prior \"at\"" => PositionAlreadyFixedByAt,
            "too many text terms" => AttributeTooManyTerms,
            "no text to fit to" => AttributeMissingText,
            "not a known color name" => UnknownColorName,
            "no such variable" => UnknownVariable,
            "value too big - max '1000th'" => OrdinalOutOfBounds,
            "no prior objects of the same type" => MissingPriorObjectType,
            "object is not a line" => NotALine,
            "no such vertex" => VertexUnknown,
            "sqrt of negative value" => NegativeSqrt,
            "too many macro arguments - max 9" => MacroTooManyArguments,
            "unterminated macro argument list" => MacroUnterminatedArgumentList,
            "token is too long - max length 50000 bytes" => TokenTooLong,
            "unrecognized token" => TokenUnknown,
            "macros nested too deep" => MacroTooDeep,
            "recursive macro definition" => MacroRecursive,
            msg => Other(msg.to_string()),
        };

        Ok(err)
    }
}

impl From<NulError> for PiktError {
    fn from(err: NulError) -> Self {
        Self {
            line: 0,
            column: 0,
            reason: PiktErrorReason::IncompatibleInput(err),
        }
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
            PiktError {
                line: 1,
                column: 5,
                reason: PiktErrorReason::TokenUnknown,
            }
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
            PiktError {
                line: 2,
                column: 36,
                reason: PiktErrorReason::DivisionByZero,
            }
        );
    }

    #[test]
    fn syntax_error() {
        let source = r#"circ "1""#;

        let actual = render(source);

        assert_eq!(
            actual.expect_err("expected syntax error"),
            PiktError {
                line: 1,
                column: 8,
                reason: PiktErrorReason::SyntaxError,
            }
        );
    }

    #[test]
    fn unknown_object() {
        let source = r#"arrow from A to B"#;

        let actual = render(source);

        assert_eq!(
            actual.expect_err("expected unknown object"),
            PiktError {
                line: 1,
                column: 12,
                reason: PiktErrorReason::UnknownObject,
            }
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
