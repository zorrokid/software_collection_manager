use std::sync::Arc;

use database::repository_manager::RepositoryManager;
use iced::Task;

use super::{add_release_tab, home_tab, settings_tab};

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Home,
    Settings,
    AddRelease,
}

#[derive(Debug, Clone)]
pub enum Message {
    Home(home_tab::Message),
    Settings(settings_tab::Message),
    AddRelease(add_release_tab::Message),
    RepositoriesTestTaskPerformed(String),
}

pub struct TabsController {
    current_tab: Tab,
    home_tab: home_tab::HomeTab,
    settings_tab: settings_tab::SettingsTab,
    add_release_tab: add_release_tab::AddReleaseTab,
}

impl TabsController {
    pub fn new(
        selected_tab: Option<Tab>,
        repositories: Arc<RepositoryManager>,
    ) -> (Self, Task<Message>) {
        let settings_tab = settings_tab::SettingsTab::new();
        let home_tab = home_tab::HomeTab::new();
        let add_release_tab = add_release_tab::AddReleaseTab::new();

        let repositories_clone = Arc::clone(&repositories);
        let repositories_test_task = Task::perform(
            async move {
                match repositories_clone.settings().get_setting("Test").await {
                    Ok(value) => value,
                    Err(_) => {
                        repositories_clone
                            .settings()
                            .add_setting("Test", "value")
                            .await
                            .unwrap();
                        repositories_clone
                            .settings()
                            .get_setting("Test")
                            .await
                            .unwrap()
                    }
                }
            },
            Message::RepositoriesTestTaskPerformed,
        );

        (
            Self {
                home_tab,
                settings_tab,
                current_tab: selected_tab.unwrap_or(Tab::Home),
                add_release_tab,
            },
            repositories_test_task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Home(message) => self.home_tab.update(message).map(Message::Home),
            Message::Settings(message) => self.settings_tab.update(message).map(Message::Settings),
            Message::RepositoriesTestTaskPerformed(message) => {
                println!("{}", message);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        match self.current_tab {
            Tab::Home => self.home_tab.view().map(Message::Home),
            Tab::Settings => self.settings_tab.view().map(Message::Settings),
            Tab::AddRelease => self.add_release_tab.view().map(Message::AddRelease),
        }
    }

    pub fn switch_to_tab(&mut self, tab: Tab) -> Task<Message> {
        self.current_tab = tab;
        Task::none()
    }
}
