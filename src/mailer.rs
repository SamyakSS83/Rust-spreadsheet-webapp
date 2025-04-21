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

/// Email sending functionality for the application
///
/// This module provides email capabilities for the web application,
/// specifically for password reset functionality. It is only compiled
/// when the "web" feature is enabled.
#[cfg(feature = "web")]
pub struct Mailer {
    /// SMTP transport client configured for sending emails
    smtp: SmtpTransport,
}

#[cfg(feature = "web")]
impl Mailer {
    /// Creates a new Mailer instance
    ///
    /// Initializes a new mailer by reading credentials from a config file
    /// and setting up the SMTP transport with TLS.
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn Error>>` - A new Mailer instance or an error
    ///
    /// # Errors
    /// * Returns an error if credentials cannot be read from the config file
    /// * Returns an error if the SMTP relay cannot be configured
    ///
    /// # Configuration
    /// Requires a file at "config/mail_credentials.txt" with:
    /// - Email address on the first line
    /// - Password on the second line
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

    /// Sends a password reset email
    ///
    /// Composes and sends an email containing a password reset code to the specified
    /// email address.
    ///
    /// # Arguments
    /// * `to_email` - The recipient's email address
    /// * `reset_code` - The generated password reset code
    ///
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - Success or an error
    ///
    /// # Errors
    /// * Returns an error if the email address is invalid
    /// * Returns an error if the SMTP transport fails to send the email
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

/// Generates a random reset code for password recovery
///
/// Creates an 8-character alphanumeric code (uppercase letters and numbers)
/// for use in the password reset process.
///
/// # Returns
/// * `String` - A randomly generated 8-character code
///
/// # Example
/// ```
/// use cop::mailer::generate_reset_code;
///
/// let reset_code = generate_reset_code();
/// assert_eq!(reset_code.len(), 8);
/// ```
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
