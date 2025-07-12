use crate::core::data_structures::*;
use crate::core::identity::DIDManager;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

pub struct MessagingSystem {
    messages: Arc<RwLock<HashMap<String, Message>>>,
    conversations: Arc<RwLock<HashMap<String, Conversation>>>,
    did_manager: DIDManager,
    message_handlers: Arc<RwLock<Vec<Box<dyn MessageHandler + Send + Sync>>>>,
}

#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: &Message) -> Result<()>;
}

impl MessagingSystem {
    pub fn new(did_manager: DIDManager) -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            did_manager,
            message_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn send_message(&self, request: MessageRequest) -> Result<MessageResponse> {
        let message_id = Uuid::new_v4().to_string();
        let timestamp = get_current_timestamp();
        
        // Create message content for signing
        let content_to_sign = format!("{}:{}:{}:{}:{}", 
            message_id, 
            self.did_manager.did.id, 
            request.to_did, 
            request.content, 
            timestamp
        );
        
        // Sign the message
        let signature = self.did_manager.sign_message(content_to_sign.as_bytes());
        
        let message = Message {
            id: message_id.clone(),
            from_did: self.did_manager.did.id.clone(),
            to_did: request.to_did.clone(),
            content: request.content,
            message_type: request.message_type,
            timestamp,
            signature,
            is_read: false,
            reply_to: request.reply_to,
        };
        
        // Store the message
        {
            let mut messages = self.messages.write().await;
            messages.insert(message_id.clone(), message.clone());
        }
        
        // Update conversation
        self.update_conversation(&message).await?;
        
        // Notify handlers
        self.notify_handlers(&message).await?;
        
        info!("Sent message {} to {}", message_id, request.to_did);
        
        Ok(MessageResponse {
            message_id,
            success: true,
            message: "Message sent successfully".to_string(),
        })
    }

    pub async fn receive_message(&self, message: Message) -> Result<()> {
        // Verify signature
        let content_to_verify = format!("{}:{}:{}:{}:{}", 
            message.id, 
            message.from_did, 
            message.to_did, 
            message.content, 
            message.timestamp
        );
        
        // For now, we'll accept all messages (in production, verify the signature)
        debug!("Received message {} from {}", message.id, message.from_did);
        
        // Store the message
        {
            let mut messages = self.messages.write().await;
            messages.insert(message.id.clone(), message.clone());
        }
        
        // Update conversation
        self.update_conversation(&message).await?;
        
        // Notify handlers
        self.notify_handlers(&message).await?;
        
        Ok(())
    }

    pub async fn get_messages(&self, peer_did: &str) -> Vec<Message> {
        let messages = self.messages.read().await;
        let mut conversation_messages = Vec::new();
        
        for message in messages.values() {
            if (message.from_did == self.did_manager.did.id && message.to_did == peer_did) ||
               (message.from_did == peer_did && message.to_did == self.did_manager.did.id) {
                conversation_messages.push(message.clone());
            }
        }
        
        // Sort by timestamp
        conversation_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        conversation_messages
    }

    pub async fn get_conversations(&self) -> Vec<Conversation> {
        let conversations = self.conversations.read().await;
        conversations.values().cloned().collect()
    }

    pub async fn mark_message_read(&self, message_id: &str) -> Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(message) = messages.get_mut(message_id) {
            message.is_read = true;
            debug!("Marked message {} as read", message_id);
        }
        Ok(())
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(message) = messages.remove(message_id) {
            // Update conversation
            self.recalculate_conversation(&message.from_did).await?;
            debug!("Deleted message {}", message_id);
        }
        Ok(())
    }

    async fn update_conversation(&self, message: &Message) -> Result<()> {
        let peer_did = if message.from_did == self.did_manager.did.id {
            &message.to_did
        } else {
            &message.from_did
        };
        
        let mut conversations = self.conversations.write().await;
        let conversation = conversations.entry(peer_did.clone()).or_insert_with(|| Conversation {
            peer_did: peer_did.clone(),
            last_message: None,
            unread_count: 0,
            message_count: 0,
        });
        
        conversation.last_message = Some(message.clone());
        conversation.message_count += 1;
        
        // Update unread count if message is from peer
        if message.from_did != self.did_manager.did.id && !message.is_read {
            conversation.unread_count += 1;
        }
        
        Ok(())
    }

    async fn recalculate_conversation(&self, peer_did: &str) -> Result<()> {
        let messages = self.get_messages(peer_did).await;
        let mut conversations = self.conversations.write().await;
        
        let conversation = conversations.entry(peer_did.to_string()).or_insert_with(|| Conversation {
            peer_did: peer_did.to_string(),
            last_message: None,
            unread_count: 0,
            message_count: 0,
        });
        
        conversation.message_count = messages.len();
        conversation.unread_count = messages.iter()
            .filter(|m| m.from_did == peer_did && !m.is_read)
            .count();
        
        conversation.last_message = messages.last().cloned();
        
        Ok(())
    }

    async fn notify_handlers(&self, message: &Message) -> Result<()> {
        let handlers = self.message_handlers.read().await;
        for handler in handlers.iter() {
            if let Err(e) = handler.handle_message(message).await {
                warn!("Message handler error: {}", e);
            }
        }
        Ok(())
    }

    pub async fn add_message_handler(&self, handler: Box<dyn MessageHandler + Send + Sync>) {
        let mut handlers = self.message_handlers.write().await;
        handlers.push(handler);
    }

    pub async fn get_message_stats(&self) -> MessageStats {
        let messages = self.messages.read().await;
        let conversations = self.conversations.read().await;
        
        let total_messages = messages.len();
        let total_conversations = conversations.len();
        let unread_messages = conversations.values()
            .map(|c| c.unread_count)
            .sum();
        
        MessageStats {
            total_messages,
            total_conversations,
            unread_messages,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub total_messages: usize,
    pub total_conversations: usize,
    pub unread_messages: usize,
} 