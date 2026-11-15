//! 개인정보를 디버그 출력과 로그에서 감추는 타입.

use std::fmt;

/// 개인정보를 감싼다. `{:?}`로 찍으면 값 대신 `***`가 나온다.
///
/// `Display`는 구현하지 않는다. `{}`로 로그에 쓰려 하면 컴파일되지 않는다.
/// 값이 꼭 필요한 곳(DB 저장, 응답)에서만 [`Personal::expose`]로 꺼낸다.
///
/// docs/conventions.md: 회원 정보는 로그에 쓰지 않는다.
///
/// # Examples
///
/// ```
/// use baeum_domain::Personal;
///
/// let name = Personal::new(String::from("회원1"));
/// assert_eq!(format!("{name:?}"), "***");
/// assert_eq!(name.expose(), "회원1");
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Personal<T>(T);

impl<T> Personal<T> {
    /// 값을 감싼다.
    pub fn new(value: T) -> Self {
        Personal(value)
    }

    /// 감싼 값을 빌려준다.
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// 감싼 값을 꺼낸다.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Personal<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Holder {
        id: i64,
        name: Personal<String>,
    }

    #[test]
    fn debug_hides_value_inside_other_structs() {
        let holder = Holder {
            id: 7,
            name: Personal::new(String::from("회원1")),
        };

        let printed = format!("{holder:?}");

        assert!(printed.contains("id: 7"));
        assert!(printed.contains("name: ***"));
        assert!(!printed.contains("회원1"));
        assert_eq!(holder.id, 7);
        assert_eq!(holder.name.expose(), "회원1");
    }

    #[test]
    fn pretty_debug_also_hides_value() {
        let name = Personal::new(String::from("회원1"));

        assert_eq!(format!("{name:#?}"), "***");
    }

    #[test]
    fn into_inner_returns_value() {
        let phone = Personal::new(String::from("01000000001"));

        assert_eq!(phone.into_inner(), "01000000001");
    }
}
