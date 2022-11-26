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

    pub fn cross<U: Algebr2D<T>, V: Algebr2D<T>>(elem1: &U, elem2: &V) -> T {
        let vec1 = elem1.as_vec();
        let vec2 = elem2.as_vec();
        (vec1.x * vec2.y) - (vec1.y * vec2.x)
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
