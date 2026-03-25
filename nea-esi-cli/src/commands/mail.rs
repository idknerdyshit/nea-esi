#[derive(clap::Subcommand)]
pub enum MailCommand {
    /// List mail headers
    List {
        /// Fetch mail before this mail ID
        #[arg(long)]
        before: Option<i64>,
    },
    /// Read a specific mail
    Read {
        /// Mail ID
        mail_id: i64,
    },
    /// List mail labels
    Labels,
    /// Delete a mail label
    DeleteLabel {
        /// Label ID
        label_id: i32,
    },
    /// Delete a mail
    Delete {
        /// Mail ID
        mail_id: i64,
    },
    /// List mailing lists
    MailingLists,
}

pub async fn execute(ctx: &super::ExecContext, cmd: MailCommand) -> anyhow::Result<()> {
    let cid = ctx
        .character_id
        .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        MailCommand::List { before } => {
            let result = match before {
                Some(before_id) => ctx.client.character_mail_before(cid, before_id).await?,
                None => ctx.client.character_mail(cid, None).await?,
            };
            crate::output::print_list(&result, ctx.format)
        }
        MailCommand::Read { mail_id } => {
            let result = ctx.client.character_mail_body(cid, mail_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        MailCommand::Labels => {
            let result = ctx.client.character_mail_labels(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        MailCommand::DeleteLabel { label_id } => {
            ctx.client.delete_mail_label(cid, label_id).await?;
            println!("Label {label_id} deleted.");
            Ok(())
        }
        MailCommand::Delete { mail_id } => {
            ctx.client.delete_mail(cid, mail_id).await?;
            println!("Mail {mail_id} deleted.");
            Ok(())
        }
        MailCommand::MailingLists => {
            let result = ctx.client.character_mailing_lists(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
