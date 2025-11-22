use crate::desktop_environment::WorkspaceId;

#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub id: WorkspaceId,
    pub active: bool,
    animation_progress: f32, // 0.0 to 1.0
}

impl WorkspaceState {
    pub fn new(id: WorkspaceId, active: bool) -> Self {
        Self {
            id,
            active,
            animation_progress: if active { 1.0 } else { 0.0 },
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn needs_update(&self) -> bool {
        self.active && self.animation_progress < 1.0
            || !self.active && self.animation_progress > 0.0
    }

    pub fn update(&mut self) {
        const ANIMATION_SPEED: f32 = 0.08;

        if self.active && self.animation_progress < 1.0 {
            self.animation_progress = (self.animation_progress + ANIMATION_SPEED).min(1.0);
        } else if !self.active && self.animation_progress > 0.0 {
            self.animation_progress = (self.animation_progress - ANIMATION_SPEED).max(0.0);
        }
    }

    pub fn current_width_multiplier(&self) -> f32 {
        // Smooth easing function
        let t = self.animation_progress;
        let eased = t * t * (3.0 - 2.0 * t); // Smoothstep
        1.0 + eased * 1.25 // Goes from 1.0 to 2.25
    }
}
