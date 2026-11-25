//! 학기 항목과 형식 확인.

use time::Date;

/// 학기 항목이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TermError {
    /// 학기 이름이 비었다.
    #[error("학기 이름을 입력해 주세요")]
    EmptyName,
    /// 학기 이름이 너무 길다.
    #[error("학기 이름은 30자까지 씁니다")]
    NameTooLong,
    /// 시작일이 없다.
    #[error("시작일을 입력해 주세요")]
    MissingStart,
    /// 종료일이 없다.
    #[error("종료일을 입력해 주세요")]
    MissingEnd,
    /// BR-T2: 종료일이 시작일보다 앞선다.
    #[error("종료일은 시작일과 같거나 뒤여야 합니다")]
    EndBeforeStart,
}

/// 학기 이름. 1~30자이고 앞뒤 공백은 지운다. 예: 2027년 1학기
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermName(String);

impl TermName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 30;

    /// 입력한 글자에서 학기 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`TermError::EmptyName`], 30자를 넘으면 [`TermError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, TermError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(TermError::EmptyName),
            n if n > Self::MAX_CHARS => Err(TermError::NameTooLong),
            _ => Ok(TermName(trimmed.to_owned())),
        }
    }

    /// 이름 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 학기 기간. 시작일과 종료일을 포함한다.
///
/// BR-T2: 종료일은 시작일보다 앞설 수 없다. 하루짜리 학기도 된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermPeriod {
    starts_on: Date,
    ends_on: Date,
}

impl TermPeriod {
    /// 시작일과 종료일로 기간을 만든다.
    ///
    /// # Errors
    ///
    /// 종료일이 시작일보다 앞서면 [`TermError::EndBeforeStart`].
    pub fn new(starts_on: Date, ends_on: Date) -> Result<Self, TermError> {
        if ends_on < starts_on {
            return Err(TermError::EndBeforeStart);
        }
        Ok(TermPeriod { starts_on, ends_on })
    }

    /// 시작일.
    pub fn starts_on(self) -> Date {
        self.starts_on
    }

    /// 종료일.
    pub fn ends_on(self) -> Date {
        self.ends_on
    }
}

/// 화면에서 받은 학기 입력. 확인 전이다. 날짜 칸을 비웠으면 `None`.
#[derive(Debug, Clone, Copy)]
pub struct TermInput<'a> {
    /// 학기 이름.
    pub name: &'a str,
    /// 시작일.
    pub starts_on: Option<Date>,
    /// 종료일.
    pub ends_on: Option<Date>,
}

/// 확인을 통과한 학기. DB에 저장하기 전의 모양이다.
///
/// BR-T1: 이름 겹침은 DB가 막는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTerm {
    name: TermName,
    period: TermPeriod,
}

impl NewTerm {
    /// 입력을 모두 확인한다.
    ///
    /// # Errors
    ///
    /// 틀린 항목이 있으면 [`TermError`]를 항목 순서(이름, 시작일, 종료일)대로 **모두** 모아 돌려준다.
    /// 두 날짜가 다 있을 때만 앞뒤를 비교한다.
    pub fn parse(input: TermInput<'_>) -> Result<Self, Vec<TermError>> {
        let name = TermName::parse(input.name);
        let period = match (input.starts_on, input.ends_on) {
            (Some(starts_on), Some(ends_on)) => {
                TermPeriod::new(starts_on, ends_on).map_err(|error| vec![error])
            }
            (starts_on, ends_on) => Err([
                starts_on.is_none().then_some(TermError::MissingStart),
                ends_on.is_none().then_some(TermError::MissingEnd),
            ]
            .into_iter()
            .flatten()
            .collect()),
        };

        match (name, period) {
            (Ok(name), Ok(period)) => Ok(NewTerm { name, period }),
            (name, period) => Err(name
                .err()
                .into_iter()
                .chain(period.err().into_iter().flatten())
                .collect()),
        }
    }

    /// 학기 이름.
    pub fn name(&self) -> &TermName {
        &self.name
    }

    /// 학기 기간.
    pub fn period(&self) -> TermPeriod {
        self.period
    }
}

#[cfg(test)]
mod tests;
