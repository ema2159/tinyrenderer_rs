use std::ops;

#[derive(Debug)]
pub struct Point2<T> {
    pub x: T,
    pub y: T,
}

impl<T, U> ops::Add<&U> for Point2<T>
where
    T: ops::Add<Output = T> + Copy,
    U: Algebr2D<T>,
{
    type Output = Self;
    fn add(self, rhs: &U) -> Self::Output {
        let lhs = self;
        let rhs = rhs.as_vec();
        Self {
            x: lhs.x + rhs.x,
            y: lhs.y + rhs.y,
        }
    }
}

impl<T, U> ops::Mul<&U> for Point2<T>
where
    T: ops::Mul<Output = T> + Copy,
    U: Algebr2D<T>,
{
    type Output = Self;
    fn mul(self, rhs: &U) -> Self::Output {
        let lhs = self;
        let rhs = rhs.as_vec();
        Self {
            x: lhs.x * rhs.x,
            y: lhs.y * rhs.y,
        }
    }
}

impl<T, U> ops::Sub<&U> for Point2<T>
where
    T: ops::Sub<Output = T> + Copy,
    U: Algebr2D<T>,
{
    type Output = Self;
    fn sub(self, rhs: &U) -> Self::Output {
        let lhs = self;
        let rhs = rhs.as_vec();
        Self {
            x: lhs.x - rhs.x,
            y: lhs.y - rhs.y,
        }
    }
}

impl<T> Algebr2D<T> for Point2<T>
where
    T: Copy,
{
    fn as_vec(&self) -> Vec2<T> {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T>
where
    T: ops::Sub<Output = T> + ops::Mul<Output = T> + Copy,
{
    pub fn from_points(point1: &Point2<T>, point2: &Point2<T>) -> Vec2<T> {
        Vec2::<T> {
            x: point2.x - point1.x,
            y: point2.y - point1.y,
        }
    }

    pub fn cross(vec0: &Vec2<T>, vec1: &Vec2<T>) -> T {
        (vec0.x * vec1.y) - (vec0.y * vec1.x)
    }
}

impl<T> Algebr2D<T> for Vec2<T>
where
    T: Copy,
{
    fn as_vec(&self) -> Vec2<T> {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

pub trait Algebr2D<T> {
    fn as_vec(&self) -> Vec2<T>;
}

#[derive(Debug)]
pub struct Point3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T, U> ops::Add<&U> for Point3<T>
where
    T: ops::Add<Output = T> + Copy,
    U: Algebr3D<T>,
{
    type Output = Self;
    fn add(self, rhs: &U) -> Self::Output {
        let lhs = self;
        let rhs = rhs.as_vec();
        Self {
            x: lhs.x + rhs.x,
            y: lhs.y + rhs.y,
            z: lhs.z + rhs.z,
        }
    }
}

impl<T, U> ops::Mul<&U> for Point3<T>
where
    T: ops::Mul<Output = T> + Copy,
    U: Algebr3D<T>,
{
    type Output = Self;
    fn mul(self, rhs: &U) -> Self::Output {
        let lhs = self;
        let rhs = rhs.as_vec();
        Self {
            x: lhs.x * rhs.x,
            y: lhs.y * rhs.y,
            z: lhs.z * rhs.z,
        }
    }
}

impl<T, U> ops::Sub<&U> for Point3<T>
where
    T: ops::Sub<Output = T> + Copy,
    U: Algebr3D<T>,
{
    type Output = Self;
    fn sub(self, rhs: &U) -> Self::Output {
        let lhs = self;
        let rhs = rhs.as_vec();
        Self {
            x: lhs.x - rhs.x,
            y: lhs.y - rhs.y,
            z: lhs.z - rhs.z,
        }
    }
}

impl<T> Algebr3D<T> for Point3<T>
where
    T: Copy,
{
    fn as_vec(&self) -> Vec3<T> {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

#[derive(Debug)]
pub struct Vec3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vec3<T>
where
    T: ops::Add<Output = T> + ops::Sub<Output = T> + ops::Mul<Output = T> + Copy,
{
    pub fn from_points(point1: &Point3<T>, point2: &Point3<T>) -> Vec3<T> {
        Vec3::<T> {
            x: point2.x - point1.x,
            y: point2.y - point1.y,
            z: point2.z - point1.z,
        }
    }

    pub fn cross(vec0: Vec3<T>, vec1: Vec3<T>) -> Vec3<T> {
        let x = (vec0.y * vec1.z) - (vec0.z * vec1.y);
        let y = (vec0.z * vec1.x) - (vec0.x * vec1.z);
        let z = (vec0.x * vec1.y) - (vec0.y * vec1.x);
        Vec3 { x, y, z }
    }
}

impl Vec3<i32> {
    pub fn normalize(self) -> Vec3<f32> {
        let magnitude = (((self.x * self.x) + (self.y * self.y) + (self.z * self.z)) as f32).sqrt();
        Vec3 {
            x: self.x as f32 / magnitude,
            y: self.y as f32 / magnitude,
            z: self.z as f32 / magnitude,
        }
    }
}

impl Vec3<f32> {
    pub fn normalize(self) -> Vec3<f32> {
        let magnitude = (((self.x * self.x) + (self.y * self.y) + (self.z * self.z)) as f32).sqrt();
        Vec3 {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }
}

pub trait Algebr3D<T> {
    fn as_vec(&self) -> Vec3<T>;
}
