use cgmath::{InnerSpace, Zero};

use self::colisions::{collision, Collider};
pub mod colisions;

pub trait Solver {
    fn solve(&self, object: &PhysicalObject);
}

pub struct PhysicalObject {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    velocity: cgmath::Vector3<f32>,
    forces: cgmath::Vector3<f32>,
    mass: f32,
    have_gravity: bool,
    have_collision: bool,
    collider: Box<dyn Collider>,
}

impl PhysicalObject {
    pub fn new(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Quaternion<f32>,
        velocity: Option<cgmath::Vector3<f32>>,
        forces: Option<cgmath::Vector3<f32>>,
        mass: f32,
        collider: Box<dyn Collider>,
    ) -> Self {
        PhysicalObject {
            position,
            rotation,
            velocity: velocity.unwrap_or(cgmath::Vector3::zero()),
            forces: forces.unwrap_or(cgmath::Vector3::zero()),
            mass,
            have_gravity: false,
            have_collision: false,
            collider,
        }
    }

    const GRAVITY: cgmath::Vector3<f32> = cgmath::Vector3 {
        x: 0.0,
        y: -9.81,
        z: 0.0,
    };

    pub fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }

    pub fn get_rotation(&self) -> cgmath::Quaternion<f32> {
        self.rotation
    }

    pub fn update(&mut self, dt: f32) {
        let forces = if self.have_gravity {
            self.forces + self.mass * Self::GRAVITY
        } else {
            self.forces
        };
        self.velocity += forces / self.mass * dt;
        self.position += self.velocity * dt;
        self.position.y = self.position.y.max(-1.0);

        self.collider.update(self.position, self.rotation);
    }

    pub fn apply_force(&mut self, force: cgmath::Vector3<f32>) {
        self.forces = force;
    }

    pub fn enable_gravity(&mut self) {
        self.have_gravity = true;
    }

    pub fn disable_gravity(&mut self) {
        self.have_gravity = false;
    }

    pub fn enable_collision(&mut self) {
        self.have_collision = true;
    }

    pub fn disable_collision(&mut self) {
        self.have_collision = false;
    }

    pub fn collide(&self, other: &PhysicalObject) -> bool {
        self.have_collision && other.have_collision && collision(&self.collider, &other.collider)
    }
}

pub struct Sphere {
    center: cgmath::Vector3<f32>,
    radius: f32,
}

impl Sphere {
    pub fn new(center: cgmath::Vector3<f32>, radius: f32) -> Self {
        Sphere { center, radius }
    }
}

impl colisions::Collider for Sphere {
    fn update(&mut self, position: cgmath::Vector3<f32>, _rotation: cgmath::Quaternion<f32>) {
        self.center = position;
    }
    fn get_center(&self) -> cgmath::Vector3<f32> {
        self.center
    }

    fn furthest_point(&self, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        self.center + direction.normalize_to(self.radius)
    }
}

pub struct Cube {
    center: cgmath::Vector3<f32>,
    half_size: cgmath::Vector3<f32>,
}

impl Cube {
    pub fn new(center: cgmath::Vector3<f32>, half_size: cgmath::Vector3<f32>) -> Self {
        Cube { center, half_size }
    }
}

impl colisions::Collider for Cube {
    fn update(&mut self, position: cgmath::Vector3<f32>, _rotation: cgmath::Quaternion<f32>) {
        self.center = position;
    }

    fn get_center(&self) -> cgmath::Vector3<f32> {
        self.center
    }

    fn furthest_point(&self, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        self.center + direction.normalize_to(self.half_size.magnitude())
    }
}
