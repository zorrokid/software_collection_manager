use iced::{widget::pick_list, Task};
use service::view_models::SystemListModel;

pub struct SystemsWidget {
    systems: Vec<SystemListModel>,
    selected_system: Option<SystemListModel>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SystemSelected(SystemListModel),
}

impl SystemsWidget {
    pub fn new() -> Self {
        Self {
            systems: vec![],
            selected_system: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self) -> iced::Element<Message> {
        pick_list(
            self.systems.as_slice(),
            self.selected_system.clone(),
            Message::SystemSelected,
        )
        .into()
    }
}
