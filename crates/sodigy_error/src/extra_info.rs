use crate::ErrorContext;

mod endec;

#[derive(Clone, Debug)]
pub struct ExtraErrInfo {
    // very context-specific message for an error,
    // for example, there may be a very specific context for `UnexpectedToken`s (suspicious typos, deprecated features, etc...)
    pub(crate) msg: String,
    pub(crate) context: ErrorContext,
    pub(crate) show_span: bool,
}

impl ExtraErrInfo {
    pub fn none() -> Self {
        ExtraErrInfo {
            msg: String::new(),
            context: ErrorContext::Unknown,
            show_span: true,
        }
    }

    pub fn at_context(context: ErrorContext) -> Self {
        ExtraErrInfo {
            msg: String::new(),
            context,
            show_span: true,
        }
    }

    pub fn has_message(&self) -> bool {
        !self.msg.is_empty()
    }

    pub fn set_error_context(&mut self, context: ErrorContext) -> &mut Self {
        self.context = context;

        self
    }

    pub fn set_message(&mut self, msg: String) -> &mut Self {
        self.msg = msg;

        self
    }

    pub fn set_show_span(&mut self, show_span: bool) -> &mut Self {
        self.show_span = show_span;

        self
    }
}
