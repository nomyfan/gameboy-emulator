use crate::boxed::BoxedMatrix;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum Event {
    /// A frame rendered.
    OnFrame(BoxedMatrix<u8, 160, 144>),
    #[cfg(debug_assertions)]
    OnDebugFrame(Vec<[[u8; 8]; 8]>),
}

pub type EventSender = Sender<Event>;
