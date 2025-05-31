use std::sync::Arc;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub(super) enum Identifier {
    Str(Box<str>), // cheap clone
    UserEvents,
    OrderUpdates,
}

pub(super) type Decoded = Arc<crate::ws::Message>;
