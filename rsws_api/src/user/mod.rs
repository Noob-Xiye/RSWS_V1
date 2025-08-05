// 导入新的处理器
pub mod avatar;
pub mod email;
pub mod password;

// 在 UserRouter 中添加新的路由
pub fn router(user_service: Arc<UserService>) -> Router {
    let profile_handler = UserProfileHandler::new(user_service.clone());
    let password_handler = UserPasswordHandler::new(user_service.clone());
    let email_handler = UserEmailHandler::new(user_service.clone());
    let avatar_handler = UserAvatarHandler::new(user_service.clone());

    Router::new()
        .push(
            Router::new()
                .path("profile")
                .get(profile_handler.get_profile)
                .patch(profile_handler.update_profile)
                .push(
                    Router::new()
                        .path("completion")
                        .get(profile_handler.check_profile_completion),
                ),
        )
        .push(
            Router::new()
                .path("password")
                .put(password_handler.change_password),
        )
        .push(
            Router::new()
                .path("email")
                .push(
                    Router::new()
                        .path("code")
                        .post(email_handler.send_email_change_code),
                )
                .push(
                    Router::new()
                        .path("verify")
                        .post(email_handler.verify_email_change),
                ),
        )
        .push(
            Router::new()
                .path("avatar")
                .post(avatar_handler.upload_avatar),
        )
}
