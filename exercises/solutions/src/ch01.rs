//! 第1章: 変数・型・関数・制御フロー

/// 1〜n の FizzBuzz を空白区切りで返す。例: n=5 → "1 2 Fizz 4 Buzz"
pub fn fizzbuzz(n: u32) -> String {
    (1..=n)
        .map(|i| match (i % 3, i % 5) {
            (0, 0) => "FizzBuzz".to_string(),
            (0, _) => "Fizz".to_string(),
            (_, 0) => "Buzz".to_string(),
            _ => i.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// スライス中の偶数の合計
pub fn sum_of_evens(nums: &[i32]) -> i32 {
    let mut total = 0;
    for &n in nums {
        if n % 2 == 0 {
            total += n;
        }
    }
    total
}

/// 点数を評価に変換: 90〜100→"A", 70〜89→"B", 50〜69→"C", 0〜49→"D"。範囲外は "invalid"
pub fn grade(score: i32) -> &'static str {
    match score {
        90..=100 => "A",
        70..=89 => "B",
        50..=69 => "C",
        0..=49 => "D",
        _ => "invalid",
    }
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
