//! 第3章: struct / enum / trait / ジェネリクス

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub w: f64,
    pub h: f64,
}

impl Rect {
    /// 正方形を作る関連関数（TS の static メソッド相当）
    pub fn square(size: f64) -> Self {
        todo!()
    }

    pub fn area(&self) -> f64 {
        todo!()
    }

    /// other が自分の中に収まるか
    pub fn can_hold(&self, other: &Rect) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { r: f64 },
    Rect(Rect),
    Triangle { base: f64, height: f64 },
}

pub trait Describe {
    fn name(&self) -> String;

    /// デフォルト実装つきメソッド
    fn describe(&self) -> String {
        format!("これは{}です", self.name())
    }
}

impl Shape {
    pub fn area(&self) -> f64 {
        todo!()
    }
}

impl Describe for Shape {
    // "円" / "長方形" / "三角形" を返す
    fn name(&self) -> String {
        todo!()
    }
}

/// 面積が最大の図形を返す。空なら None
pub fn largest(shapes: &[Shape]) -> Option<&Shape> {
    todo!()
}

/// Describe を実装した任意の型を受け取るジェネリック関数
pub fn describe_all<T: Describe>(items: &[T]) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let big = Rect { w: 10.0, h: 5.0 };
        let small = Rect::square(3.0);
        assert_eq!(big.area(), 50.0);
        assert!(big.can_hold(&small));
        assert!(!small.can_hold(&big));
    }

    #[test]
    fn test_shape_area() {
        assert_eq!(Shape::Rect(Rect::square(2.0)).area(), 4.0);
        assert_eq!(Shape::Triangle { base: 4.0, height: 3.0 }.area(), 6.0);
        assert!((Shape::Circle { r: 1.0 }.area() - 3.14159).abs() < 0.001);
    }

    #[test]
    fn test_describe() {
        assert_eq!(Shape::Circle { r: 1.0 }.describe(), "これは円です");
        let all = describe_all(&[Shape::Rect(Rect::square(1.0)), Shape::Triangle { base: 1.0, height: 1.0 }]);
        assert_eq!(all, vec!["これは長方形です", "これは三角形です"]);
    }

    #[test]
    fn test_largest() {
        let shapes = vec![
            Shape::Rect(Rect::square(2.0)),
            Shape::Circle { r: 2.0 },
            Shape::Triangle { base: 1.0, height: 1.0 },
        ];
        assert_eq!(largest(&shapes), Some(&Shape::Circle { r: 2.0 }));
        assert_eq!(largest(&[]), None);
    }
}
