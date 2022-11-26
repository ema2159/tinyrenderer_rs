use std::ops;

#[derive(Debug)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> ops::Add<Point<T>> for Point<T>
where
    T: ops::Add<Output = T> + Copy,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let lhs = self;
        Self {
            x: lhs.x + rhs.x,
            y: lhs.y + rhs.y,
        }
    }
}

impl<T> ops::Mul<Vec2<T>> for Point<T>
where
    T: ops::Mul<Output = T> + Copy,
{
    type Output = Self;
    fn mul(self, rhs: Vec2<T>) -> Self::Output {
        let lhs = self;
        Self {
            x: lhs.x * rhs.x,
            y: lhs.y * rhs.y,
        }
    }
}

impl<T> Vec2<T>
where
    T: ops::Sub<Output = T> + Copy,

impl<T> Algebr2D<T> for Point<T>
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
{
    pub fn from_points(point1: &Point<T>, point2: &Point<T>) -> Vec2<T> {
        Vec2::<T> {
            x: point2.x - point1.x,
            y: point2.y - point1.y,
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
