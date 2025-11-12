use anyhow::Result;
use resend_rs::{Resend, types::CreateEmailBaseOptions};

pub struct EmailService {
    client: Resend,
    from_address: String,
}

impl EmailService {
    pub fn new(api_key: &str, from_address: &str) -> Self {
        Self {
            client: Resend::new(api_key),
            from_address: from_address.to_string(),
        }
    }

    pub async fn send_verification_code(&self, to: &str, code: &str) -> Result<()> {
        let subject = "Email Verification Code";
        let html = format!(
            r#"<h2>Verify Your Email</h2>
            <p>Your verification code is: <strong>{}</strong></p>
            <p>This code will expire in 1 hour.</p>"#,
            code
        );
        tracing::debug!(
            "Sending verification code email from {} to {}",
            self.from_address,
            to
        );
        let email = CreateEmailBaseOptions::new(&self.from_address, [to], subject).with_html(&html);

        self.client.emails.send(email).await.map_err(|e| {
            tracing::error!("Failed to send verification code email: {:?}", e);
            e
        })?;
        Ok(())
    }

    pub async fn send_password_reset_code(&self, to: &str, code: &str) -> Result<()> {
        let subject = "Password Reset Code";
        let html = format!(
            r#"<h2>Reset Your Password</h2>
            <p>Your password reset code is: <strong>{}</strong></p>
            <p>This code will expire in 1 hour.</p>"#,
            code
        );

        let email = CreateEmailBaseOptions::new(&self.from_address, [to], subject).with_html(&html);

        self.client.emails.send(email).await?;
        Ok(())
    }

    pub async fn send_email(&self, to: &str, subject: &str, html_content: &str) -> Result<()> {
        let email =
            CreateEmailBaseOptions::new(&self.from_address, [to], subject).with_html(html_content);

        self.client.emails.send(email).await?;
        Ok(())
    }
}
