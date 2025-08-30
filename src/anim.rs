use dunge_winit::dunge::glam::Vec2;

#[derive(Clone, Copy)]
pub struct Animate {
    t: f32,
    origin: Vec2,
    target: Vec2,
}

impl Animate {
    pub fn new(initial: Vec2) -> Self {
        Self {
            t: 1.,
            origin: Vec2::ZERO,
            target: initial,
        }
    }

    pub fn with_target(self, target: Vec2) -> Self {
        Self {
            t: 0.,
            origin: self.point(),
            target,
        }
    }

    pub fn point(self) -> Vec2 {
        let eased = ease_out_quart(self.t);
        self.origin.lerp(self.target, eased)
    }

    pub fn advance(self, dt: f32) -> Self {
        let t = f32::clamp(self.t + dt, 0., 1.);
        Self { t, ..self }
    }
}

fn ease_out_quart(x: f32) -> f32 {
    1. - f32::powi(1. - x, 4)
}
