use crate::{hotkey, EventCtx, GfxCtx, Key, Line, ModalMenu, SidebarPos, Text};
use std::fmt::Debug;

pub struct ContextMenu<T: Clone + PartialEq + Debug> {
    menu: ModalMenu,
    title: String,
    obj_info: Option<Text>,
    state: State<T>,
}

impl<T: Clone + PartialEq + Debug> ContextMenu<T> {
    pub fn new(title: &str, ctx: &EventCtx, pos: SidebarPos) -> ContextMenu<T> {
        ContextMenu {
            menu: ModalMenu::new(title, Vec::new(), ctx).set_pos(ctx, pos),
            title: title.to_string(),
            obj_info: None,
            state: State::Unfocused,
        }
    }

    // TODO When should this be called, before or after recalculating the current_selection?
    // Returns true if the current_focus changes.
    pub fn event(&mut self, ctx: &mut EventCtx, current_selection: Option<T>) -> bool {
        let mut txt = Text::prompt(&self.title);
        self.state.add_to_prompt(&mut txt);
        if let Some(ref t) = self.obj_info {
            txt.extend(t);
        }
        self.menu.handle_event(ctx, Some(txt));
        let old = self.current_focus();
        self.state.event(current_selection, &mut self.menu, ctx);
        let change = self.current_focus() != old;
        if change {
            self.obj_info = None;
        }
        change
    }

    pub fn set_obj_info(&mut self, info: Text) {
        assert!(self.current_focus().is_some());
        self.obj_info = Some(info);
    }

    // Returns the current focus, if any. Hovering doesn't count.
    pub fn draw(&self, g: &mut GfxCtx) -> Option<T> {
        self.menu.draw(g);
        match self.state {
            State::Focused { ref id, .. } => Some(id.clone()),
            _ => None,
        }
    }

    pub fn current_focus(&self) -> Option<T> {
        match self.state {
            State::Unfocused => None,
            State::Hovering { ref id, .. } | State::Focused { ref id, .. } => Some(id.clone()),
        }
    }

    pub fn action<S: Into<String>>(&mut self, key: Key, raw_name: S, ctx: &mut EventCtx) -> bool {
        self.state.action(key, raw_name, &mut self.menu, ctx)
    }
}

enum State<T: Clone + PartialEq + Debug> {
    Unfocused,
    Hovering {
        id: T,
        actions: Vec<String>,
    },
    Focused {
        id: T,
        actions: Vec<String>,
        hovering: Option<T>,
    },
}

impl<T: Clone + PartialEq + Debug> State<T> {
    fn event(&mut self, current_selection: Option<T>, menu: &mut ModalMenu, ctx: &mut EventCtx) {
        match self {
            State::Unfocused => {
                if let Some(ref id) = current_selection {
                    *self = State::Hovering {
                        id: id.clone(),
                        actions: Vec::new(),
                    };
                }
            }
            State::Hovering {
                ref mut id,
                ref mut actions,
            } => {
                if Some(id.clone()) == current_selection {
                    if ctx.input.left_mouse_button_released() && !ctx.canvas.is_dragging() {
                        *self = State::Focused {
                            id: id.clone(),
                            actions: actions.drain(..).collect(),
                            hovering: None,
                        };
                    }
                } else {
                    for action in actions.drain(..) {
                        menu.remove_action(&action, ctx);
                    }
                    if let Some(other) = current_selection {
                        *id = other;
                    } else {
                        *self = State::Unfocused;
                    }
                }
            }
            State::Focused {
                ref mut id,
                ref mut actions,
                ref mut hovering,
            } => {
                *hovering = current_selection;
                if Some(id.clone()) == hovering.clone() {
                    *hovering = None;
                }
                if ctx.input.left_mouse_button_released() && !ctx.canvas.is_dragging() {
                    for action in actions.drain(..) {
                        menu.remove_action(&action, ctx);
                    }
                    if let Some(other) = hovering.take() {
                        *id = other;
                    } else {
                        *self = State::Unfocused;
                    }
                }
            }
        }
    }

    fn action<S: Into<String>>(
        &mut self,
        key: Key,
        raw_name: S,
        menu: &mut ModalMenu,
        ctx: &mut EventCtx,
    ) -> bool {
        let name = raw_name.into();

        match self {
            State::Unfocused => panic!(
                "action({}) when there's no focused object doesn't make sense",
                name
            ),
            State::Hovering {
                ref mut actions, ..
            }
            | State::Focused {
                ref mut actions, ..
            } => {
                if actions.contains(&name) {
                    if menu.action(&name) {
                        // The world will change, so reset these.
                        for action in actions.drain(..) {
                            menu.remove_action(&action, ctx);
                        }
                        return true;
                    }
                } else {
                    menu.add_action(hotkey(key), &name, ctx);
                    actions.push(name);
                }
                false
            }
        }
    }

    fn add_to_prompt(&self, txt: &mut Text) {
        match self {
            State::Unfocused => {
                txt.add(Line("Unfocused"));
            }
            State::Focused {
                ref id,
                ref hovering,
                ..
            } => {
                txt.add(Line(format!("Focused on {:?}", id)));
                if let Some(ref other) = hovering {
                    txt.add(Line(format!("Click to focus on {:?} instead", other)));
                } else {
                    txt.add(Line("Click to unfocus"));
                }
            }
            State::Hovering { ref id, .. } => {
                txt.add(Line(format!("Click to focus on {:?}", id)));
            }
        }
    }
}
