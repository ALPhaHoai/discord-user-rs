//! Stage instance operations for DiscordUser

use crate::{context::DiscordContext, error::Result, route::Route, types::*};

impl<T: DiscordContext + Send + Sync> StageOps for T {}

/// Extension trait providing stage instance CRUD operations.
///
/// Stage instances represent live audio sessions within Stage channels.
#[allow(async_fn_in_trait)]
pub trait StageOps: DiscordContext {
    /// Get the active stage instance for a Stage channel.
    ///
    /// Returns `404` if there is no active stage instance in the channel.
    async fn get_stage_instance(&self, channel_id: &ChannelId) -> Result<StageInstance> {
        self.http().get(Route::GetStageInstance { channel_id: channel_id.get() }).await
    }

    /// Create a stage instance.
    ///
    /// The channel must be a Stage channel (type 13). Only one active stage
    /// instance is allowed per channel at a time.
    async fn create_stage_instance(&self, req: CreateStageInstanceRequest) -> Result<StageInstance> {
        self.http().post(Route::CreateStageInstance, req).await
    }

    /// Edit the topic or privacy level of an active stage instance.
    async fn edit_stage_instance(&self, channel_id: &ChannelId, req: EditStageInstanceRequest) -> Result<StageInstance> {
        self.http().patch(Route::EditStageInstance { channel_id: channel_id.get() }, req).await
    }

    /// Delete (end) a stage instance.
    async fn delete_stage_instance(&self, channel_id: &ChannelId) -> Result<()> {
        self.http().delete(Route::DeleteStageInstance { channel_id: channel_id.get() }).await
    }
}
