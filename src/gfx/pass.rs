use crate::gfx::texture::*;

pub struct Node<'a> {
    inputs: &'a [Texture],
    outputs: &'a [Texture],
}

impl<'a> Node<'a> {
    fn new(inputs: &'a [Texture], outputs: &'a [Texture]) -> Self {
        Self {
            inputs,
            outputs
        }
    }
}
