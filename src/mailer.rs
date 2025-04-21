#[cfg(feature = "web")]
use lettre::transport::smtp::authentication::Credentials;
#[cfg(feature = "web")]
use lettre::transport::smtp::client::{Tls, TlsParameters};
#[cfg(feature = "web")]
use lettre::{Message, SmtpTransport, Transport};
#[cfg(feature = "web")]
use rand::Rng;
#[cfg(feature = "web")]
use std::error::Error;
#[cfg(feature = "web")]
use std::fs;

#[cfg(feature = "web")]
pub struct Mailer {
    smtp: SmtpTransport,
}

#[cfg(feature = "web")]
impl Mailer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Read the email credentials from a config file.
        // The file "config/mail_credentials.txt" should have the email on the first line and the password on the second.
        let creds_data = fs::read_to_string("config/mail_credentials.txt")?;
        let mut lines = creds_data.lines();
        let email = lines.next().unwrap_or("").trim().to_string();
        let password = lines.next().unwrap_or("").trim().to_string();
        
        if email.is_empty() || password.is_empty() {
            return Err("Invalid mail credentials in config file".into());
        }

        let creds = Credentials::new(email, password);

        let tls_parameters = TlsParameters::new("smtp.iitd.ac.in".to_string())?;

        let smtp = SmtpTransport::relay("smtp.iitd.ac.in")?
            .credentials(creds)
            .port(465)
            .tls(Tls::Wrapper(tls_parameters))
            .build();

        Ok(Mailer { smtp })
    }

    pub fn send_password_reset(
        &self,
        to_email: &str,
        reset_code: &str,
    ) -> Result<(), Box<dyn Error>> {
        let email = Message::builder()
            .from("Rusty <cs1230807@iitd.ac.in>".parse()?)
            .to(to_email.parse()?)
            .subject("Password Reset Request")
            .body(format!(
                "Your password reset code is: {}\nThis code will expire in 1 hour.",
                reset_code
            ))?;

        self.smtp.send(&email)?;
        Ok(())
    }
}

#[cfg(feature = "web")]
pub fn generate_reset_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
