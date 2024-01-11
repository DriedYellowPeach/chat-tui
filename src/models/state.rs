use crate::action::StateModelAction;

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum StateModel {
    #[default]
    Chats,
    Messages,
    FPS,
}

impl StateModel {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn handle_action(&mut self, action: StateModelAction) {
        match action {
            StateModelAction::NextState => self.act_on_next(),
            StateModelAction::SetActive(s) => self.act_on_set_active(s),
        }
    }

    fn act_on_next(&mut self) {
        match self {
            Self::Chats => *self = Self::Messages,
            Self::Messages => *self = Self::FPS,
            Self::FPS => *self = Self::Chats,
        }
    }

    fn act_on_set_active(&mut self, s: StateModel) {
        *self = s;
    }
}
