//! 강의실 항목과 형식 확인.

/// 강의실 항목이 형식에 맞지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RoomError {
    /// 강의실 이름이 비었다.
    #[error("강의실 이름을 입력해 주세요")]
    EmptyName,
    /// 강의실 이름이 너무 길다.
    #[error("강의실 이름은 30자까지 씁니다")]
    NameTooLong,
    /// 건물이 비었다. BR-R1: 강의실은 건물을 반드시 가진다.
    #[error("건물을 입력해 주세요")]
    EmptyBuilding,
    /// 건물 이름이 너무 길다.
    #[error("건물 이름은 20자까지 씁니다")]
    BuildingTooLong,
    /// 층이 고를 수 있는 범위 밖이다.
    #[error("층은 지하 2층부터 5층까지 고릅니다")]
    InvalidFloor,
}

/// 강의실 이름. 1~30자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomName(String);

impl RoomName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 30;

    /// 입력한 글자에서 강의실 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`RoomError::EmptyName`], 30자를 넘으면 [`RoomError::NameTooLong`].
    pub fn parse(input: &str) -> Result<Self, RoomError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(RoomError::EmptyName),
            n if n > Self::MAX_CHARS => Err(RoomError::NameTooLong),
            _ => Ok(RoomName(trimmed.to_owned())),
        }
    }

    /// 이름 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 건물 이름. 1~20자이고 앞뒤 공백은 지운다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingName(String);

impl BuildingName {
    /// 가장 긴 글자 수.
    pub const MAX_CHARS: usize = 20;

    /// 입력한 글자에서 건물 이름을 만든다.
    ///
    /// # Errors
    ///
    /// 비었으면 [`RoomError::EmptyBuilding`], 20자를 넘으면 [`RoomError::BuildingTooLong`].
    pub fn parse(input: &str) -> Result<Self, RoomError> {
        let trimmed = input.trim();
        match trimmed.chars().count() {
            0 => Err(RoomError::EmptyBuilding),
            n if n > Self::MAX_CHARS => Err(RoomError::BuildingTooLong),
            _ => Ok(BuildingName(trimmed.to_owned())),
        }
    }

    /// 건물 이름 글자.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 층. 지하 1층은 `-1`, 3층은 `3`이다. 0층은 없다.
///
/// 숫자로 두어 지하 1층 → 1층 → 2층 순서로 바르게 정렬된다.
/// 화면에서는 "지하 1층", "3층"으로 보여 준다.
///
/// # Examples
///
/// ```
/// use baeum_domain::Floor;
///
/// assert_eq!(Floor::new(-1).unwrap().value(), -1);
/// assert!(Floor::new(0).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Floor(i8);

impl Floor {
    /// 가장 낮은 층 (지하 2층).
    pub const LOWEST: i8 = -2;
    /// 가장 높은 층 (5층).
    pub const HIGHEST: i8 = 5;

    /// 층 번호에서 층을 만든다.
    ///
    /// # Errors
    ///
    /// 0이거나 지하 2층~5층 밖이면 [`RoomError::InvalidFloor`].
    pub fn new(value: i32) -> Result<Self, RoomError> {
        let value = i8::try_from(value).map_err(|_| RoomError::InvalidFloor)?;
        if value == 0 || !(Self::LOWEST..=Self::HIGHEST).contains(&value) {
            return Err(RoomError::InvalidFloor);
        }
        Ok(Floor(value))
    }

    /// 층 번호.
    pub fn value(self) -> i8 {
        self.0
    }
}

/// 화면에서 받은 새 강의실 입력. 확인 전이라 글자 그대로다.
#[derive(Debug, Clone, Copy)]
pub struct RoomInput<'a> {
    /// 강의실 이름.
    pub name: &'a str,
    /// 건물.
    pub building: &'a str,
    /// 층 번호. 선택.
    pub floor: Option<i32>,
}

/// 확인을 통과한 강의실. DB에 저장하기 전의 모양이다.
///
/// BR-R1: 건물은 반드시 있다. 같은 건물 안 이름 겹침은 DB가 막는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewRoom {
    name: RoomName,
    building: BuildingName,
    floor: Option<Floor>,
}

impl NewRoom {
    /// 입력을 모두 확인한다.
    ///
    /// # Errors
    ///
    /// 틀린 항목이 있으면 [`RoomError`]를 항목 순서(이름, 건물, 층)대로 **모두** 모아 돌려준다.
    pub fn parse(input: RoomInput<'_>) -> Result<Self, Vec<RoomError>> {
        let name = RoomName::parse(input.name);
        let building = BuildingName::parse(input.building);
        let floor = input.floor.map(Floor::new).transpose();

        match (name, building, floor) {
            (Ok(name), Ok(building), Ok(floor)) => Ok(NewRoom {
                name,
                building,
                floor,
            }),
            (name, building, floor) => Err([name.err(), building.err(), floor.err()]
                .into_iter()
                .flatten()
                .collect()),
        }
    }

    /// 강의실 이름.
    pub fn name(&self) -> &RoomName {
        &self.name
    }

    /// 건물.
    pub fn building(&self) -> &BuildingName {
        &self.building
    }

    /// 층.
    pub fn floor(&self) -> Option<Floor> {
        self.floor
    }
}

#[cfg(test)]
mod tests;
