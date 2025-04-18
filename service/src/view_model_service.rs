use std::sync::Arc;

use database::{database_error::DatabaseError, repository_manager::RepositoryManager};

use crate::{
    error::Error,
    view_models::{EmulatorSystemViewModel, EmulatorViewModel, Settings, SystemListModel},
};

pub struct ViewModelService {
    repository_manager: Arc<RepositoryManager>,
}

impl ViewModelService {
    pub fn new(repository_manager: Arc<RepositoryManager>) -> Self {
        Self { repository_manager }
    }

    pub async fn get_emulator_view_model(
        &self,
        emulator_id: i64,
    ) -> Result<EmulatorViewModel, DatabaseError> {
        let (emulator, emulator_systems) = self
            .repository_manager
            .get_emulator_repository()
            .get_emulator_with_systems(emulator_id)
            .await?;

        Ok(EmulatorViewModel {
            id: emulator.id,
            name: emulator.name,
            executable: emulator.executable,
            extract_files: emulator.extract_files,
            systems: emulator_systems
                .into_iter()
                .map(|es| EmulatorSystemViewModel {
                    system_id: es.system_id,
                    system_name: es.system_name,
                    arguments: es.arguments,
                })
                .collect(),
        })
    }

    pub async fn get_settings(&self) -> Result<Settings, DatabaseError> {
        let settings_map = self.repository_manager.settings().get_settings().await?;
        Ok(Settings::from(settings_map))
    }

    pub async fn get_system_list_models(&self) -> Result<Vec<SystemListModel>, Error> {
        let systems = self
            .repository_manager
            .get_system_repository()
            .get_systems()
            .await
            .map_err(|err| Error::DbError(err.to_string()))?;

        let mut list_models: Vec<SystemListModel> =
            systems.iter().map(SystemListModel::from).collect();

        for system in list_models.iter_mut() {
            system.can_delete = !self
                .repository_manager
                .get_system_repository()
                .is_system_in_use(system.id)
                .await
                .map_err(|err| Error::DbError(err.to_string()))?;
        }

        Ok(list_models)
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;
    use database::{models::SettingName, setup_test_db};

    #[async_std::test]
    async fn test_get_emulator_view_model() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        let repository_manager = Arc::new(RepositoryManager::new(pool.clone()));
        let view_model_service = ViewModelService::new(repository_manager.clone());

        let emulator_id = repository_manager
            .get_emulator_repository()
            .add_emulator("Test Emulator".to_string(), "temu".to_string(), false)
            .await
            .unwrap();

        let system_id = repository_manager
            .get_system_repository()
            .add_system("Test System".to_string())
            .await
            .unwrap();

        repository_manager
            .get_emulator_repository()
            .add_emulator_system(emulator_id, system_id, "args".to_string())
            .await
            .unwrap();

        let emulator_view_model = view_model_service
            .get_emulator_view_model(emulator_id)
            .await
            .unwrap();

        assert_eq!(emulator_view_model.id, emulator_id);
        assert_eq!(emulator_view_model.name, "Test Emulator");
        assert_eq!(emulator_view_model.executable, "temu");
        assert!(!emulator_view_model.extract_files);
        assert_eq!(emulator_view_model.systems.len(), 1);
        assert_eq!(emulator_view_model.systems[0].system_id, system_id);
        assert_eq!(emulator_view_model.systems[0].system_name, "Test System");
        assert_eq!(emulator_view_model.systems[0].arguments, "args");
    }

    #[async_std::test]
    async fn test_get_settings() {
        let pool = setup_test_db().await;
        let pool = Arc::new(pool);
        let repository_manager = Arc::new(RepositoryManager::new(pool.clone()));
        let view_model_service = ViewModelService::new(repository_manager.clone());

        repository_manager
            .settings()
            .add_setting(SettingName::CollectionRootDir.as_str(), "test_value")
            .await
            .unwrap();

        let settings = view_model_service.get_settings().await.unwrap();
        assert_eq!(settings.collection_root_dir, PathBuf::from("test_value"));
    }
}
