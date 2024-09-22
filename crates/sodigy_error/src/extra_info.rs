use crate::ErrorContext;

mod endec;

#[derive(Clone, Debug)]
pub struct ExtraErrorInfo {
    // very context-specific message for an error,
    // for example, there may be a very specific context for `UnexpectedToken`s (suspicious typos, deprecated features, etc...)
    pub(crate) extra_messages: Vec<String>,
    pub(crate) context: ErrorContext,
    pub(crate) show_span: bool,
}

impl ExtraErrorInfo {
    pub fn none() -> Self {
        ExtraErrorInfo {
            extra_messages: vec![],
            context: ErrorContext::Unknown,
            show_span: true,
        }
    }

    pub fn at_context(context: ErrorContext) -> Self {
        ExtraErrorInfo {
            extra_messages: vec![],
            context,
            show_span: true,
        }
    }

    pub fn has_extra_message(&self) -> bool {
        !self.extra_messages.is_empty()
    }

    pub fn set_error_context(&mut self, context: ErrorContext) -> &mut Self {
        self.context = context;

        self
    }

    pub fn push_message(&mut self, message: String) -> &mut Self {
        self.extra_messages.push(message);
        self
    }

    pub fn set_show_span(&mut self, show_span: bool) -> &mut Self {
        self.show_span = show_span;

        self
    }
}
