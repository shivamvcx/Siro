use egui::{Pos2, Rect, Vec2};

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub pan: Pos2,
    pub zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pan: Pos2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

impl Camera {
    pub const MIN_ZOOM: f32 = 0.05; // 5%
    pub const MAX_ZOOM: f32 = 20.0; // 2000%

    pub fn new(pan: Pos2, zoom: f32) -> Self {
        Self {
            pan,
            zoom: zoom.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM),
        }
    }

    /// Converts a screen pixel position to world canvas coordinates.
    pub fn screen_to_world(&self, screen_pos: Pos2, viewport_rect: Rect) -> Pos2 {
        let relative_screen = screen_pos - viewport_rect.min;
        let world_x = (relative_screen.x - self.pan.x) / self.zoom;
        let world_y = (relative_screen.y - self.pan.y) / self.zoom;
        Pos2::new(world_x, world_y)
    }

    /// Converts a world canvas coordinate to screen pixel coordinates.
    pub fn world_to_screen(&self, world_pos: Pos2, viewport_rect: Rect) -> Pos2 {
        let screen_x = world_pos.x * self.zoom + self.pan.x + viewport_rect.min.x;
        let screen_y = world_pos.y * self.zoom + self.pan.y + viewport_rect.min.y;
        Pos2::new(screen_x, screen_y)
    }

    /// Converts a world vector to screen vector (scaled by zoom).
    pub fn world_vec_to_screen(&self, world_vec: Vec2) -> Vec2 {
        world_vec * self.zoom
    }

    /// Converts a screen vector to world vector.
    pub fn screen_vec_to_world(&self, screen_vec: Vec2) -> Vec2 {
        screen_vec / self.zoom
    }

    /// Converts a world-space bounding box to screen-space rect.
    pub fn world_rect_to_screen(&self, world_rect: Rect, viewport_rect: Rect) -> Rect {
        let min = self.world_to_screen(world_rect.min, viewport_rect);
        let max = self.world_to_screen(world_rect.max, viewport_rect);
        Rect::from_min_max(min, max)
    }

    /// Returns the world-space bounding box visible inside the viewport.
    pub fn visible_world_rect(&self, viewport_rect: Rect) -> Rect {
        let top_left = self.screen_to_world(viewport_rect.min, viewport_rect);
        let bottom_right = self.screen_to_world(viewport_rect.max, viewport_rect);
        Rect::from_min_max(top_left, bottom_right)
    }

    /// Zooms anchored around the specified screen pointer position.
    pub fn zoom_at_pointer(&mut self, pointer_screen: Pos2, factor: f32, viewport_rect: Rect) {
        if factor <= 0.0 || !factor.is_finite() {
            return;
        }

        let old_zoom = self.zoom;
        let new_zoom = (old_zoom * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (new_zoom - old_zoom).abs() < 1e-6 {
            return;
        }

        let rel_screen = pointer_screen - viewport_rect.min;
        let old_pan = self.pan;
        let new_pan_x = rel_screen.x - (rel_screen.x - old_pan.x) * (new_zoom / old_zoom);
        let new_pan_y = rel_screen.y - (rel_screen.y - old_pan.y) * (new_zoom / old_zoom);

        self.zoom = new_zoom;
        self.pan = Pos2::new(new_pan_x, new_pan_y);
    }

    /// Sets explicit zoom level anchored around the screen center or pointer.
    pub fn set_zoom_anchored(&mut self, new_zoom_level: f32, anchor_screen: Pos2, viewport_rect: Rect) {
        let factor = new_zoom_level / self.zoom;
        self.zoom_at_pointer(anchor_screen, factor, viewport_rect);
    }

    pub fn pan_by(&mut self, screen_delta: Vec2) {
        self.pan += screen_delta;
    }

    pub fn fit_to_rect(&mut self, target_world_rect: Rect, viewport_rect: Rect) {
        let vp_size = viewport_rect.size();
        let target_size = target_world_rect.size();

        if target_size.x <= 0.0 || target_size.y <= 0.0 || vp_size.x <= 0.0 || vp_size.y <= 0.0 {
            return;
        }

        let padding = 60.0;
        let avail_w = (vp_size.x - padding * 2.0).max(100.0);
        let avail_h = (vp_size.y - padding * 2.0).max(100.0);

        let zoom_x = avail_w / target_size.x;
        let zoom_y = avail_h / target_size.y;
        let new_zoom = zoom_x.min(zoom_y).clamp(Self::MIN_ZOOM, 2.0);

        let target_center_world = target_world_rect.center();
        let screen_center = viewport_rect.size() / 2.0;

        self.zoom = new_zoom;
        self.pan = Pos2::new(
            screen_center.x - target_center_world.x * new_zoom,
            screen_center.y - target_center_world.y * new_zoom,
        );
    }
}
