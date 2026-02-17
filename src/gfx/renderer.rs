use std::{sync::Arc};

use crate::gfx::Context;

pub trait Renderer {
    fn new(context: Arc<Context>) -> Self;
}

pub struct DeferredRenderer {
    context: Arc<Context>,
}

impl Renderer for DeferredRenderer {
    fn new(context: Arc<Context>) -> Self {
        Self {
            context
        }
    }
}
