//! Rust friendly `AvahiEntryGroup` wrappers/helpers

use std::sync::Arc;

use super::{client::ManagedAvahiClient, string_list::ManagedAvahiStringList};
use crate::ffi::UnwrapMutOrNull;
use crate::linux::avahi_util;
use crate::Result;
use avahi_sys::{
    avahi_client_errno, avahi_entry_group_add_service_strlst, avahi_entry_group_commit,
    avahi_entry_group_free, avahi_entry_group_is_empty, avahi_entry_group_new,
    avahi_entry_group_reset, AvahiEntryGroup, AvahiEntryGroupCallback, AvahiIfIndex, AvahiProtocol,
    AvahiPublishFlags,
};
use libc::{c_char, c_void};

/// Wraps the `AvahiEntryGroup` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiEntryGroup` when `ManagedAvahiEntryGroup::new()` is
/// invoked and calls the Avahi function responsible for freeing the group on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiEntryGroup {
    inner: *mut AvahiEntryGroup,
    _client: Arc<ManagedAvahiClient>,
}

impl ManagedAvahiEntryGroup {
    /// Initializes the underlying `*mut AvahiEntryGroup` and verifies it was created; returning
    /// `Err(String)` if unsuccessful.
    pub fn new(
        ManagedAvahiEntryGroupParams {
            client,
            callback,
            userdata,
        }: ManagedAvahiEntryGroupParams,
    ) -> Result<Self> {
        let inner = unsafe { avahi_entry_group_new(client.inner, callback, userdata) };

        if inner.is_null() {
            let err = avahi_util::get_error(unsafe { avahi_client_errno(client.inner) });
            Err(format!("could not initialize AvahiEntryGroup: {}", err).into())
        } else {
            Ok(Self {
                inner,
                _client: client,
            })
        }
    }

    /// Delegate function for [`avahi_entry_group_is_empty()`].
    ///
    /// [`avahi_entry_group_is_empty()`]: https://avahi.org/doxygen/html/publish_8h.html#af5a78ee1fda6678970536889d459d85c
    pub fn is_empty(&self) -> bool {
        unsafe { avahi_entry_group_is_empty(self.inner) != 0 }
    }

    /// Delegate function for [`avahi_entry_group_add_service()`].
    ///
    /// Also propagates any error returned into a `Result`.
    ///
    /// [`avahi_entry_group_add_service()`]: https://avahi.org/doxygen/html/publish_8h.html#acb05a7d3d23a3b825ca77cb1c7d00ce4
    pub fn add_service(
        &mut self,
        AddServiceParams {
            interface,
            protocol,
            flags,
            name,
            kind,
            domain,
            host,
            port,
            txt,
        }: AddServiceParams,
    ) -> Result<()> {
        avahi!(
            avahi_entry_group_add_service_strlst(
                self.inner,
                interface,
                protocol,
                flags,
                name,
                kind,
                domain,
                host,
                port,
                txt.map(|t| t.inner()).unwrap_mut_or_null()
            ),
            "could not register service"
        )?;

        avahi!(
            avahi_entry_group_commit(self.inner),
            "could not commit service"
        )
    }

    /// Delegate function for [`avahi_entry_group_reset()`].
    ///
    /// [`avahi_entry_group_reset()`]: https://avahi.org/doxygen/html/publish_8h.html#a1293bbccf878dbeb9916660022bc71b2
    pub fn reset(&mut self) {
        unsafe { avahi_entry_group_reset(self.inner) };
    }
}

impl Drop for ManagedAvahiEntryGroup {
    fn drop(&mut self) {
        unsafe { avahi_entry_group_free(self.inner) };
    }
}

/// Holds parameters for initializing a new `ManagedAvahiEntryGroup` with
/// `ManagedAvahiEntryGroup::new()`.
///
/// See [`avahi_entry_group_new()`] for more information about these parameters.
///
/// [avahi_entry_group_new()]: https://avahi.org/doxygen/html/publish_8h.html#abb17598f2b6ec3c3f69defdd488d568c
#[derive(Builder, BuilderDelegate)]
pub struct ManagedAvahiEntryGroupParams {
    client: Arc<ManagedAvahiClient>,
    callback: AvahiEntryGroupCallback,
    userdata: *mut c_void,
}

/// Holds parameters for `ManagedAvahiEntryGroup::add_service()`.
///
/// See [`avahi_entry_group_add_service()`] for more information about these parameters.
///
/// [`avahi_entry_group_add_service()`]: https://avahi.org/doxygen/html/publish_8h.html#acb05a7d3d23a3b825ca77cb1c7d00ce4
#[derive(Builder, BuilderDelegate)]
pub struct AddServiceParams<'a> {
    interface: AvahiIfIndex,
    protocol: AvahiProtocol,
    flags: AvahiPublishFlags,
    name: *const c_char,
    kind: *const c_char,
    domain: *const c_char,
    host: *const c_char,
    port: u16,
    txt: Option<&'a ManagedAvahiStringList>,
}
