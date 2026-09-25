//! First screen. Shows the bech32 npub only. No secret field.

use std::fmt;

/// Widgets on the npub popup. None of them is a private-key field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupWidgetKind {
    Title,
    NpubValue,
    Continue,
}

impl PopupWidgetKind {
    pub fn is_private_key_field(self) -> bool {
        match self {
            Self::Title | Self::NpubValue | Self::Continue => false,
        }
    }

    pub fn field_name(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::NpubValue => "npub",
            Self::Continue => "continue",
        }
    }
}

/// Why an npub popup was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupError {
    /// The value started with `nsec1` or was not an `npub1` string.
    NotNpub,
}

/// Popup model. It holds the npub and the three widgets. It does not hold a secret.
#[derive(Clone, PartialEq, Eq)]
pub struct NpubPopupModel {
    npub: String,
}

impl fmt::Debug for NpubPopupModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NpubPopupModel")
            .field("npub", &self.npub)
            .field(
                "widgets",
                &self
                    .widgets()
                    .iter()
                    .map(|widget| widget.field_name())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl NpubPopupModel {
    /// `npub` must already start with `npub1`. Anything starting with `nsec1` is rejected.
    pub fn new(npub: impl Into<String>) -> Result<Self, PopupError> {
        let npub = npub.into();
        if npub.starts_with("nsec1") || npub.contains("nsec1") || !npub.starts_with("npub1") {
            return Err(PopupError::NotNpub);
        }
        Ok(Self { npub })
    }

    pub fn npub(&self) -> &str {
        &self.npub
    }

    pub fn widgets(&self) -> [PopupWidgetKind; 3] {
        [
            PopupWidgetKind::Title,
            PopupWidgetKind::NpubValue,
            PopupWidgetKind::Continue,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_widget_has_no_private_key_field() {
        let keys = nostr::key::Keys::generate();
        let npub = nostr::nips::nip19::ToBech32::to_bech32(&keys.public_key())
            .expect("public key encodes");
        let model = NpubPopupModel::new(npub).expect("npub");
        for widget in model.widgets() {
            assert!(!widget.is_private_key_field());
        }
        let names: Vec<&str> = model
            .widgets()
            .iter()
            .map(|widget| widget.field_name())
            .collect();
        assert_eq!(names, vec!["title", "npub", "continue"]);
        assert!(model.npub().starts_with("npub1"));
        let debug = format!("{model:?}");
        assert!(!debug.contains("nsec1"));
        assert!(NpubPopupModel::new("nsec1aaaaaaaa").is_err());
    }
}
