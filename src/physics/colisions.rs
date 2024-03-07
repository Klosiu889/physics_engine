use cgmath::{InnerSpace, Zero};

const ORIGIN: cgmath::Vector3<f32> = cgmath::Vector3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};

const MAX_ITERATIONS: usize = 100;

pub trait Collider {
    fn update(&mut self, position: cgmath::Vector3<f32>, rotation: cgmath::Quaternion<f32>);
    fn get_center(&self) -> cgmath::Vector3<f32>;
    fn furthest_point(&self, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32>;
    fn collision<S>(&self, other: &S) -> bool
    where
        Self: Sized,
        S: Collider,
    {
        gjk_collision(self, other)
    }
}

fn same_direction(a: cgmath::Vector3<f32>, b: cgmath::Vector3<f32>) -> bool {
    a.dot(b) > 0.0
}

fn gjk_collision<S1, S2>(shape1: &S1, shape2: &S2) -> bool
where
    S1: Collider,
    S2: Collider,
{
    let mut simplex = Vec::new();
    let mut direction = shape2.get_center() - shape1.get_center();
    let sup = support(shape1, shape2, direction);

    simplex.push(sup);
    direction = ORIGIN - sup;

    for _ in 0..MAX_ITERATIONS {
        let sup = support(shape1, shape2, direction);
        if !same_direction(sup, direction) {
            return false;
        }

        simplex.push(sup);

        if next_simplex(&mut simplex, &mut direction) {
            return true;
        }
    }
    false
}

fn support<S1, S2>(a: &S1, b: &S2, direction: cgmath::Vector3<f32>) -> cgmath::Vector3<f32>
where
    S1: Collider,
    S2: Collider,
{
    a.furthest_point(direction) - b.furthest_point(-direction)
}

fn next_simplex(
    simplex: &mut Vec<cgmath::Vector3<f32>>,
    direction: &mut cgmath::Vector3<f32>,
) -> bool {
    match simplex.len() {
        2 => do_line(simplex, direction),
        3 => do_triangle(simplex, direction),
        4 => do_tetrahedron(simplex, direction),
        _ => panic!("Invalid simplex length"),
    }
}

fn do_line(simplex: &mut Vec<cgmath::Vector3<f32>>, direction: &mut cgmath::Vector3<f32>) -> bool {
    let a = simplex[0];
    let b = simplex[1];

    let ab = b - a;
    let ao = ORIGIN - a;

    if same_direction(ab, ao) {
        *direction = ab.cross(ao).cross(ab);
    } else {
        simplex.remove(1);
        *direction = ao;
    }

    *direction == cgmath::Vector3::zero()
}

fn do_triangle(
    simplex: &mut Vec<cgmath::Vector3<f32>>,
    direction: &mut cgmath::Vector3<f32>,
) -> bool {
    let a = simplex[0];
    let b = simplex[1];
    let c = simplex[2];

    let ab = b - a;
    let ac = c - a;
    let ao = ORIGIN - a;

    let abc = ab.cross(ac);

    if same_direction(abc.cross(ac), ao) {
        if same_direction(ac, ao) {
            simplex.remove(1);
            *direction = ac.cross(ao).cross(ac);
        } else {
            simplex.remove(2);
            return do_line(simplex, direction);
        }
    } else {
        if same_direction(ab.cross(abc), ao) {
            simplex.remove(2);
            return do_line(simplex, direction);
        }
        if same_direction(abc, ao) {
            *direction = abc;
        } else {
            simplex.remove(1);
            simplex.push(b);
            *direction = -abc;
        }
    }

    *direction == cgmath::Vector3::zero()
}

fn do_tetrahedron(
    simplex: &mut Vec<cgmath::Vector3<f32>>,
    direction: &mut cgmath::Vector3<f32>,
) -> bool {
    let a = simplex[0];
    let b = simplex[1];
    let c = simplex[2];
    let d = simplex[3];

    let ab = b - a;
    let ac = c - a;
    let ad = d - a;
    let ao = ORIGIN - a;

    let abc = ab.cross(ac);
    let acd = ac.cross(ad);
    let adb = ad.cross(ab);

    if same_direction(abc, ao) {
        *simplex = vec![a, b, c];
        return do_triangle(simplex, direction);
    }
    if same_direction(acd, ao) {
        *simplex = vec![a, c, d];
        return do_triangle(simplex, direction);
    }
    if same_direction(adb, ao) {
        *simplex = vec![a, d, b];
        return do_triangle(simplex, direction);
    }

    false
}
