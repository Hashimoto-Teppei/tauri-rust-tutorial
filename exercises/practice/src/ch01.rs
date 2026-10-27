//! 第1章: 変数・型・関数・制御フロー

/// 1〜n の FizzBuzz を空白区切りで返す。例: n=5 → "1 2 Fizz 4 Buzz"
pub fn fizzbuzz(n: u32) -> String {
    todo!()
}

/// スライス中の偶数の合計
pub fn sum_of_evens(nums: &[i32]) -> i32 {
    todo!()
}

/// 点数を評価に変換: 90〜100→"A", 70〜89→"B", 50〜69→"C", 0〜49→"D"。範囲外は "invalid"
pub fn grade(score: i32) -> &'static str {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fizzbuzz() {
        assert_eq!(fizzbuzz(5), "1 2 Fizz 4 Buzz");
        assert_eq!(fizzbuzz(15).split(' ').last(), Some("FizzBuzz"));
        assert_eq!(fizzbuzz(0), "");
    }

    #[test]
    fn test_sum_of_evens() {
        assert_eq!(sum_of_evens(&[1, 2, 3, 4, -6]), 0);
        assert_eq!(sum_of_evens(&[2, 4, 6]), 12);
        assert_eq!(sum_of_evens(&[]), 0);
    }

    #[test]
    fn test_grade() {
        assert_eq!(grade(100), "A");
        assert_eq!(grade(75), "B");
        assert_eq!(grade(50), "C");
        assert_eq!(grade(0), "D");
        assert_eq!(grade(101), "invalid");
        assert_eq!(grade(-1), "invalid");
    }
}
