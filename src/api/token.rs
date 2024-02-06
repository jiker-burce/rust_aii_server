/**
 * login
 *   直接查询数据库, 判断是否存在
 *   不存在则返回无该用户
 *   存在则返回token
 */
use crate::api::tags::ApiTags;
use crate::api::user::UserInfo;

use poem::{error::InternalServerError, http::StatusCode, Error, Request, Result};
use poem_openapi::{payload::Json, types::Example, ApiResponse, Object, OpenApi, Union};
use serde::{Deserialize, Serialize};

#[derive(Debug, Object)]
struct LoginCredentialPassword {
    /// Email
    email: String,

    /// Password
    password: String,
}

/// Login credential
#[derive(Debug, Union)]
#[oai(discriminator_name = "type")]
enum LoginCredential {
    #[oai(mapping = "password")]
    Password(LoginCredentialPassword),
}

fn default_device() -> String {
    "unknown".to_string()
}

/// Login request
#[derive(Debug, Object)]
#[oai(example)]
struct LoginRequest {
    /// Credential
    credential: LoginCredential,

    /// Device id
    #[oai(default = "default_device")]
    device: String,

    /// FCM device token
    device_token: Option<String>,
}

/// Example for LoginRequest
/// 生成swagger时,会展示此示例
impl Example for LoginRequest {
    fn example() -> Self {
        LoginRequest {
            credential: LoginCredential::Password(LoginCredentialPassword {
                email: "admin@bruce-gu.com".to_string(),
                password: "123456".to_string(),
            }),
            device: "web".to_string(),
            device_token: None,
        }
    }
}

/// Token response
#[derive(Debug, Object)]
pub struct LoginResponse {
    /// Access token
    token: String,
    /// Refresh token
    refresh_token: String,
    // The access token expired in seconds
    // expired_in: i64,
    /// User info
    user: UserInfo,
}

#[derive(Object)]
pub struct ErrorMessage {
    code: i32,
    reason: String,
}

#[derive(ApiResponse)]
pub enum LoginApiResponse {
    /// Login success
    #[oai(status = 200)]
    Ok(Json<LoginResponse>),
    /// Login method does not supported
    #[oai(status = 403)]
    LoginMethodNotSupported,
    /// Invalid account or password
    #[oai(status = 401)]
    InvalidAccount(Json<ErrorMessage>),
    /// User does not exists
    #[oai(status = 404)]
    UserDoesNotExist,
    /// User has been frozen
    #[oai(status = 423)]
    Frozen,
    /// Email collision
    #[oai(status = 409)]
    EmailConflict,
    /// Account not associated
    #[oai(status = 410)]
    AccountNotAssociated,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct CurrentUser {
    pub uid: u64,
    pub device: String,
}

pub struct ApiToken;

/**
 * TODO:
 * 1, 从数据库去取当前用户 DONE
 * 2, 判断用户是否存在 DONE, 不存在给出异常提示及状态码 DONE
 * 3, 存在, 校验密码 DONE, 密码错误返回错误提示及状态码 DONE
 * 4, 密码校验成功, 则返回用户token DONE 和对应用户信息(不含敏感信息) DONE
 */

#[OpenApi(prefix_path = "/token", tag = "ApiTags::Token")]
impl ApiToken {
    #[oai(path = "/login", method = "post")]
    async fn login(&self, req: Json<LoginRequest>, request: &Request) -> Result<LoginApiResponse> {
        let cu = request.extensions().get::<CurrentUser>();
        println!("current user: {:?}", cu);
        self.do_login(req).await
    }

    async fn do_login(&self, req: Json<LoginRequest>) -> Result<LoginApiResponse> {
        let db = rc_database::Database::new()
            .await
            .expect("Database connection expected");

        // 因为使用 enum, 不能直接访问 req.credential.Password.email
        // 需要通过模式匹配的方式访问数据
        let (email, password) = match &req.credential {
            LoginCredential::Password(lcp) => (lcp.email.to_owned(), lcp.password.to_owned()),
        };

        let user: UserInfo = sqlx::query_as("SELECT * FROM users where email = ?")
            .bind(email)
            .fetch_one(db.get_pool())
            .await
            .map_err(|e| {
                println!("{:?}", e);
                LoginApiResponse::UserDoesNotExist
            })?;

        if !user.check_pw(&password) {
            return Ok(LoginApiResponse::InvalidAccount(Json(ErrorMessage {
                code: -1,
                reason: "密码不正确,请重新输入".to_string(),
            })));
        }

        let secret_key = dotenvy::var("SECRET_KEY").map_err(InternalServerError)?;
        let token_expiry_seconds = dotenvy::var("TOKEN_EXPIRY_SECONDS")
            .map_err(InternalServerError)?
            .parse::<i64>()
            .unwrap();
        let refresh_token_expiry_seconds = dotenvy::var("REFRESH_TOKEN_EXPIRY_SECONDS")
            .map_err(InternalServerError)?
            .parse::<i64>()
            .unwrap();

        let (refresh_token, token) = rc_token::create_token_pair(
            &secret_key,
            CurrentUser {
                uid: user.id,
                device: "web".to_string(),
            },
            token_expiry_seconds,
            refresh_token_expiry_seconds,
        )
        .map_err(InternalServerError)?;

        Ok(LoginApiResponse::Ok(Json(LoginResponse {
            token,
            refresh_token,
            user: UserInfo {
                id: user.id,
                email: user.email,
                name: user.name,
                age: user.age,
                created_at: user.created_at,
                password: "".to_string(),
                salt: "".to_string(),
            },
        })))
    }
}
