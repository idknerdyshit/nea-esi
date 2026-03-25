#[derive(clap::Subcommand)]
pub enum CalendarCommand {
    /// List upcoming calendar events
    List,
    /// Get calendar event details
    Event {
        /// Event ID
        event_id: i64,
    },
    /// Respond to a calendar event
    Respond {
        /// Event ID
        event_id: i64,
        /// Response: accepted, declined, tentative
        response: String,
    },
    /// Get event attendees
    Attendees {
        /// Event ID
        event_id: i64,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: CalendarCommand) -> anyhow::Result<()> {
    let cid = ctx
        .character_id
        .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        CalendarCommand::List => {
            let result = ctx.client.character_calendar(cid, None).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CalendarCommand::Event { event_id } => {
            let result = ctx.client.character_calendar_event(cid, event_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CalendarCommand::Respond { event_id, response } => {
            ctx.client
                .set_event_response(cid, event_id, &response)
                .await?;
            println!("Response set to '{response}' for event {event_id}.");
            Ok(())
        }
        CalendarCommand::Attendees { event_id } => {
            let result = ctx.client.event_attendees(cid, event_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
