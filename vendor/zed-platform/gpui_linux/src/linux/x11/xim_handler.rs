use std::default::Default;

use x11rb::protocol::{Event, xproto};
use xim::{AHashMap, AttributeName, Client, ClientError, ClientHandler, InputStyle};

pub enum XimCallbackEvent {
    XimXEvent(x11rb::protocol::Event),
    XimPreeditEvent(xproto::Window, String),
    XimCommitEvent(xproto::Window, String),
}

fn forwarded_key_event(xev: xproto::KeyPressEvent) -> Option<XimCallbackEvent> {
    // XSendEvent sets the high bit; XIM can return it unchanged. It is not part
    // of the event type, including for keys forwarded by embedded windows.
    match xev.response_type & 0x7f {
        xproto::KEY_PRESS_EVENT => Some(XimCallbackEvent::XimXEvent(Event::KeyPress(xev))),
        xproto::KEY_RELEASE_EVENT => Some(XimCallbackEvent::XimXEvent(Event::KeyRelease(xev))),
        _ => None,
    }
}

pub struct XimHandler {
    pub im_id: u16,
    pub ic_id: u16,
    pub connected: bool,
    pub window: xproto::Window,
    pub last_callback_event: Option<XimCallbackEvent>,
}

impl XimHandler {
    pub fn new() -> Self {
        Self {
            im_id: Default::default(),
            ic_id: Default::default(),
            connected: false,
            window: Default::default(),
            last_callback_event: None,
        }
    }
}

impl<C: Client<XEvent = xproto::KeyPressEvent>> ClientHandler<C> for XimHandler {
    fn handle_connect(&mut self, client: &mut C) -> Result<(), ClientError> {
        client.open("C")
    }

    fn handle_open(&mut self, client: &mut C, input_method_id: u16) -> Result<(), ClientError> {
        self.im_id = input_method_id;

        client.get_im_values(input_method_id, &[AttributeName::QueryInputStyle])
    }

    fn handle_get_im_values(
        &mut self,
        client: &mut C,
        input_method_id: u16,
        _attributes: AHashMap<AttributeName, Vec<u8>>,
    ) -> Result<(), ClientError> {
        let ic_attributes = client
            .build_ic_attributes()
            .push(AttributeName::InputStyle, InputStyle::PREEDIT_CALLBACKS)
            .push(AttributeName::ClientWindow, self.window)
            .push(AttributeName::FocusWindow, self.window)
            .build();
        client.create_ic(input_method_id, ic_attributes)
    }

    fn handle_create_ic(
        &mut self,
        _client: &mut C,
        _input_method_id: u16,
        input_context_id: u16,
    ) -> Result<(), ClientError> {
        self.connected = true;
        self.ic_id = input_context_id;
        Ok(())
    }

    fn handle_commit(
        &mut self,
        _client: &mut C,
        _input_method_id: u16,
        _input_context_id: u16,
        text: &str,
    ) -> Result<(), ClientError> {
        self.last_callback_event =
            Some(XimCallbackEvent::XimCommitEvent(self.window, String::from(text)));
        Ok(())
    }

    fn handle_forward_event(
        &mut self,
        _client: &mut C,
        _input_method_id: u16,
        _input_context_id: u16,
        _flag: xim::ForwardEventFlag,
        xev: C::XEvent,
    ) -> Result<(), ClientError> {
        if let Some(event) = forwarded_key_event(xev) {
            self.last_callback_event = Some(event);
        }
        Ok(())
    }

    fn handle_close(&mut self, client: &mut C, _input_method_id: u16) -> Result<(), ClientError> {
        client.disconnect()
    }

    fn handle_preedit_draw(
        &mut self,
        _client: &mut C,
        _input_method_id: u16,
        _input_context_id: u16,
        _caret: i32,
        _chg_first: i32,
        _chg_len: i32,
        _status: xim::PreeditDrawStatus,
        preedit_string: &str,
        _feedbacks: Vec<xim::Feedback>,
    ) -> Result<(), ClientError> {
        // XIMReverse: 1, XIMPrimary: 8, XIMTertiary: 32: selected text
        // XIMUnderline: 2, XIMSecondary: 16: underlined text
        // XIMHighlight: 4: normal text
        // XIMVisibleToForward: 64, XIMVisibleToBackward: 128, XIMVisibleCenter: 256: text align position
        // XIMPrimary, XIMHighlight, XIMSecondary, XIMTertiary are not specified,
        // but interchangeable as above
        // Currently there's no way to support these.
        self.last_callback_event =
            Some(XimCallbackEvent::XimPreeditEvent(self.window, String::from(preedit_string)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use x11rb::x11_utils::Serialize;

    #[test]
    fn xim_preserves_native_and_forwarded_key_presses_and_releases() {
        for flag in [0, 0x80] {
            for event_type in [xproto::KEY_PRESS_EVENT, xproto::KEY_RELEASE_EVENT] {
                let key = xproto::KeyPressEvent {
                    response_type: event_type | flag,
                    detail: 38,
                    event: 123,
                    time: 456,
                    state: xproto::KeyButMask::CONTROL | xproto::KeyButMask::SHIFT,
                    ..Default::default()
                };
                let Some(XimCallbackEvent::XimXEvent(event)) = forwarded_key_event(key) else {
                    panic!("XIM dropped key type {event_type} with flag {flag}");
                };
                match event {
                    Event::KeyPress(received) if event_type == xproto::KEY_PRESS_EVENT => {
                        assert_eq!(received.serialize(), key.serialize());
                    }
                    Event::KeyRelease(received) if event_type == xproto::KEY_RELEASE_EVENT => {
                        assert_eq!(received.serialize(), key.serialize());
                    }
                    _ => panic!("XIM changed the key event type"),
                }
            }
        }
        assert!(
            forwarded_key_event(xproto::KeyPressEvent {
                response_type: xproto::FOCUS_IN_EVENT | 0x80,
                ..Default::default()
            })
            .is_none()
        );
    }
}
