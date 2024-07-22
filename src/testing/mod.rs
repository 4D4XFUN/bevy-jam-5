#[macro_export]
macro_rules! assert_vec2_close {
    ($a:expr, $b:expr) => {
        assert_vec2_close!($a, $b, 1e-6)
    };
    ($a:expr, $b:expr, $epsilon:expr) => {{
        let diff = $a - $b;
        _ = (diff.x.abs() < $epsilon) && (diff.y.abs() < $epsilon)
    }};
}
