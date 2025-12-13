//! Authentication handlers for WebSocket connections.
//!
//! Handles login, registration, and token-based authentication.

use axum::extract::ws::{Message, WebSocket};
use rand::Rng;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::api::ws::AppState;
use crate::domain::error::UserError;
use crate::domain::models::{User, PRICE_SCALE};
use crate::presentation::websocket::messages::{CompanyInfo, ServerMessage};

use super::helpers::calculate_net_worth;
use super::send_message;

/// Handle token-based authentication (for reconnection)
///
/// Uses secure cryptographic tokens instead of plain user IDs.
/// Tokens are validated against the TokenService which tracks active sessions.
pub async fn handle_auth(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    token: &str,
) {
    // Don't log the full token for security
    let token_preview = if token.len() > 8 {
        format!("{}...", &token[..8])
    } else {
        token.to_string()
    };
    debug!("Auth attempt with token: {}", token_preview);

    // Validate the secure token
    match state.tokens.validate_token(token) {
        Some(uid) => {
            match state.user_repo.find_by_id(uid).await {
                Ok(Some(user)) => {
                    if user.banned {
                        warn!("Auth failed: user {} is banned", uid);
                        // Revoke token for banned user
                        state.tokens.revoke_token(token);
                        let msg = ServerMessage::AuthFailed {
                            reason: "Account has been banned".to_string(),
                        };
                        send_message(sender, &msg).await;
                        return;
                    }

                    // Create session (kicks old sessions if max reached)
                    let (sid, kicked) = state.sessions.create_session(uid);
                    *session_id = Some(sid);
                    *user_id = Some(uid);

                    if !kicked.is_empty() {
                        info!(
                            "User {} authenticated via token, kicked {} old session(s)",
                            uid,
                            kicked.len()
                        );
                    } else {
                        info!("User {} authenticated via token with session {}", uid, sid);
                    }

                    // Token already validated - no need to create new one
                    // (the token they used is still valid)
                    let auth_msg = ServerMessage::AuthSuccess {
                        user_id: uid,
                        name: user.name.clone(),
                        role: user.role.to_string(),
                        token: None, // Don't issue new token on reconnection
                    };
                    send_message(sender, &auth_msg).await;

                    // Send post-auth data
                    send_post_auth_data(sender, state, &user).await;
                }
                Ok(None) => {
                    warn!("Auth failed: user {} from token not found in database", uid);
                    // Revoke orphaned token
                    state.tokens.revoke_token(token);
                    let msg = ServerMessage::AuthFailed {
                        reason: "User not found".to_string(),
                    };
                    send_message(sender, &msg).await;
                }
                Err(e) => {
                    error!("Auth error for uid {}: {}", uid, e);
                    let msg = ServerMessage::AuthFailed {
                        reason: format!("Auth error: {}", e),
                    };
                    send_message(sender, &msg).await;
                }
            }
        }
        None => {
            warn!("Auth failed: invalid or expired token");
            let msg = ServerMessage::AuthFailed {
                reason: "Invalid or expired token".to_string(),
            };
            send_message(sender, &msg).await;
        }
    }
}

/// Handle login with registration number and password
pub async fn handle_login(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    regno: String,
    password: String,
) {
    debug!("Login attempt for regno: {}", regno);

    match state.user_repo.find_by_regno(&regno).await {
        Ok(Some(user)) => {
            // Verify password (simple comparison - in production use proper hashing)
            if user.password_hash != password {
                warn!("Login failed for {}: invalid password", regno);
                let msg = ServerMessage::AuthFailed {
                    reason: "Invalid password".to_string(),
                };
                send_message(sender, &msg).await;
                return;
            }

            if user.banned {
                warn!("Login failed for {}: account banned", regno);
                let msg = ServerMessage::AuthFailed {
                    reason: "Account has been banned".to_string(),
                };
                send_message(sender, &msg).await;
                return;
            }

            // Create session (kicks old sessions if max reached)
            let (sid, kicked) = state.sessions.create_session(user.id);
            *session_id = Some(sid);
            *user_id = Some(user.id);

            if !kicked.is_empty() {
                info!(
                    "User {} (regno={}) logged in, kicked {} old session(s)",
                    user.name,
                    regno,
                    kicked.len()
                );
            } else {
                info!(
                    "User {} (regno={}) logged in with session {}",
                    user.name, regno, sid
                );
            }

            // Log login event
            state.event_log.log_user_login(user.id, &regno, &user.name);

            // Create secure auth token for reconnection
            let (token, _revoked) = state.tokens.create_token(user.id);

            let auth_msg = ServerMessage::AuthSuccess {
                user_id: user.id,
                name: user.name.clone(),
                role: user.role.to_string(),
                token: Some(token),
            };
            send_message(sender, &auth_msg).await;

            // Send post-auth data
            send_post_auth_data(sender, state, &user).await;
        }
        Ok(None) => {
            warn!("Login failed: regno {} not found", regno);
            let msg = ServerMessage::AuthFailed {
                reason: "User not found. Please register first.".to_string(),
            };
            send_message(sender, &msg).await;
        }
        Err(e) => {
            error!("Login error for regno {}: {}", regno, e);
            let msg = ServerMessage::AuthFailed {
                reason: format!("Login error: {}", e),
            };
            send_message(sender, &msg).await;
        }
    }
}

/// Handle new user registration
pub async fn handle_register(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user_id: &mut Option<u64>,
    session_id: &mut Option<u64>,
    regno: String,
    name: String,
    password: String,
) {
    debug!("Registration attempt for regno: {}, name: {}", regno, name);

    // Check if registration is allowed for this regno
    if let Err(reason) = state.config.is_regno_allowed(&regno) {
        warn!("Registration rejected for regno {}: {}", regno, reason);
        let msg = ServerMessage::RegisterFailed { reason };
        send_message(sender, &msg).await;
        return;
    }

    // Check if regno already exists (using the repository helper method)
    match state.user_repo.regno_exists(&regno).await {
        Ok(true) => {
            warn!("Registration failed: regno {} already exists", regno);
            let err = UserError::RegnoExists {
                regno: regno.clone(),
            };
            let msg = ServerMessage::RegisterFailed {
                reason: err.to_string(),
            };
            send_message(sender, &msg).await;
            return;
        }
        Err(e) => {
            error!("Registration error for regno {}: {}", regno, e);
            let msg = ServerMessage::RegisterFailed {
                reason: format!("Registration error: {}", e),
            };
            send_message(sender, &msg).await;
            return;
        }
        Ok(false) => {} // Regno doesn't exist - proceed with registration
    }

    // Get companies for initial share allocation
    let companies = match state.company_repo.all().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to fetch companies for registration: {}", e);
            let msg = ServerMessage::RegisterFailed {
                reason: "Failed to initialize portfolio".to_string(),
            };
            send_message(sender, &msg).await;
            return;
        }
    };

    // Create new user with configured starting money
    let mut new_user = User::new(regno.clone(), name.clone(), password);
    let total_starting_value = state.config.default_starting_money();

    // Allocate ~50% as shares, ~50% as cash
    let base_price: i64 = 100 * PRICE_SCALE;
    let target_portfolio_value = total_starting_value / 2;

    let num_companies = companies.len() as i64;
    if num_companies > 0 {
        let value_per_company = target_portfolio_value / num_companies;
        let shares_per_company = (value_per_company / base_price) as u64;

        let mut rng = rand::thread_rng();
        let mut total_portfolio_value: i64 = 0;

        for company in &companies {
            // Random variance between -20% and +20%
            let variance: i64 = rng.gen_range(-20..=20);
            let adjusted_shares = ((shares_per_company as i64 * (100 + variance)) / 100) as u64;
            let final_shares = adjusted_shares.max(1);

            let share_value = (final_shares as i64) * base_price;
            total_portfolio_value += share_value;

            new_user.portfolio.push(crate::domain::models::Portfolio {
                user_id: new_user.id,
                symbol: company.symbol.clone(),
                qty: final_shares,
                short_qty: 0,
                locked_qty: 0,
                average_buy_price: base_price,
            });

            debug!(
                "  {} allocated {} shares = ${}",
                company.symbol,
                final_shares,
                share_value / PRICE_SCALE
            );
        }

        new_user.money = (total_starting_value - total_portfolio_value).max(0);

        let actual_networth = new_user.money + total_portfolio_value;
        info!(
            "New trader {} allocated: cash=${}, portfolio=${}, networth=${}",
            name,
            new_user.money / PRICE_SCALE,
            total_portfolio_value / PRICE_SCALE,
            actual_networth / PRICE_SCALE
        );
    } else {
        new_user.money = total_starting_value;
    }

    let new_user_id = new_user.id;

    match state.user_repo.save(new_user.clone()).await {
        Ok(_) => {
            // Create session
            let (sid, _) = state.sessions.create_session(new_user_id);
            *session_id = Some(sid);
            *user_id = Some(new_user_id);

            info!(
                "New user registered: {} (regno={}, id={}, session={})",
                name, regno, new_user_id, sid
            );

            // Log registration event
            let portfolio_value = new_user
                .portfolio
                .iter()
                .map(|p| (p.qty as i64) * p.average_buy_price)
                .sum::<i64>();
            state.event_log.log_user_registered(
                new_user_id,
                &regno,
                &name,
                new_user.money,
                portfolio_value,
            );

            // Create secure auth token for the new user
            let (token, _) = state.tokens.create_token(new_user_id);

            let msg = ServerMessage::RegisterSuccess {
                user_id: new_user_id,
                name: name.clone(),
                role: new_user.role.to_string(),
                token,
            };
            send_message(sender, &msg).await;

            // Send post-auth data
            send_post_auth_data(sender, state, &new_user).await;

            // Broadcast welcome message
            let portfolio_value_display = new_user
                .portfolio
                .iter()
                .map(|p| (p.qty as i64) * p.average_buy_price)
                .sum::<i64>()
                / PRICE_SCALE;
            let starting_cash = new_user.money / PRICE_SCALE;
            let system_msg = ServerMessage::System {
                message: format!(
                    "Welcome {}! You start with ${} in cash and ${} in stocks.",
                    name, starting_cash, portfolio_value_display
                ),
            };
            send_message(sender, &system_msg).await;
        }
        Err(e) => {
            error!("Failed to save user {}: {}", regno, e);
            let msg = ServerMessage::RegisterFailed {
                reason: format!("Failed to save user: {}", e),
            };
            send_message(sender, &msg).await;
        }
    }
}

/// Send common post-authentication data (companies, portfolio, market status)
async fn send_post_auth_data(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    state: &Arc<AppState>,
    user: &User,
) {
    // Send company list
    if let Ok(companies) = state.company_repo.all().await {
        let company_list: Vec<CompanyInfo> = companies
            .iter()
            .map(|c| CompanyInfo {
                id: c.id,
                symbol: c.symbol.clone(),
                name: c.name.clone(),
                sector: c.sector.clone(),
                volatility: c.volatility,
            })
            .collect();
        let companies_msg = ServerMessage::CompanyList {
            companies: company_list,
        };
        send_message(sender, &companies_msg).await;
    }

    // Send initial portfolio
    let net_worth = calculate_net_worth(user, &state.market);
    let portfolio_msg = ServerMessage::PortfolioUpdate {
        money: user.money,
        locked: user.locked_money,
        margin_locked: user.margin_locked,
        net_worth,
        items: user.portfolio.clone(),
    };
    send_message(sender, &portfolio_msg).await;

    // Send market status
    let status_msg = ServerMessage::MarketStatus {
        is_open: state.engine.is_market_open(),
    };
    send_message(sender, &status_msg).await;
}
