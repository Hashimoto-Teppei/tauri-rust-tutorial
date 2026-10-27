//! 第2章: 所有権・借用・ライフタイム

/// 末尾に "!" を追加する（可変参照で受け取り、その場で書き換える）
pub fn add_exclamation(s: &mut String) {
    todo!()
}

/// 先頭の空白を無視して最初の単語を返す（ヒント: split_whitespace）。
/// 所有権を取らず、元の文字列の一部を借用して返す
pub fn first_word(s: &str) -> &str {
    todo!()
}

/// 長い方の文字列を返す（同じ長さなら a）。戻り値は両方の引数と同じだけ生きる
pub fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    todo!()
}

/// Vec の所有権を受け取り、昇順ソートして返す（ムーブで受け取って返すパターン）
pub fn sorted(mut v: Vec<i32>) -> Vec<i32> {
    todo!()
}

/// 借用した名前のリストから、新しい String（所有値）を作って返す
pub fn join_names(names: &[String]) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_exclamation() {
        let mut s = String::from("hello");
        add_exclamation(&mut s);
        add_exclamation(&mut s);
        assert_eq!(s, "hello!!");
    }

    #[test]
    fn test_first_word() {
        assert_eq!(first_word("hello world"), "hello");
        assert_eq!(first_word("  rust "), "rust");
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn test_longest() {
        assert_eq!(longest("abc", "de"), "abc");
        assert_eq!(longest("a", "xyz"), "xyz");
        assert_eq!(longest("ab", "cd"), "ab");
    }

    #[test]
    fn test_sorted() {
        let v = vec![3, 1, 2];
        let v = sorted(v); // 元の v はムーブ済みなので、再束縛（シャドーイング）する
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_join_names() {
        let names = vec!["Alice".to_string(), "Bob".to_string()];
        assert_eq!(join_names(&names), "Alice, Bob");
        assert_eq!(names.len(), 2); // 借用なので names はまだ使える
    }
}
