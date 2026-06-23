pub const MONITORS_CHANGED: &str = "events.monitors.changed";
pub const MONITORS_EVENTS: &str = "events.monitors.>";

pub async fn publish_monitors_changed(nats: &async_nats::Client) -> anyhow::Result<()> {
    nats.publish(MONITORS_CHANGED, "".into()).await?;
    Ok(())
}
