//! Authentication extractors for dashboard and machine-token auth.
//!
//! Two auth paths:
//! - `DashboardAuth`: MVP stub that always returns the default user.
//!   TODO: Swap for JWT-based authentication.
//! - `MachineTokenAuth`: Validates `Authorization: Bearer <token>` against the
//!   `machine_tokens` table and resolves the associated user context.

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::ErrorBody;
use crate::AppState;

/// Authenticated identity context shared by both auth paths.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub org_id: String,
    pub team_id: String,
}

// ---------------------------------------------------------------------------
// Dashboard auth — MVP: unconditional default user
// ---------------------------------------------------------------------------

/// Extractor for dashboard endpoints (GET /machines, approve, revoke).
///
/// MVP implementation: always resolves to the default user context.
/// TODO: Replace with JWT validation (read from cookie or Authorization header).
#[derive(Debug, Clone)]
pub struct DashboardAuth(pub AuthContext);

impl<S> FromRequestParts<S> for DashboardAuth
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // MVP: unconditionally return the seeded default user.
        // Swap point: validate JWT token here and resolve user from claims.
        Ok(Self(AuthContext {
            user_id: "user-default".to_string(),
            org_id: "org-default".to_string(),
            team_id: "team-default".to_string(),
        }))
    }
}

// ---------------------------------------------------------------------------
// Machine token auth — Bearer token lookup
// ---------------------------------------------------------------------------

/// Extractor for machine endpoints (register, heartbeat).
///
/// Reads `Authorization: Bearer <token>` and resolves the token to an
/// `AuthContext` via the `machine_tokens` + `users` tables.
#[derive(Debug, Clone)]
pub struct MachineTokenAuth(pub AuthContext);

impl FromRequestParts<AppState> for MachineTokenAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(parts)?;
        resolve_token_context(state, &token)
    }
}

/// Extract the bearer token from the Authorization header.
#[allow(clippy::result_large_err)]
fn extract_bearer_token(parts: &Parts) -> Result<String, Response> {
    let header = parts
        .headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "missing Authorization header".to_string(),
                }),
            )
                .into_response()
        })?;

    let token = header.strip_prefix("Bearer ").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "invalid Authorization header format, expected 'Bearer <token>'".to_string(),
            }),
        )
            .into_response()
    })?;

    Ok(token.to_string())
}

/// Look up the token in the database and resolve the full auth context.
#[allow(clippy::result_large_err)]
fn resolve_token_context(state: &AppState, token: &str) -> Result<MachineTokenAuth, Response> {
    let row = state
        .db
        .lock()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "internal server error".to_string(),
                }),
            )
                .into_response()
        })?
        .query_row(
            "SELECT u.id, u.org_id, u.team_id
             FROM machine_tokens mt
             JOIN users u ON u.id = mt.user_id
             WHERE mt.token = ?1",
            [token],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        );

    match row {
        Ok((user_id, org_id, team_id)) => Ok(MachineTokenAuth(AuthContext {
            user_id,
            org_id,
            team_id,
        })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorBody {
                error: "invalid or expired token".to_string(),
            }),
        )
            .into_response()),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorBody {
                error: "internal server error".to_string(),
            }),
        )
            .into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_context_clone() {
        let ctx = AuthContext {
            user_id: "u1".into(),
            org_id: "o1".into(),
            team_id: "t1".into(),
        };
        let cloned = ctx.clone();
        assert_eq!(cloned.user_id, "u1");
        assert_eq!(cloned.org_id, "o1");
        assert_eq!(cloned.team_id, "t1");
    }
}
