use crate::helpers::ID;
use crate::ui::UI;
use ezgui::{hotkey, EventCtx, GfxCtx, Key, Line, ModalMenu, SidebarPos, Text};

// TODO Move/generalize.
// This wraps the menu entirely. Not sure if everyone will want this or not.
pub struct ContextMenu {
    menu: ModalMenu,
    title: String,
    state: State,
}

impl ContextMenu {
    pub fn new(title: &str, ctx: &EventCtx) -> ContextMenu {
        ContextMenu {
            menu: ModalMenu::new(title, Vec::new(), ctx).set_pos(ctx, SidebarPos::Left),
            title: title.to_string(),
            state: State::new(),
        }
    }

    // TODO When should this be called, before or after recalculate_current_selection?
    pub fn event(&mut self, ctx: &mut EventCtx, ui: &UI) {
        let mut txt = Text::prompt(&self.title);
        self.state.add_to_prompt(&mut txt);
        self.menu.handle_event(ctx, Some(txt));

        self.state
            .event(ui.primary.current_selection.clone(), &mut self.menu, ctx);
    }

    pub fn draw(&self, g: &mut GfxCtx, ui: &UI) {
        self.state.draw(g, ui);
        self.menu.draw(g);
    }

    pub fn current_focus(&self) -> Option<ID> {
        self.state.current_focus()
    }

    pub fn action<S: Into<String>>(&mut self, key: Key, raw_name: S, ctx: &mut EventCtx) -> bool {
        self.state.action(key, raw_name, &mut self.menu, ctx)
    }
}

enum State {
    Unfocused,
    Hovering {
        id: ID,
        actions: Vec<String>,
    },
    Focused {
        id: ID,
        actions: Vec<String>,
        hovering: Option<ID>,
    },
}

impl State {
    fn new() -> State {
        State::Unfocused
    }

    fn current_focus(&self) -> Option<ID> {
        match self {
            State::Unfocused => None,
            State::Hovering { ref id, .. } | State::Focused { ref id, .. } => Some(id.clone()),
        }
    }

    fn event(&mut self, current_selection: Option<ID>, menu: &mut ModalMenu, ctx: &mut EventCtx) {
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

    fn draw(&self, g: &mut GfxCtx, ui: &UI) {
        if let State::Focused { ref id, .. } = self {
            g.draw_polygon(
                // TODO Or a diff color?
                ui.cs.get("selected"),
                &ui.primary
                    .draw_map
                    .get_renderable(id.clone(), &ui.primary.draw_map.agents.borrow())
                    .get_outline(&ui.primary.map),
            );
        }
    }
}
