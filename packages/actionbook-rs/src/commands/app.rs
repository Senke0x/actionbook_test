//! Electron desktop application automation commands.
//!
//! This module provides the `actionbook app` command for controlling Electron
//! desktop applications (VS Code, Slack, Discord, Notion, etc.) via Chrome DevTools Protocol.
//!
//! ## Command Delegation
//!
//! Most commands are delegated to browser.rs for maximum code reuse:
//! - App-specific: launch, attach, list, status, close, restart (implemented here)
//! - Shared commands: click, type, snapshot, etc. (delegated to browser module)

use colored::Colorize;

use crate::browser::{discover_electron_apps, SessionManager};
use crate::cli::{AppCommands, Cli};
use crate::config::Config;
use crate::error::{ActionbookError, Result};

/// Main entry point for app commands
pub async fn run(cli: &Cli, command: &AppCommands) -> Result<()> {
    let config = Config::load()?;

    match command {
        // App-specific lifecycle commands
        AppCommands::Launch { app_name } => launch(cli, &config, app_name).await,
        AppCommands::Attach { target } => attach(cli, &config, target).await,
        AppCommands::List => list(cli).await,
        AppCommands::Status => status(cli, &config).await,
        AppCommands::Close => close(cli, &config).await,
        AppCommands::Restart => restart(cli, &config).await,

        // Shared commands - delegate to browser module
        AppCommands::Goto { url, timeout } => {
            crate::commands::browser::goto(cli, &config, url, *timeout).await
        }
        AppCommands::Back => crate::commands::browser::back(cli, &config).await,
        AppCommands::Forward => crate::commands::browser::forward(cli, &config).await,
        AppCommands::Reload => crate::commands::browser::reload(cli, &config).await,
        AppCommands::Pages => crate::commands::browser::pages(cli, &config).await,
        AppCommands::Switch { page_id } => {
            crate::commands::browser::switch(cli, &config, page_id).await
        }
        AppCommands::Wait { selector, timeout } => {
            crate::commands::browser::wait(cli, &config, selector, *timeout).await
        }
        AppCommands::WaitNav { timeout } => {
            crate::commands::browser::wait_nav(cli, &config, *timeout).await
        }
        AppCommands::Click { selector, wait, ref_id, human } => {
            crate::commands::browser::click(
                cli,
                &config,
                selector.as_deref(),
                *wait,
                ref_id.as_deref(),
                *human,
            )
            .await
        }
        AppCommands::Type { selector, text, wait, ref_id, human } => {
            // Validate: text is required unless using --ref mode
            if text.is_none() && ref_id.is_none() {
                return Err(ActionbookError::InvalidArgument(
                    "Text is required. Use --ref <ID> to type into a snapshot reference, or provide text directly.".to_string()
                ));
            }
            crate::commands::browser::type_text(
                cli,
                &config,
                selector.as_deref(),
                text.as_deref().unwrap_or(""),
                *wait,
                ref_id.as_deref(),
                *human,
            )
            .await
        }
        AppCommands::Fill { selector, text, wait, ref_id } => {
            // Validate: text is required unless using --ref mode
            if text.is_none() && ref_id.is_none() {
                return Err(ActionbookError::InvalidArgument(
                    "Text is required. Use --ref <ID> to fill a snapshot reference, or provide text directly.".to_string()
                ));
            }
            crate::commands::browser::fill(
                cli,
                &config,
                selector.as_deref(),
                text.as_deref().unwrap_or(""),
                *wait,
                ref_id.as_deref(),
            )
            .await
        }
        AppCommands::Select { selector, value } => {
            crate::commands::browser::select(cli, &config, selector, value).await
        }
        AppCommands::Hover { selector } => {
            crate::commands::browser::hover(cli, &config, selector).await
        }
        AppCommands::Focus { selector } => {
            crate::commands::browser::focus(cli, &config, selector).await
        }
        AppCommands::Press { key } => {
            crate::commands::browser::press(cli, &config, key).await
        }
        AppCommands::Screenshot { path, full_page } => {
            crate::commands::browser::screenshot(cli, &config, path, *full_page).await
        }
        AppCommands::Pdf { path } => {
            crate::commands::browser::pdf(cli, &config, path).await
        }
        AppCommands::Eval { code } => {
            crate::commands::browser::eval(cli, &config, code).await
        }
        AppCommands::Html { selector } => {
            crate::commands::browser::html(cli, &config, selector.as_deref()).await
        }
        AppCommands::Text { selector, mode } => {
            crate::commands::browser::text(cli, &config, selector.as_deref(), mode).await
        }
        AppCommands::Snapshot {
            interactive,
            cursor,
            compact,
            depth,
            selector,
            format,
            diff,
            max_tokens,
        } => {
            crate::commands::browser::snapshot(
                cli,
                &config,
                *interactive,
                *cursor,
                *compact,
                format,
                *depth,
                selector.as_deref(),
                *diff,
                *max_tokens,
            )
            .await
        }
        AppCommands::Inspect { x, y, desc } => {
            crate::commands::browser::inspect(cli, &config, *x, *y, desc.as_deref()).await
        }
        AppCommands::Viewport => {
            crate::commands::browser::viewport(cli, &config).await
        }
        AppCommands::Cookies { command } => {
            crate::commands::browser::cookies(cli, &config, command).await
        }
        AppCommands::Scroll { direction, smooth } => {
            crate::commands::browser::scroll(cli, &config, direction, *smooth).await
        }
        AppCommands::Batch { file, delay } => {
            crate::commands::batch::run(cli, &config, file.as_deref(), *delay).await
        }
        AppCommands::Fingerprint { command } => {
            crate::commands::browser::fingerprint(cli, &config, command).await
        }
        AppCommands::Console { duration, level } => {
            crate::commands::browser::console_log(cli, &config, *duration, level).await
        }
        AppCommands::WaitIdle { timeout, idle_time } => {
            crate::commands::browser::wait_idle(cli, &config, *timeout, *idle_time).await
        }
        AppCommands::Info { selector } => {
            crate::commands::browser::info(cli, &config, selector).await
        }
        AppCommands::Storage { command } => {
            crate::commands::browser::storage(cli, &config, command).await
        }
        AppCommands::Emulate { device } => {
            crate::commands::browser::emulate(cli, &config, device).await
        }
        AppCommands::WaitFn { expression, timeout, interval } => {
            crate::commands::browser::wait_fn(cli, &config, expression, *timeout, *interval).await
        }
        AppCommands::Upload { files, selector, ref_id, wait } => {
            crate::commands::browser::upload(cli, &config, files, selector.as_deref(), ref_id.as_deref(), *wait).await
        }
        AppCommands::Tab { command } => {
            crate::commands::browser::tab_command(cli, &config, command).await
        }
    }
}

// ============================================================================
// App-specific implementations
// ============================================================================

/// Launch an Electron application by name
async fn launch(cli: &Cli, config: &Config, app_name: &str) -> Result<()> {
    // Discover installed apps
    let apps = discover_electron_apps();

    // Find matching app (case-insensitive)
    let app_name_lower = app_name.to_lowercase();
    let app = apps
        .iter()
        .find(|a| {
            a.name.to_lowercase().contains(&app_name_lower)
                || a.path
                    .to_str()
                    .map(|p| p.to_lowercase().contains(&app_name_lower))
                    .unwrap_or(false)
        })
        .ok_or_else(|| {
            ActionbookError::ConfigError(format!(
                "Application '{}' not found. Run 'actionbook app list' to see available apps.",
                app_name
            ))
        })?;

    println!("{} {}", "Launching".green(), app.name);
    println!("  Path: {}", app.path.display());

    // Use the same profile resolution logic as other commands
    let profile_name = crate::commands::browser::effective_profile_name(cli, config);

    // Launch the app with CDP debugging
    let session_manager = SessionManager::new(config.clone());

    // Convert PathBuf to string
    let app_path = app
        .path
        .to_str()
        .ok_or_else(|| ActionbookError::ConfigError("Invalid app path".to_string()))?;

    // Parse CDP port from CLI if provided
    let port = if let Some(cdp) = &cli.cdp {
        // Try to parse as port number
        cdp.parse::<u16>().ok()
    } else {
        None
    };

    let (_browser, _handler) = session_manager
        .launch_custom_app(profile_name, app_path, vec![], port)
        .await?;

    println!("{} Connected to {}", "✓".green(), app.name);
    println!("  Profile: {}", profile_name);
    println!("\n{}", "App is ready for automation.".bright_green());
    println!("\nUse 'actionbook app status' to check connection info.");

    Ok(())
}

/// Attach to a running application
async fn attach(cli: &Cli, config: &Config, target: &str) -> Result<()> {
    // Determine if target is a port number or WebSocket URL
    let endpoint = if target.parse::<u16>().is_ok() {
        // It's a port number - pass as-is (don't convert to HTTP URL)
        target.to_string()
    } else if target.starts_with("ws://") || target.starts_with("wss://") {
        // It's already a WebSocket URL
        target.to_string()
    } else if target.starts_with("http://") || target.starts_with("https://") {
        // It's an HTTP URL - extract port number
        let port = target
            .split("://")
            .nth(1)
            .and_then(|s| s.split(':').nth(1))
            .and_then(|s| s.split('/').next())
            .and_then(|s| s.parse::<u16>().ok())
            .ok_or_else(|| {
                ActionbookError::ConfigError(format!(
                    "Cannot extract port from HTTP URL: {}. Use port number (e.g., 9222) or WebSocket URL instead.",
                    target
                ))
            })?;
        port.to_string()
    } else {
        // Try to find app by name
        let apps = discover_electron_apps();
        let app_name_lower = target.to_lowercase();
        let app = apps
            .iter()
            .find(|a| a.name.to_lowercase().contains(&app_name_lower))
            .ok_or_else(|| {
                ActionbookError::ConfigError(format!(
                    "Could not find app '{}'. Use port number or WebSocket URL instead.",
                    target
                ))
            })?;

        println!(
            "{} Found app: {} at {}",
            "ℹ".blue(),
            app.name,
            app.path.display()
        );

        // Try to auto-detect CDP port (common ports: 9222-9225)
        println!("Scanning for active CDP ports...");
        for port in [9222, 9223, 9224, 9225] {
            if is_cdp_port_responding(port).await {
                println!("{} Detected CDP port: {}", "✓".green(), port);

                // Connect and save session with app path
                let profile_name = crate::commands::browser::effective_profile_name(cli, config);
                let (cdp_port, cdp_url) = crate::commands::browser::resolve_cdp_endpoint(&port.to_string()).await?;

                let session_manager = SessionManager::new(config.clone());
                let app_path_str = app.path.to_str().map(|s| s.to_string());
                session_manager.save_external_session_with_app(
                    profile_name,
                    cdp_port,
                    &cdp_url,
                    app_path_str,
                )?;

                if cli.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": true,
                            "app_name": app.name,
                            "app_path": app.path,
                            "profile": profile_name,
                            "cdp_port": cdp_port,
                            "cdp_url": cdp_url
                        })
                    );
                } else {
                    println!("{} Connected to {} at port {}", "✓".green(), app.name, cdp_port);
                    println!("  WebSocket URL: {}", cdp_url);
                    println!("  Profile: {}", profile_name);
                }

                return Ok(());
            }
        }

        return Err(ActionbookError::ConfigError(format!(
            "App '{}' found but no active CDP port detected (tried 9222-9225).\n\
             Please launch the app with --remote-debugging-port=<PORT> and use:\n  \
             actionbook app attach <PORT>",
            app.name
        )));
    };

    // Delegate to browser connect command (for port/URL endpoints)
    crate::commands::browser::connect(cli, config, &endpoint).await
}

/// Check if a CDP port is responding with valid CDP protocol
async fn is_cdp_port_responding(port: u16) -> bool {
    use std::time::Duration;

    let client = match reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(1))
        .build()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    // Check /json/version endpoint for CDP protocol
    let response = match client
        .get(format!("http://127.0.0.1:{}/json/version", port))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return false,
    };

    // Verify response is valid CDP JSON with webSocketDebuggerUrl field
    if let Ok(json) = response.json::<serde_json::Value>().await {
        json.get("webSocketDebuggerUrl").is_some()
            || json.get("Browser").is_some()
            || json.get("Protocol-Version").is_some()
    } else {
        false
    }
}

/// List all discoverable Electron applications
async fn list(_cli: &Cli) -> Result<()> {
    let apps = discover_electron_apps();

    if apps.is_empty() {
        println!("{}", "No Electron applications detected.".yellow());
        println!("\nTo control an app, it must be launched with:");
        println!("  --remote-debugging-port=9222");
        return Ok(());
    }

    println!("{}", "Detected Electron applications:".bright_green());
    println!();

    for (idx, app) in apps.iter().enumerate() {
        println!("{}. {}", idx + 1, app.name.bright_cyan());
        println!("   Path: {}", app.path.display().to_string().dimmed());
        if let Some(version) = &app.version {
            println!("   Version: {}", version.dimmed());
        }
        println!();
    }

    println!("{}", "To launch an app:".bright_white());
    println!("  actionbook app launch \"App Name\"");
    println!();
    println!("{}", "To attach to a running app:".bright_white());
    println!("  actionbook app attach <port>");

    Ok(())
}

/// Show application status
async fn status(cli: &Cli, config: &Config) -> Result<()> {
    // Delegate to browser status
    crate::commands::browser::status(cli, config).await
}

/// Close the connected application
async fn close(cli: &Cli, config: &Config) -> Result<()> {
    // Delegate to browser close
    crate::commands::browser::close(cli, config).await
}

/// Restart the connected application
async fn restart(cli: &Cli, config: &Config) -> Result<()> {
    use crate::browser::SessionManager;
    use std::fs;
    use std::path::PathBuf;

    let profile_name = crate::commands::browser::effective_profile_name(cli, config);

    // Load session state to check if it's a custom app
    // Use same path as SessionManager: ~/.actionbook/sessions
    let sessions_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".actionbook")
        .join("sessions");
    let session_file = sessions_dir.join(format!("{}.json", profile_name));

    let session_state_content = fs::read_to_string(&session_file).map_err(|_| {
        ActionbookError::BrowserNotRunning
    })?;

    let session_state: serde_json::Value = serde_json::from_str(&session_state_content)
        .map_err(|e| ActionbookError::ConfigError(format!("Failed to parse session state: {}", e)))?;

    // Check if this is a custom app session
    if let Some(app_path) = session_state.get("custom_app_path").and_then(|v| v.as_str()) {
        // This is a custom app - restart it properly
        println!("{} Restarting application: {}", "ℹ".blue(), app_path);

        // Close current session
        crate::commands::browser::close(cli, config).await?;

        // Get CDP port from old session
        let port = session_state.get("cdp_port").and_then(|v| v.as_u64()).map(|p| p as u16);

        // Relaunch the custom app
        let session_manager = SessionManager::new(config.clone());
        let (_browser, _handler) = session_manager
            .launch_custom_app(profile_name, app_path, vec![], port)
            .await?;

        println!("{} Application restarted", "✓".green());
        Ok(())
    } else {
        // This is a regular browser session - use browser restart
        crate::commands::browser::restart(cli, config).await
    }
}
