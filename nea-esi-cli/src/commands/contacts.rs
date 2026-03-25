#[derive(clap::Subcommand)]
pub enum ContactsCommand {
    /// List character contacts
    List,
    /// List character contact labels
    Labels,
    /// Add contacts
    Add {
        /// Contact IDs
        #[arg(required = true)]
        contact_ids: Vec<i64>,
        /// Standing value (-10.0 to 10.0)
        #[arg(long)]
        standing: f64,
        /// Label IDs
        #[arg(long, value_delimiter = ',')]
        label_ids: Vec<i64>,
        /// Watch contact
        #[arg(long)]
        watched: bool,
    },
    /// Edit contacts
    Edit {
        /// Contact IDs
        #[arg(required = true)]
        contact_ids: Vec<i64>,
        /// Standing value (-10.0 to 10.0)
        #[arg(long)]
        standing: f64,
        /// Label IDs
        #[arg(long, value_delimiter = ',')]
        label_ids: Vec<i64>,
        /// Watch contact
        #[arg(long)]
        watched: bool,
    },
    /// Delete contacts
    Delete {
        /// Contact IDs
        #[arg(required = true)]
        contact_ids: Vec<i64>,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: ContactsCommand) -> anyhow::Result<()> {
    let cid = ctx
        .character_id
        .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        ContactsCommand::List => {
            let result = ctx.client.character_contacts(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContactsCommand::Labels => {
            let result = ctx.client.character_contact_labels(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContactsCommand::Add {
            contact_ids,
            standing,
            label_ids,
            watched,
        } => {
            let labels = if label_ids.is_empty() {
                None
            } else {
                Some(label_ids.as_slice())
            };
            let watched = if watched { Some(true) } else { None };
            ctx.client
                .add_contacts(cid, standing, &contact_ids, labels, watched)
                .await?;
            println!("Contacts added.");
            Ok(())
        }
        ContactsCommand::Edit {
            contact_ids,
            standing,
            label_ids,
            watched,
        } => {
            let labels = if label_ids.is_empty() {
                None
            } else {
                Some(label_ids.as_slice())
            };
            let watched = if watched { Some(true) } else { None };
            ctx.client
                .edit_contacts(cid, standing, &contact_ids, labels, watched)
                .await?;
            println!("Contacts updated.");
            Ok(())
        }
        ContactsCommand::Delete { contact_ids } => {
            ctx.client.delete_contacts(cid, &contact_ids).await?;
            println!("Contacts deleted.");
            Ok(())
        }
    }
}
