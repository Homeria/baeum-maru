use super::*;

fn capacity(value: i64) -> Capacity {
    Capacity::new(value).expect("1~200")
}

#[test]
fn br_s4_guaranteed_take_seats_and_rest_are_drawn_for_remaining() {
    let seed = LotterySeed::new(0x0123_4567_89AB_CDEF);
    let others: Vec<i64> = (1..=10).collect();

    let allocation = allocate(&[30, 20], &others, capacity(6), seed, LotteryVersion::V1).unwrap();

    assert_eq!(allocation.guaranteed(), [20, 30]);
    // 나머지는 정원 4명짜리 추첨과 같다 (ADR 0017: 섞는 방법은 버전 1 그대로)
    let plain = draw_lottery(&others, capacity(4), seed, LotteryVersion::V1);
    assert_eq!(allocation.draw(), &plain);
    assert_eq!(allocation.draw().selected(), [10, 4, 1, 8]);
}

#[test]
fn br_s4_without_guaranteed_is_same_as_plain_draw() {
    let seed = LotterySeed::new(7);
    let others: Vec<i64> = (1..=8).collect();

    let allocation = allocate(&[], &others, capacity(5), seed, LotteryVersion::V1).unwrap();

    assert_eq!(
        allocation.draw(),
        &draw_lottery(&others, capacity(5), seed, LotteryVersion::V1)
    );
}

#[test]
fn br_s4_guaranteed_filling_capacity_leaves_others_waitlisted() {
    let allocation = allocate(
        &[1, 2],
        &[3, 4, 5],
        capacity(2),
        LotterySeed::new(1),
        LotteryVersion::V1,
    )
    .unwrap();

    assert!(allocation.draw().selected().is_empty());
    assert_eq!(allocation.draw().waitlisted().len(), 3);
}

#[test]
fn br_s5_more_guaranteed_than_capacity_is_refused() {
    let result = allocate(
        &[1, 2, 3],
        &[4],
        capacity(2),
        LotterySeed::new(1),
        LotteryVersion::V1,
    );

    assert_eq!(
        result,
        Err(OverCapacity {
            guaranteed: 3,
            capacity: 2
        })
    );
}

#[test]
fn splitmix64_matches_published_values() {
    // 알고리즘을 만든 쪽이 공개한 시드 1234567의 처음 다섯 값
    let mut random = SplitMix64::new(1_234_567);
    let values: Vec<u64> = (0..5).map(|_| random.next()).collect();
    assert_eq!(
        values,
        [
            6_457_827_717_110_365_317,
            3_203_168_211_198_807_973,
            9_817_491_932_198_370_423,
            4_593_380_528_125_082_431,
            16_408_922_859_458_223_821,
        ]
    );
}

#[test]
fn br_l2_version_1_result_never_changes() {
    // 이 값이 바뀌면 옛 추첨 기록을 다시 만들 수 없다. 방식을 바꾸려면 새 버전을 더한다
    let ids: Vec<i64> = (1..=10).collect();
    let result = draw_lottery(
        &ids,
        capacity(6),
        LotterySeed::new(0x0123_4567_89AB_CDEF),
        LotteryVersion::V1,
    );
    assert_eq!(result.order(), [10, 4, 1, 8, 2, 3, 5, 7, 9, 6]);
    assert_eq!(result.selected(), [10, 4, 1, 8, 2, 3]);
    assert_eq!(result.waitlisted(), [5, 7, 9, 6]);
}

#[test]
fn br_l2_given_order_does_not_change_result() {
    let seed = LotterySeed::new(42);
    let sorted = draw_lottery(&[10, 20, 30], capacity(1), seed, LotteryVersion::V1);
    let mixed = draw_lottery(&[30, 10, 20], capacity(1), seed, LotteryVersion::V1);
    assert_eq!(sorted, mixed);
    assert_eq!(sorted.order(), [10, 30, 20]);
}

#[test]
fn br_s7_everyone_is_either_selected_or_waitlisted() {
    let ids: Vec<i64> = (1..=30).collect();
    let result = draw_lottery(&ids, capacity(20), LotterySeed::new(7), LotteryVersion::V1);
    assert_eq!(result.selected().len(), 20);
    assert_eq!(result.waitlisted().len(), 10);
    let mut everyone = result.order().to_vec();
    everyone.sort_unstable();
    assert_eq!(everyone, ids);
}

#[test]
fn br_s6_everyone_selected_when_not_over_capacity() {
    let result = draw_lottery(
        &[3, 1, 2],
        capacity(3),
        LotterySeed::new(7),
        LotteryVersion::V1,
    );
    assert_eq!(result.selected().len(), 3);
    assert!(result.waitlisted().is_empty());

    let empty = draw_lottery(&[], capacity(3), LotterySeed::new(7), LotteryVersion::V1);
    assert!(empty.order().is_empty());
}

#[test]
fn every_position_is_equally_likely() {
    // 신청 3건을 시드 0..30000으로 추첨하면 첫 자리는 셋이 각각 약 1만 번
    let mut first_counts = [0_u32; 3];
    for seed in 0..30_000 {
        let result = draw_lottery(
            &[0, 1, 2],
            capacity(1),
            LotterySeed::new(seed),
            LotteryVersion::V1,
        );
        let first = usize::try_from(result.selected()[0]).expect("0~2");
        first_counts[first] += 1;
    }
    for count in first_counts {
        assert!((9_500..=10_500).contains(&count), "{first_counts:?}");
    }
}

#[test]
fn below_never_reaches_bound() {
    let mut random = SplitMix64::new(1);
    for bound in 1..50 {
        for _ in 0..100 {
            assert!(random.below(bound) < bound);
        }
    }
}

#[test]
fn seed_round_trips_as_16_hex_digits() {
    let seed = LotterySeed::new(0x0123_4567_89AB_CDEF);
    assert_eq!(seed.to_hex(), "0123456789abcdef");
    assert_eq!(LotterySeed::from_hex("0123456789abcdef"), Some(seed));
    assert_eq!(LotterySeed::new(u64::MAX).to_hex(), "ffffffffffffffff");
    assert_eq!(LotterySeed::from_hex("123"), None);
    assert_eq!(LotterySeed::from_hex("+123456789abcdef"), None);
    assert_eq!(LotterySeed::from_hex("zzzzzzzzzzzzzzzz"), None);
}

#[test]
fn version_number_round_trips() {
    assert_eq!(LotteryVersion::CURRENT.number(), 1);
    assert_eq!(LotteryVersion::from_number(1), Some(LotteryVersion::V1));
    assert_eq!(LotteryVersion::from_number(2), None);
}
