use zkprov_corelib::gadgets::range::{range_check_slice_u64, range_check_u64};

#[test]
fn range_ok_and_fail() {
    range_check_u64(15, 4).unwrap();
    assert!(range_check_u64(16, 4).is_err());
    range_check_u64(u64::MAX, 64).unwrap();
    assert!(range_check_u64(u64::MAX, 63).is_err());
}

#[test]
fn range_batch() {
    let xs = [0, 1, 2, 3, 7];
    range_check_slice_u64(&xs, 3).unwrap();
    let ys = [0, 8];
    assert!(range_check_slice_u64(&ys, 3).is_err());
}
