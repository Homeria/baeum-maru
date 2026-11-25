//! 우선 배정 사유와 배정 방식(BR-S10, BR-S11, ADR 0017).

/// 우선 배정 사유 항목이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PriorityError {
    /// 사유 이름이 비었다.
    #[error("사유 이름을 입력해 주세요")]
    EmptyName,
    /// 사유 이름이 너무 길다.
    #[error("사유 이름은 30자까지 씁니다")]
    NameTooLong,
}

/// 우선 배정 사유 이름. 1~30자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityReasonName(String);

impl PriorityReasonName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 30;

    /// 입력한 글자에서 사유 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`PriorityError::EmptyName`], 30자를 넘으면 [`PriorityError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, PriorityError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(PriorityError::EmptyName),
            n if n > Self::MAX_CHARS => Err(PriorityError::NameTooLong),
            _ => Ok(PriorityReasonName(trimmed.to_owned())),
        }
    }

    /// 이름 글자. 관리자가 쓴 띄어쓰기 그대로다.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 겹침을 비교할 때 쓰는 글자. 띄어쓰기를 모두 뺀다.
    ///
    /// BR-S10: 띄어쓰기만 다른 이름은 같은 이름으로 본다.
    ///
    /// # Examples
    ///
    /// ```
    /// use baeum_domain::PriorityReasonName;
    ///
    /// let a = PriorityReasonName::parse("장애인 우대").unwrap();
    /// let b = PriorityReasonName::parse("장애인우대").unwrap();
    /// assert_eq!(a.key(), b.key());
    /// ```
    pub fn key(&self) -> String {
        self.0.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

/// 우선 배정 사유가 자리를 받는 방식(BR-S11).
///
/// 할당 정원, 정원 외 좌석 같은 방식은 아직 없다(ADR 0017).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityMethod {
    /// 자리 보장. 선발 없이 당첨되고 정원에 들어간다(BR-S3~S5).
    Guaranteed,
}

impl PriorityMethod {
    /// 모든 방식.
    pub const ALL: [PriorityMethod; 1] = [PriorityMethod::Guaranteed];

    /// DB와 API에 쓰는 글자.
    pub fn code(self) -> &'static str {
        match self {
            PriorityMethod::Guaranteed => "guaranteed",
        }
    }

    /// [`PriorityMethod::code`]의 글자에서 방식을 찾는다. 모르는 글자면 `None`.
    pub fn from_code(code: &str) -> Option<Self> {
        PriorityMethod::ALL
            .into_iter()
            .find(|method| method.code() == code)
    }
}

/// 확인을 통과한 새 사유. DB에 저장하기 전의 모양이다.
///
/// BR-S10: 이름 겹침은 DB가 [`PriorityReasonName::key`]로 막는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewPriorityReason {
    name: PriorityReasonName,
    method: PriorityMethod,
}

impl NewPriorityReason {
    /// 입력을 확인한다.
    ///
    /// # Errors
    ///
    /// 이름이 형식에 맞지 않으면 [`PriorityError`].
    pub fn parse(name: &str, method: PriorityMethod) -> Result<Self, PriorityError> {
        Ok(NewPriorityReason {
            name: PriorityReasonName::parse(name)?,
            method,
        })
    }

    /// 사유 이름.
    pub fn name(&self) -> &PriorityReasonName {
        &self.name
    }

    /// 배정 방식.
    pub fn method(&self) -> PriorityMethod {
        self.method
    }
}

#[cfg(test)]
mod tests;
