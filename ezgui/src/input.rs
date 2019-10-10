use crate::{text, Canvas, Event, Key, Line, ScreenPt, Text};
use std::collections::HashMap;

// As we check for user input, record the input and the thing that would happen. This will let us
// build up some kind of OSD of possible actions.
pub struct UserInput {
    pub(crate) event: Event,
    pub(crate) event_consumed: bool,
    important_actions: Vec<(Key, String)>,
    // If two different callers both expect the same key, there's likely an unintentional conflict.
    reserved_keys: HashMap<Key, String>,
}

impl UserInput {
    pub(crate) fn new(event: Event, canvas: &mut Canvas) -> UserInput {
        let input = UserInput {
            event,
            event_consumed: false,
            important_actions: Vec::new(),
            reserved_keys: HashMap::new(),
        };

        // First things first...
        if let Event::WindowResized(width, height) = input.event {
            canvas.window_width = width;
            canvas.window_height = height;
        }

        if input.event == Event::KeyPress(Key::LeftControl) {
            canvas.lctrl_held = true;
        }
        if input.event == Event::KeyRelease(Key::LeftControl) {
            canvas.lctrl_held = false;
        }

        input
    }

    pub fn key_pressed(&mut self, key: Key, action: &str) -> bool {
        self.reserve_key(key, action);

        self.important_actions.push((key, action.to_string()));

        if self.event_consumed {
            return false;
        }

        if self.event == Event::KeyPress(key) {
            self.consume_event();
            return true;
        }
        false
    }

    pub fn unimportant_key_pressed(&mut self, key: Key, action: &str) -> bool {
        self.reserve_key(key, action);

        if self.event_consumed {
            return false;
        }

        if self.event == Event::KeyPress(key) {
            self.consume_event();
            return true;
        }
        false
    }

    pub fn key_released(&mut self, key: Key) -> bool {
        if self.event_consumed {
            return false;
        }

        if self.event == Event::KeyRelease(key) {
            self.consume_event();
            return true;
        }
        false
    }

    // No consuming for these?
    pub fn left_mouse_button_pressed(&mut self) -> bool {
        self.event == Event::LeftMouseButtonDown
    }
    pub fn left_mouse_button_released(&mut self) -> bool {
        self.event == Event::LeftMouseButtonUp
    }

    pub(crate) fn window_gained_cursor(&mut self) -> bool {
        self.event == Event::WindowGainedCursor
    }
    pub fn window_lost_cursor(&self) -> bool {
        self.event == Event::WindowLostCursor
    }

    pub fn get_moved_mouse(&self) -> Option<ScreenPt> {
        if let Event::MouseMovedTo(pt) = self.event {
            return Some(pt);
        }
        None
    }

    pub(crate) fn get_mouse_scroll(&self) -> Option<f64> {
        if let Event::MouseWheelScroll(dy) = self.event {
            return Some(dy);
        }
        None
    }

    pub fn nonblocking_is_update_event(&mut self) -> bool {
        if self.event_consumed {
            return false;
        }

        self.event == Event::Update
    }
    pub fn use_update_event(&mut self) {
        self.consume_event();
        assert!(self.event == Event::Update)
    }

    pub fn nonblocking_is_keypress_event(&mut self) -> bool {
        if self.event_consumed {
            return false;
        }

        match self.event {
            Event::KeyPress(_) => true,
            _ => false,
        }
    }

    // TODO I'm not sure this is even useful anymore
    pub(crate) fn use_event_directly(&mut self) -> Option<Event> {
        if self.event_consumed {
            return None;
        }
        self.consume_event();
        Some(self.event)
    }

    fn consume_event(&mut self) {
        assert!(!self.event_consumed);
        self.event_consumed = true;
    }

    // Just for Wizard
    pub(crate) fn has_been_consumed(&self) -> bool {
        self.event_consumed
    }

    pub fn populate_osd(&mut self, osd: &mut Text) {
        for (key, a) in self.important_actions.drain(..) {
            osd.add_appended(vec![
                Line("Press "),
                Line(key.describe()).fg(text::HOTKEY_COLOR),
                Line(format!(" to {}", a)),
            ]);
        }
    }

    fn reserve_key(&mut self, key: Key, action: &str) {
        if let Some(prev_action) = self.reserved_keys.get(&key) {
            println!("both {} and {} read key {:?}", prev_action, action, key);
        }
        self.reserved_keys.insert(key, action.to_string());
    }
}
