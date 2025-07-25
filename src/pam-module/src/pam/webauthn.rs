use crate::pam::{sys_err, sys_info};
use pamsm::{Pam, PamLibExt};
use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;
use std::time::Instant;
use tokio::task;
use webauthn_authenticator_rs::AuthenticatorBackend;
use webauthn_authenticator_rs::ctap2::CtapAuthenticator;
use webauthn_authenticator_rs::prelude::Url;
use webauthn_authenticator_rs::transport::{TokenEvent, Transport};
use webauthn_authenticator_rs::types::{CableRequestType, CableState, EnrollSampleStatus};
use webauthn_authenticator_rs::ui::UiCallback;
use webauthn_authenticator_rs::usb::{USBToken, USBTransport};
use webauthn_rs::prelude::PublicKeyCredential;
use webauthn_rs_core::proto::PublicKeyCredentialRequestOptions;

pub static UI: LazyLock<(PamWebauthn, flume::Receiver<PamReq>)> = LazyLock::new(|| {
    let (tx, rx) = flume::bounded(1);
    let ui = PamWebauthn { tx: tx.clone() };
    (ui, rx)
});

#[derive(Debug)]
pub enum PamReq {
    Info(String),
    Err(String),
    GetPin(flume::Sender<String>),
    Result(PublicKeyCredential),
}

pub struct PamWebauthn {
    tx: flume::Sender<PamReq>,
}

impl Debug for PamWebauthn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebauthnUI")
    }
}

impl PamWebauthn {
    // pub fn new() -> (Self, flume::Receiver<PamReq>) {
    //     let (tx, rx) = flume::bounded(1);
    //     (Self { tx }, rx)
    // }

    pub async fn perform_auth(
        pamh: &Pam,
        mut authenticator: CtapAuthenticator<'static, USBToken, Self>,
        origin: Url,
        public_key: PublicKeyCredentialRequestOptions,
    ) -> anyhow::Result<PublicKeyCredential> {
        let (ui, rx) = &*UI;

        task::spawn_blocking(move || {
            let timeout = public_key.timeout.unwrap_or(60_000);
            let start = Instant::now();

            loop {
                match authenticator.perform_auth(
                    origin.clone(),
                    public_key.clone(),
                    start.elapsed().as_millis() as u32 - timeout,
                ) {
                    Ok(pk_cred) => {
                        ui.tx.send(PamReq::Result(pk_cred)).unwrap();
                        break;
                    }
                    Err(err) => {
                        ui.tx
                            .send(PamReq::Err(format!("Passkey validation error: {err:?}")))
                            .unwrap();
                    }
                }
            }
        });

        let mut touch_requested = false;
        loop {
            match rx.recv_async().await.unwrap() {
                PamReq::Info(msg) => {
                    if msg.contains("Touch") {
                        if !touch_requested {
                            println!("{msg}");
                            touch_requested = true;
                        }
                    } else {
                        println!("{msg}");
                        sys_info(pamh, &msg);
                    }
                }
                PamReq::Err(err) => {
                    eprintln!("{err}");
                    sys_err(pamh, &err);
                }
                PamReq::GetPin(ack) => loop {
                    match pamh.get_authtok(Some("Enter PIN: ")).unwrap() {
                        None => {
                            println!("PIN must not be empty");
                        }
                        Some(cstr) => {
                            let p = cstr.to_str().unwrap_or_default().to_string();
                            ack.send_async(p).await?;
                            break;
                        }
                    }
                },
                PamReq::Result(pk_cred) => {
                    return Ok(pk_cred);
                }
            }
        }
    }

    pub async fn wait_for_passkey<U: UiCallback>(ui: &U) -> CtapAuthenticator<USBToken, U> {
        use futures::StreamExt;

        let reader = USBTransport::new().await.unwrap();

        loop {
            match reader.watch().await {
                Ok(mut tokens) => {
                    while let Some(event) = tokens.next().await {
                        match event {
                            TokenEvent::Added(token) => {
                                let auth = CtapAuthenticator::new(token, ui).await;

                                if let Some(auth) = auth {
                                    return auth;
                                }
                            }

                            TokenEvent::EnumerationComplete => {
                                println!(
                                    "device enumeration completed without detecting a FIDO2 authenticator, connect one to authenticate!"
                                );
                            }

                            TokenEvent::Removed(_) => {}
                        }
                    }
                }
                Err(e) => panic!("Error: {e:?}"),
            }

            eprintln!("Insert Passkey");
        }
    }
}

impl UiCallback for PamWebauthn {
    fn request_pin(&self) -> Option<String> {
        let (ack, rx) = flume::bounded(1);
        self.tx.send(PamReq::GetPin(ack)).unwrap();
        rx.recv().ok()
    }

    fn request_touch(&self) {
        self.tx
            .send(PamReq::Info("Touch Passkey".to_string()))
            .unwrap()
    }

    fn processing(&self) {
        // self.tx
        //     .send(PamReq::Info("Processing ...".to_string()))
        //     .unwrap()
    }

    fn fingerprint_enrollment_feedback(
        &self,
        _remaining_samples: u32,
        _feedback: Option<EnrollSampleStatus>,
    ) {
        unreachable!("Fingerprint Enrollment not supported")
    }

    fn cable_qr_code(&self, _request_type: CableRequestType, _url: String) {
        // match request_type {
        //     CableRequestType::DiscoverableMakeCredential | CableRequestType::MakeCredential => {
        //         println!(
        //             "Scan the QR code with your mobile device to create a new credential with caBLE:"
        //         );
        //     }
        //     CableRequestType::GetAssertion => {
        //         println!("Scan the QR code with your mobile device to sign in with caBLE:");
        //     }
        // }
        // println!("This feature requires Android with Google Play, or iOS 16 or later.");
        //
        // {
        //     let qr = QrCode::new(&url).expect("Could not create QR code");
        //
        //     let code = qr
        //         .render::<Dense1x2>()
        //         .dark_color(Dense1x2::Light)
        //         .light_color(Dense1x2::Dark)
        //         .build();
        //
        //     println!("{}", code);
        // }
        todo!("cable_qr_code")
    }

    fn dismiss_qr_code(&self) {
        self.tx
            .send(PamReq::Info(
                "caBLE authenticator detected, connecting...".to_string(),
            ))
            .unwrap()
    }

    fn cable_status_update(&self, state: CableState) {
        self.tx
            .send(PamReq::Info(format!("caBLE status: {state:?}")))
            .unwrap()
    }
}
