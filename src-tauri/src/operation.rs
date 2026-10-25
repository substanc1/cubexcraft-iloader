use serde::Serialize;
use tauri::{Emitter, Window};

use crate::error::AppError;

pub struct Operation<'a> {
    id: String,
    window: &'a Window,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationUpdate<'a> {
    update_type: &'a str,
    step_id: &'a str,
    extra_details: Option<AppError>,
    // CubeXCraft: этап внутри шага (sign, pack, upload, upload_wifi, install) и процент — окно пишет, что идёт
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<u8>,
}

impl<'a> Operation<'a> {
    pub fn new(id: String, window: &'a Window) -> Operation<'a> {
        Operation { id, window }
    }

    pub fn move_on(&self, old_id: &str, new_id: &str) -> Result<(), AppError> {
        self.complete(old_id)?;
        self.start(new_id)
    }

    pub fn start(&self, id: &str) -> Result<(), AppError> {
        self.window
            .emit(
                &format!("operation_{}", self.id),
                OperationUpdate {
                    update_type: "started",
                    step_id: id,
                    extra_details: None,
                    phase: None,
                    progress: None,
                },
            )
            .map_err(|e| AppError::OperationUpdate(e.to_string()))
    }

    pub fn complete(&self, id: &str) -> Result<(), AppError> {
        self.window
            .emit(
                &format!("operation_{}", self.id),
                OperationUpdate {
                    update_type: "finished",
                    step_id: id,
                    extra_details: None,
                    phase: None,
                    progress: None,
                },
            )
            .map_err(|e| AppError::OperationUpdate(e.to_string()))
    }

    pub fn progress(&self, id: &str, phase: &str, progress: u8) {
        let _ = self.window.emit(
            &format!("operation_{}", self.id),
            OperationUpdate {
                update_type: "progress",
                step_id: id,
                extra_details: None,
                phase: Some(phase),
                progress: Some(progress.min(100)),
            },
        );
    }

    pub fn fail<T>(&self, id: &str, error: AppError) -> Result<T, AppError> {
        self.window
            .emit(
                &format!("operation_{}", self.id),
                OperationUpdate {
                    update_type: "failed",
                    step_id: id,
                    extra_details: Some(error.clone()),
                    phase: None,
                    progress: None,
                },
            )
            .map_err(|e| AppError::OperationUpdate(e.to_string()))?;
        Err(error)
    }

    pub fn fail_if_err<T>(&self, id: &str, res: Result<T, AppError>) -> Result<T, AppError> {
        match res {
            Ok(t) => Ok(t),
            Err(e) => self.fail::<T>(id, e),
        }
    }
}
