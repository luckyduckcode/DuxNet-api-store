use crate::core::data_structures::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use crate::wallet::Currency;

#[derive(Clone)]
pub struct TaskEngine {
    pub pending_tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    pub completed_tasks: Arc<RwLock<HashMap<TaskId, TaskResult>>>,
    pub processing_tasks: Arc<RwLock<HashMap<TaskId, String>>>, // task_id -> processor_did
    pub community_fund_manager: Option<Arc<crate::core::community_fund::CommunityFundManager>>,
}

impl TaskEngine {
    pub fn new() -> Self {
        TaskEngine {
            pending_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            processing_tasks: Arc::new(RwLock::new(HashMap::new())),
            community_fund_manager: None,
        }
    }

    pub fn with_community_fund_manager(mut self, manager: Arc<crate::core::community_fund::CommunityFundManager>) -> Self {
        self.community_fund_manager = Some(manager);
        self
    }

    pub async fn submit_task(&self, task: Task) -> Result<()> {
        let mut pending = self.pending_tasks.write().await;
        pending.insert(task.id.clone(), task.clone());
        info!("Submitted task: {}", task.id.0);
        Ok(())
    }

    pub async fn accept_task(&self, task_id: &TaskId, processor_did: String) -> Option<Task> {
        let mut pending = self.pending_tasks.write().await;
        if let Some(task) = pending.remove(task_id) {
            let mut processing = self.processing_tasks.write().await;
            processing.insert(task_id.clone(), processor_did.clone());
            info!("Task {} accepted by {}", task_id.0, processor_did);
            Some(task)
        } else {
            None
        }
    }

    pub async fn complete_task(&self, result: TaskResult) -> Result<()> {
        let mut completed = self.completed_tasks.write().await;
        let mut processing = self.processing_tasks.write().await;
        
        completed.insert(result.task_id.clone(), result.clone());
        processing.remove(&result.task_id);
        
        info!("Task {} completed by {}", result.task_id.0, result.processor_did);
        Ok(())
    }

    pub async fn process_task(&self, task: Task, processor_did: String) -> Result<TaskResult> {
        // 1. Deserialize payload (assume JSON for this example)
        let params: serde_json::Value = serde_json::from_slice(&task.payload)?;

        // 2. Route to the correct service handler
        let result_data = match task.service_id.0.as_str() {
            "text-processing" => process_text(params).await?,
            "image-analysis" => process_image(params).await?,
            "data-computation" => process_data(params).await?,
            "ml-training" => process_ml(params).await?,
            _ => return Err(anyhow::anyhow!("Unknown service")),
        };

        // 3. Sign/hash the result
        let proof = sign_result(&result_data);
        
        Ok(TaskResult {
            task_id: task.id,
            processor_did,
            result: serde_json::to_vec(&result_data)?,
            proof,
            completed_at: get_current_timestamp(),
        })
    }

    pub async fn process_pending_tasks(&self) -> Result<()> {
        // Get all pending tasks
        let pending_tasks = {
            let pending = self.pending_tasks.read().await;
            pending.clone()
        };

        // Process each pending task (simplified - just log for now)
        for (task_id, task) in pending_tasks {
            info!("Processing pending task: {}", task_id.0);
            // In a real implementation, this would dispatch tasks to available processors
        }

        Ok(())
    }
}

// --- Service handler scaffolds ---
async fn process_text(params: serde_json::Value) -> Result<serde_json::Value> {
    // TODO: Implement real text processing (e.g., sentiment analysis)
    Ok(serde_json::json!({"result": "text processed", "params": params}))
}

async fn process_image(params: serde_json::Value) -> Result<serde_json::Value> {
    // TODO: Implement real image analysis
    Ok(serde_json::json!({"result": "image analyzed", "params": params}))
}

async fn process_data(params: serde_json::Value) -> Result<serde_json::Value> {
    // TODO: Implement real data computation
    Ok(serde_json::json!({"result": "data computed", "params": params}))
}

async fn process_ml(params: serde_json::Value) -> Result<serde_json::Value> {
    // TODO: Implement real ML model training/inference
    Ok(serde_json::json!({"result": "ml processed", "params": params}))
}

fn sign_result(result: &serde_json::Value) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_vec(result).unwrap_or_default());
    hasher.finalize().to_vec()
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    NotFound,
}

#[derive(Debug, Clone)]
pub struct TaskStats {
    pub pending_count: usize,
    pub processing_count: usize,
    pub completed_count: usize,
    pub total_tasks: usize,
} 