//! 과목 항목과 형식 확인.

/// 과목 항목이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SubjectError {
    /// 과목 이름이 비었다.
    #[error("과목 이름을 입력해 주세요")]
    EmptyName,
    /// 과목 이름이 너무 길다.
    #[error("과목 이름은 30자까지 씁니다")]
    NameTooLong,
    /// 분류 이름이 너무 길다.
    #[error("분류 이름은 20자까지 씁니다")]
    CategoryTooLong,
}

/// 과목 이름. 1~30자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectName(String);

impl SubjectName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 30;

    /// 입력한 글자에서 과목 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`SubjectError::EmptyName`], 30자를 넘으면 [`SubjectError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, SubjectError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(SubjectError::EmptyName),
            n if n > Self::MAX_CHARS => Err(SubjectError::NameTooLong),
            _ => Ok(SubjectName(trimmed.to_owned())),
        }
    }

    /// 이름 글자. 직원이 쓴 띄어쓰기 그대로다.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 겹침을 비교할 때 쓰는 글자. 띄어쓰기를 모두 뺀다.
    ///
    /// BR-J1: 띄어쓰기만 다른 이름은 같은 이름으로 본다.
    ///
    /// # Examples
    ///
    /// ```
    /// use baeum_domain::SubjectName;
    ///
    /// let a = SubjectName::parse("한글교실").unwrap();
    /// let b = SubjectName::parse("한글 교실").unwrap();
    /// assert_eq!(a.key(), b.key());
    /// ```
    pub fn key(&self) -> String {
        self.0.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

/// 분류 이름. 1~20자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryName(String);

impl CategoryName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 20;

    /// 입력한 글자에서 분류 이름을 만든다. 분류는 선택이라 비었으면 `None`.
    ///
    /// # Errors
    ///
    /// 20자를 넘으면 [`SubjectError::CategoryTooLong`].
    pub fn parse(input: &str) -> Result<Option<Self>, SubjectError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Ok(None),
            n if n > Self::MAX_CHARS => Err(SubjectError::CategoryTooLong),
            _ => Ok(Some(CategoryName(trimmed.to_owned()))),
        }
    }

    /// 분류 이름 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 화면에서 받은 과목 입력. 확인 전이라 글자 그대로다.
#[derive(Debug, Clone, Copy)]
pub struct SubjectInput<'a> {
    /// 과목 이름.
    pub name: &'a str,
    /// 분류. 선택.
    pub category: Option<&'a str>,
}

/// 확인을 통과한 과목. DB에 저장하기 전의 모양이다.
///
/// BR-J1: 이름 겹침은 DB가 [`SubjectName::key`]로 막는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSubject {
    name: SubjectName,
    category: Option<CategoryName>,
}

impl NewSubject {
    /// 입력을 모두 확인한다.
    ///
    /// # Errors
    ///
    /// 틀린 항목이 있으면 [`SubjectError`]를 항목 순서(이름, 분류)대로 **모두** 모아 돌려준다.
    pub fn parse(input: SubjectInput<'_>) -> Result<Self, Vec<SubjectError>> {
        let name = SubjectName::parse(input.name);
        let category = input.category.map_or(Ok(None), CategoryName::parse);

        match (name, category) {
            (Ok(name), Ok(category)) => Ok(NewSubject { name, category }),
            (name, category) => Err([name.err(), category.err()].into_iter().flatten().collect()),
        }
    }

    /// 과목 이름.
    pub fn name(&self) -> &SubjectName {
        &self.name
    }

    /// 분류.
    pub fn category(&self) -> Option<&CategoryName> {
        self.category.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_trims_spaces() {
        assert_eq!(SubjectName::parse(" 요가 ").unwrap().as_str(), "요가");
    }

    #[test]
    fn name_rejects_empty_and_too_long() {
        assert_eq!(SubjectName::parse("  "), Err(SubjectError::EmptyName));
        assert!(SubjectName::parse(&"가".repeat(30)).is_ok());
        assert_eq!(
            SubjectName::parse(&"가".repeat(31)),
            Err(SubjectError::NameTooLong)
        );
    }

    #[test]
    fn name_keeps_inner_spaces_as_typed() {
        assert_eq!(
            SubjectName::parse("한글 교실").unwrap().as_str(),
            "한글 교실"
        );
    }

    #[test]
    fn br_j1_key_ignores_every_space() {
        let key = |name: &str| SubjectName::parse(name).unwrap().key();

        assert_eq!(key("한글교실"), key("한글 교실"));
        assert_eq!(key("스마트폰 초급 반"), "스마트폰초급반");
    }

    #[test]
    fn br_j1_key_tells_different_names_apart() {
        let key = |name: &str| SubjectName::parse(name).unwrap().key();

        assert_ne!(key("스마트폰"), key("스마트폰 활용"));
    }

    #[test]
    fn category_is_optional() {
        assert_eq!(CategoryName::parse(""), Ok(None));
        assert_eq!(CategoryName::parse("   "), Ok(None));
    }

    #[test]
    fn category_trims_and_limits_length() {
        assert_eq!(
            CategoryName::parse(" 평생교육 ").unwrap().unwrap().as_str(),
            "평생교육"
        );
        assert!(CategoryName::parse(&"가".repeat(20)).unwrap().is_some());
        assert_eq!(
            CategoryName::parse(&"가".repeat(21)),
            Err(SubjectError::CategoryTooLong)
        );
    }

    #[test]
    fn parse_keeps_category() {
        let subject = NewSubject::parse(SubjectInput {
            name: "요가",
            category: Some("취미여가"),
        })
        .unwrap();

        assert_eq!(subject.name().as_str(), "요가");
        assert_eq!(
            subject.category().map(CategoryName::as_str),
            Some("취미여가")
        );
    }

    #[test]
    fn missing_or_blank_category_is_none() {
        for category in [None, Some(""), Some("  ")] {
            let subject = NewSubject::parse(SubjectInput {
                name: "요가",
                category,
            })
            .unwrap();

            assert_eq!(subject.category(), None, "{category:?}");
        }
    }

    #[test]
    fn collects_every_error_in_field_order() {
        let long = "가".repeat(21);
        let errors = NewSubject::parse(SubjectInput {
            name: "",
            category: Some(&long),
        })
        .unwrap_err();

        assert_eq!(
            errors,
            [SubjectError::EmptyName, SubjectError::CategoryTooLong]
        );
    }
}
