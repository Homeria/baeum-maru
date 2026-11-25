//! 추첨(BR-S6, BR-S7)과 추첨 방식(ADR 0016).
//!
//! 시드를 받아 신청의 순서를 정하는 순수한 계산만 둔다. 시드를 만드는 일(OS 난수)과
//! 기록은 앱 쪽이 한다. 같은 대상, 시드, 버전이면 언제나 같은 결과다(BR-L2).
//!
//! 우선 배정은 자리 보장(BR-S3~S5)만 있다. 대기 인원 상한(BR-S9)은 아직 없다.

use crate::course::Capacity;

/// 추첨의 난수 시드. 기록에는 16자리 16진수 글자로 남긴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotterySeed(u64);

impl LotterySeed {
    /// 숫자에서 시드를 만든다. 실제 추첨에서는 OS 난수를 넣는다.
    pub fn new(value: u64) -> Self {
        LotterySeed(value)
    }

    /// 기록에 남기는 글자. 예: `0123456789abcdef`
    pub fn to_hex(self) -> String {
        format!("{:016x}", self.0)
    }

    /// [`LotterySeed::to_hex`]의 글자에서 시드를 읽는다. 16자리 16진수가 아니면 `None`.
    pub fn from_hex(text: &str) -> Option<Self> {
        // from_str_radix는 앞의 `+`도 받아서 글자를 직접 확인한다
        if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        u64::from_str_radix(text, 16).ok().map(LotterySeed)
    }
}

/// 추첨 방식의 버전(ADR 0016). 방식을 바꾸면 새 버전을 더하고 옛 버전은 지우지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LotteryVersion {
    /// 신청 id 순 → SplitMix64 → 피셔-예이츠 섞기.
    V1,
}

impl LotteryVersion {
    /// 새 추첨에 쓰는 버전.
    pub const CURRENT: LotteryVersion = LotteryVersion::V1;

    /// 기록에 남기는 번호.
    pub fn number(self) -> i64 {
        match self {
            LotteryVersion::V1 => 1,
        }
    }

    /// [`LotteryVersion::number`]의 번호에서 버전을 찾는다. 모르는 번호면 `None`.
    pub fn from_number(number: i64) -> Option<Self> {
        match number {
            1 => Some(LotteryVersion::V1),
            _ => None,
        }
    }
}

/// 추첨 한 번의 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotteryDraw {
    order: Vec<i64>,
    selected_count: usize,
}

impl LotteryDraw {
    /// 뽑힌 순서대로 모든 신청 id. 화면의 신청자 목록이 이 순서로 멈춘다.
    pub fn order(&self) -> &[i64] {
        &self.order
    }

    /// 당첨된 신청. 순서의 앞에서 정원만큼.
    pub fn selected(&self) -> &[i64] {
        &self.order[..self.selected_count]
    }

    /// 대기가 된 신청. 앞에서부터 대기 순번 1, 2, 3…이다.
    pub fn waitlisted(&self) -> &[i64] {
        &self.order[self.selected_count..]
    }
}

/// 신청들을 추첨한다(BR-S7). 신청자가 정원 이하이면 모두 당첨이다(BR-S6).
///
/// `application_ids`의 순서는 결과에 영향을 주지 않는다. 방식이 먼저 id 순으로 늘어놓는다.
pub fn draw_lottery(
    application_ids: &[i64],
    capacity: Capacity,
    seed: LotterySeed,
    version: LotteryVersion,
) -> LotteryDraw {
    draw_seats(
        application_ids,
        usize::from(capacity.value()),
        seed,
        version,
    )
}

/// `seats`자리를 추첨한다. 자리 보장을 빼고 남은 자리는 0일 수 있어 [`Capacity`]가 아닌 수를 받는다.
fn draw_seats(
    application_ids: &[i64],
    seats: usize,
    seed: LotterySeed,
    version: LotteryVersion,
) -> LotteryDraw {
    let mut order = application_ids.to_vec();
    order.sort_unstable();
    match version {
        LotteryVersion::V1 => shuffle_v1(&mut order, seed),
    }
    let selected_count = order.len().min(seats);
    LotteryDraw {
        order,
        selected_count,
    }
}

/// 자리 보장 신청이 정원보다 많아 추첨할 수 없다(BR-S5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("자리 보장 신청 {guaranteed}건이 정원 {capacity}명보다 많습니다")]
pub struct OverCapacity {
    /// 자리 보장 신청 수.
    pub guaranteed: usize,
    /// 강좌 정원.
    pub capacity: u16,
}

/// 자리 보장과 추첨을 합친 선발 결과.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allocation {
    guaranteed: Vec<i64>,
    draw: LotteryDraw,
}

impl Allocation {
    /// 자리 보장으로 당첨된 신청. id 순서다.
    pub fn guaranteed(&self) -> &[i64] {
        &self.guaranteed
    }

    /// 나머지 신청의 추첨 결과. 남은 자리만큼 당첨이다.
    pub fn draw(&self) -> &LotteryDraw {
        &self.draw
    }
}

/// 자리 보장 신청을 먼저 당첨시키고, 나머지 신청으로 남은 자리를 추첨한다.
///
/// BR-S4: 남은 자리 = 정원 - 자리 보장 신청 수. 남은 자리가 0이면 나머지는 모두 대기다.
/// 추첨은 [`draw_lottery`]와 같은 방식이다(ADR 0017). 두 목록에 같은 신청이 있으면 안 된다.
///
/// # Errors
///
/// BR-S5: 자리 보장 신청이 정원보다 많으면 [`OverCapacity`].
pub fn allocate(
    guaranteed_ids: &[i64],
    other_ids: &[i64],
    capacity: Capacity,
    seed: LotterySeed,
    version: LotteryVersion,
) -> Result<Allocation, OverCapacity> {
    let seats = usize::from(capacity.value());
    let remaining = seats
        .checked_sub(guaranteed_ids.len())
        .ok_or(OverCapacity {
            guaranteed: guaranteed_ids.len(),
            capacity: capacity.value(),
        })?;
    let mut guaranteed = guaranteed_ids.to_vec();
    guaranteed.sort_unstable();
    Ok(Allocation {
        guaranteed,
        draw: draw_seats(other_ids, remaining, seed, version),
    })
}

/// 버전 1의 섞기: 맨 끝 자리부터 앞쪽의 아무 자리와 바꾼다(피셔-예이츠).
fn shuffle_v1(items: &mut [i64], seed: LotterySeed) {
    let mut random = SplitMix64::new(seed.0);
    for index in (1..items.len()).rev() {
        // index는 슬라이스 길이보다 작아 u64로도 usize로도 잃는 값이 없다
        let bound = index as u64 + 1;
        let other = random.below(bound) as usize;
        items.swap(index, other);
    }
}

/// SplitMix64 생성기(Steele, Lea, Flood 2014). 짧고 공개된 알고리즘이라 직접 둔다.
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `0..bound`에서 고르게 하나. `bound`는 1 이상.
    ///
    /// 그냥 `next() % bound`이면 앞쪽 값이 조금 더 자주 나온다. 넘치는 구간의 값을
    /// 버리고 다시 뽑는다(거부 샘플링).
    fn below(&mut self, bound: u64) -> u64 {
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let value = self.next();
            if value >= threshold {
                return value % bound;
            }
        }
    }
}

#[cfg(test)]
mod tests;
