use std::sync::Arc;

use database::{database_error::Error as DatabaseError, repository_manager::RepositoryManager};
use iced::{widget::column, Task};
use service::{error::Error, view_model_service::ViewModelService, view_models::SystemListModel};

use crate::widgets::{
    add_system_widget::{self, AddSystemWidget},
    system_select_widget::{self, SystemSelectWidget},
};

pub struct AddReleaseTab {
    repositories: Arc<RepositoryManager>,
    view_model_service: Arc<ViewModelService>,
    systems: Vec<SystemListModel>,
    systems_widget: SystemSelectWidget,
    add_system_widget: AddSystemWidget,
}

#[derive(Debug, Clone)]
pub enum Message {
    SystemsFetched(Result<Vec<SystemListModel>, Error>),
    AddSystem(crate::widgets::add_system_widget::Message),
    SystemSelect(crate::widgets::system_select_widget::Message),
    SystemAdded(Result<i64, DatabaseError>),
}

impl AddReleaseTab {
    pub fn new(
        repositories: Arc<RepositoryManager>,
        view_model_service: Arc<ViewModelService>,
    ) -> (Self, Task<Message>) {
        let view_model_service_clone = Arc::clone(&view_model_service);
        let fetch_systems_task = Task::perform(
            async move { view_model_service_clone.get_system_list_models().await },
            Message::SystemsFetched,
        );

        (
            Self {
                repositories,
                view_model_service,
                systems: vec![],
                systems_widget: SystemSelectWidget::new(),
                add_system_widget: AddSystemWidget::new(),
            },
            fetch_systems_task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SystemsFetched(result) => match result {
                Ok(systems) => {
                    self.systems = systems;
                    self.systems_widget
                        .update(system_select_widget::Message::SetSystems(
                            self.systems.clone(),
                        ))
                        .map(Message::SystemSelect)
                }
                Err(error) => {
                    eprint!("Error when fetching systems: {}", error);
                    Task::none()
                }
            },
            Message::AddSystem(message) => match self.add_system_widget.update(message) {
                add_system_widget::Action::AddSystem(name) => {
                    let repo = Arc::clone(&self.repositories);
                    Task::perform(
                        async move { repo.get_system_repository().add_system(name).await },
                        Message::SystemAdded,
                    )
                }
                add_system_widget::Action::None => Task::none(),
            },
            Message::SystemSelect(message) => {
                if let system_select_widget::Message::SystemSelected(system) = message {
                    println!("Selected system: {:?}", system);
                    Task::none()
                } else {
                    Task::none()
                }
            }
            Message::SystemAdded(result) => match result {
                Ok(_) => {
                    let service = Arc::clone(&self.view_model_service);
                    Task::perform(
                        async move { service.get_system_list_models().await },
                        Message::SystemsFetched,
                    )
                }
                Err(error) => {
                    eprint!("Error when adding system: {}", error);
                    Task::none()
                }
            },
        }
    }

    pub fn view(&self) -> iced::Element<Message> {
        let add_system_view = self.add_system_widget.view().map(Message::AddSystem);
        let systems_view = self.systems_widget.view().map(Message::SystemSelect);
        column![add_system_view, systems_view].into()
    }
}
