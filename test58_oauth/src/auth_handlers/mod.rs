mod facebook;
mod google;

pub use self::{
    facebook::{
        facebook_auth_callback,
        login_with_facebook
    },
    google::{
        google_auth_callback,
        login_with_google
    }
};