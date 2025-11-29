use bitflags::bitflags;

bitflags! {
    /// Visual state for an entity.
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct VisualFlags: u32 {
        const VISIBLE = 0b0001;
        const SELECTED = 0b0010;
        const HIGHLIGHTED = 0b0100;
    }
}

impl VisualFlags {
    pub fn visible(self, on: bool) -> Self {
        let mut flags = self;
        flags.set(Self::VISIBLE, on);
        flags
    }

    pub fn selected(self, on: bool) -> Self {
        let mut flags = self;
        flags.set(Self::SELECTED, on);
        flags
    }

    pub fn highlighted(self, on: bool) -> Self {
        let mut flags = self;
        flags.set(Self::HIGHLIGHTED, on);
        flags
    }
}

bitflags! {
    /// Dirty markers for incremental sync.
    #[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct DirtyFlags: u32 {
        const GEOMETRY = 0b0001;
        const TRANSFORM = 0b0010;
        const VISUAL = 0b0100;
    }
}
