use crate::ErrorContext;

mod endec;

#[derive(Clone, Debug)]
pub struct ExtraErrorInfo {
    // very context-specific message for an error,
    // for example, there may be a very specific context for `UnexpectedToken`s (suspicious typos, deprecated features, etc...)
    pub(crate) msg: String,
    pub(crate) context: ErrorContext,
    pub(crate) show_span: bool,
}

impl ExtraErrorInfo {
    pub fn none() -> Self {
        ExtraErrorInfo {
            msg: String::new(),
            context: ErrorContext::Unknown,
            show_span: true,
        }
    }

    pub fn at_context(context: ErrorContext) -> Self {
        ExtraErrorInfo {
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
        // I want to make sure that it doesn't override previous message
        // If there're previous ones, I want it to be stacked (TODO)
        debug_assert!(self.msg.is_empty());

        self.msg = msg;
        self
    }

    pub fn set_show_span(&mut self, show_span: bool) -> &mut Self {
        self.show_span = show_span;

        self
    }
}
