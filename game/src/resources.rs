/// A resource definition which is compiled into the binary using Windows SDK
/// tools.
#[derive(Debug, Clone, Copy)]
pub struct Resource {
    path: &'static str,
    id: isize,
}

impl Resource {
    /// Resource path relative to the root of the crate.
    #[allow(dead_code)]
    pub const fn path(&self) -> &'static str {
        self.path
    }

    /// The compiled resource ID, as would be returned by `MAKEINTRESOURCE` or
    /// as can be found in the `.rc` file.
    #[allow(dead_code)]
    pub const fn id(&self) -> isize {
        self.id
    }

    /// The compiled resource ID, as would be returned by `MAKEINTRESOURCE` or
    /// as can be found in the `.rc` file.
    #[allow(dead_code)]
    pub fn id_string(&self) -> String {
        self.id.to_string()
    }
}

/// Ferris app icon.
pub const FERRIS_ICON: Resource = Resource {
    path: "icon.ico",
    id: 1,
};
