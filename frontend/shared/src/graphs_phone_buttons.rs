//! Cypress it() title: check buttons - mobile.
//!
//! Viewport iphone-6. After the skeleton wait, the second small button,
//! `#dropdownFees`, and `.btn-group` are visible. No indexer body.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphsPhoneButtons {
    pub small_buttons_second_child_visible: bool,
    pub dropdown_fees_visible: bool,
    pub btn_group_visible: bool,
}

pub fn check_buttons_mobile(viewport: &str) -> GraphsPhoneButtons {
    let visible = viewport == "iphone-6";
    GraphsPhoneButtons {
        small_buttons_second_child_visible: visible,
        dropdown_fees_visible: visible,
        btn_group_visible: visible,
    }
}

pub fn check_buttons_tablet(viewport: &str) -> GraphsPhoneButtons {
    let visible = viewport == "ipad-2";
    GraphsPhoneButtons {
        small_buttons_second_child_visible: visible,
        dropdown_fees_visible: visible,
        btn_group_visible: visible,
    }
}

pub fn check_buttons_desktop(viewport: &str) -> GraphsPhoneButtons {
    let visible = viewport == "macbook-16";
    GraphsPhoneButtons {
        small_buttons_second_child_visible: visible,
        dropdown_fees_visible: visible,
        btn_group_visible: visible,
    }
}
