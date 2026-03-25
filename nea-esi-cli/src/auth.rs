use nea_esi::{EsiClient, EsiTokens};
use url::Url;

use crate::config::Config;
use crate::token_store;

/// Default read scopes for ESI.
pub const DEFAULT_SCOPES: &[&str] = &[
    "esi-skills.read_skills.v1",
    "esi-skills.read_skillqueue.v1",
    "esi-wallet.read_character_wallet.v1",
    "esi-assets.read_assets.v1",
    "esi-characters.read_contacts.v1",
    "esi-killmails.read_killmails.v1",
    "esi-mail.read_mail.v1",
    "esi-location.read_location.v1",
    "esi-location.read_ship_type.v1",
    "esi-location.read_online.v1",
    "esi-clones.read_clones.v1",
    "esi-clones.read_implants.v1",
    "esi-universe.read_structures.v1",
    "esi-fittings.read_fittings.v1",
    "esi-fleets.read_fleet.v1",
    "esi-characters.read_notifications.v1",
    "esi-industry.read_character_jobs.v1",
    "esi-contracts.read_character_contracts.v1",
    "esi-calendar.read_calendar_events.v1",
    "esi-characters.read_corporation_roles.v1",
    "esi-planets.manage_planets.v1",
    "esi-characters.read_standings.v1",
    "esi-characters.read_loyalty.v1",
    "esi-characters.read_fatigue.v1",
    "esi-characters.read_medals.v1",
    "esi-characters.read_titles.v1",
    "esi-search.search_structures.v1",
    "esi-corporations.read_corporation_membership.v1",
    "esi-industry.read_character_mining.v1",
];

/// Additional write scopes beyond `DEFAULT_SCOPES`.
const WRITE_SCOPES: &[&str] = &[
    "esi-mail.send_mail.v1",
    "esi-mail.organize_mail.v1",
    "esi-fittings.write_fittings.v1",
    "esi-characters.write_contacts.v1",
    "esi-fleets.write_fleet.v1",
    "esi-ui.open_window.v1",
    "esi-ui.write_waypoint.v1",
    "esi-calendar.respond_calendar_events.v1",
];

pub struct LoginOptions {
    pub scopes: Option<Vec<String>>,
    pub all_scopes: bool,
    pub headless: bool,
}

pub async fn login(
    client: &EsiClient,
    config: &Config,
    token_path: &std::path::Path,
    opts: LoginOptions,
) -> anyhow::Result<EsiTokens> {
    if config.app.client_id.is_none() {
        anyhow::bail!(
            "No client_id configured. Set one with:\n  \
             nea-esi-cli config set app.client_id YOUR_CLIENT_ID\n\n\
             Register an app at https://developers.eveonline.com/"
        );
    }

    // Determine scopes
    let scopes: Vec<&str> = if opts.all_scopes {
        DEFAULT_SCOPES
            .iter()
            .chain(WRITE_SCOPES.iter())
            .copied()
            .collect()
    } else if let Some(ref custom) = opts.scopes {
        custom.iter().map(String::as_str).collect()
    } else if !config.auth.scopes.is_empty() {
        config.auth.scopes.iter().map(String::as_str).collect()
    } else {
        DEFAULT_SCOPES.to_vec()
    };

    let headless = opts.headless || config.auth.headless;

    if headless {
        login_copy_paste(client, token_path, &scopes).await
    } else {
        match login_browser(client, token_path, &scopes).await {
            Ok(tokens) => Ok(tokens),
            Err(e) => {
                eprintln!("Browser login failed ({e}), falling back to copy-paste mode...");
                login_copy_paste(client, token_path, &scopes).await
            }
        }
    }
}

async fn login_browser(
    client: &EsiClient,
    token_path: &std::path::Path,
    scopes: &[&str],
) -> anyhow::Result<EsiTokens> {
    let server = tiny_http::Server::http("127.0.0.1:0")
        .map_err(|e| anyhow::anyhow!("Failed to start local server: {e}"))?;

    let port = server.server_addr().to_ip().unwrap().port();
    let redirect_uri = format!("http://localhost:{port}/callback");

    let challenge = client.authorize_url(&redirect_uri, scopes)?;

    eprintln!("Opening browser for EVE SSO login...");
    eprintln!("If it doesn't open, visit:\n  {}", challenge.authorize_url);

    if let Err(e) = open::that(&challenge.authorize_url) {
        eprintln!("Could not open browser: {e}");
    }

    // Wait for the callback (with timeout)
    let request = server
        .recv_timeout(std::time::Duration::from_secs(120))
        .map_err(|e| anyhow::anyhow!("Error waiting for callback: {e}"))?
        .ok_or_else(|| anyhow::anyhow!("Timed out waiting for SSO callback (120s)"))?;

    let request_url = format!("http://localhost{}", request.url());
    let parsed = Url::parse(&request_url)?;
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();

    let code = params
        .get("code")
        .ok_or_else(|| anyhow::anyhow!("No 'code' parameter in callback URL"))?;

    let state = params
        .get("state")
        .ok_or_else(|| anyhow::anyhow!("No 'state' parameter in callback URL"))?;

    if *state != challenge.state {
        anyhow::bail!("State mismatch — possible CSRF attack");
    }

    // Send success response to browser
    let response = tiny_http::Response::from_string(
        "<html><body><h2>Login successful!</h2><p>You can close this tab.</p></body></html>",
    )
    .with_header(
        "Content-Type: text/html"
            .parse::<tiny_http::Header>()
            .unwrap(),
    );
    let _ = request.respond(response);

    eprintln!("Exchanging authorization code...");
    let tokens = client
        .exchange_code(code, &challenge.code_verifier, &redirect_uri)
        .await?;

    token_store::save_tokens_at(&tokens, token_path)?;
    eprintln!(
        "Logged in successfully. Token expires at {}",
        tokens.expires_at
    );

    Ok(tokens)
}

async fn login_copy_paste(
    client: &EsiClient,
    token_path: &std::path::Path,
    scopes: &[&str],
) -> anyhow::Result<EsiTokens> {
    let redirect_uri = "https://localhost/callback";

    let challenge = client.authorize_url(redirect_uri, scopes)?;

    eprintln!("Open this URL in a browser to log in:\n");
    eprintln!("  {}\n", challenge.authorize_url);
    eprintln!("After authorizing, you will be redirected to a URL that won't load.");
    eprintln!("Copy the FULL URL from your browser's address bar and paste it here.\n");

    let callback_url: String = dialoguer::Input::new()
        .with_prompt("Paste callback URL")
        .interact_text()?;

    let parsed = Url::parse(callback_url.trim())?;
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();

    let code = params
        .get("code")
        .ok_or_else(|| anyhow::anyhow!("No 'code' parameter found in the URL you pasted"))?;

    if let Some(state) = params.get("state")
        && *state != challenge.state
    {
        anyhow::bail!("State mismatch — the callback URL doesn't match this login session");
    }

    eprintln!("Exchanging authorization code...");
    let tokens = client
        .exchange_code(code, &challenge.code_verifier, redirect_uri)
        .await?;

    token_store::save_tokens_at(&tokens, token_path)?;
    eprintln!(
        "Logged in successfully. Token expires at {}",
        tokens.expires_at
    );

    Ok(tokens)
}
