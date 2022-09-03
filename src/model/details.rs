lazy_static::lazy_static! {
    static ref SIZE_POSTFIX: Vec<String> = {
        let mut vec = Vec::new();
        vec.push("B".to_string());
        vec.push("K".to_string());
        vec.push("M".to_string());
        vec.push("G".to_string());
        vec.push("T".to_string());
        vec.push("P".to_string());
        vec.push("X".to_string());
        vec
    };
}

pub fn human_readable_size(size_in_bytes: u64) -> (f64, String) {
    const UPPER_LIMIT_LEFT: f64 = 1000.0;
    const UPPER_LIMIT_RIGHT: f64 = 100.0;

    let mut size = size_in_bytes as f64;
    let mut i = 0;

    if size < UPPER_LIMIT_LEFT {
        return (size, SIZE_POSTFIX.get(0).unwrap().into());
    } else {
        while size >= UPPER_LIMIT_LEFT {
            i += 1;
            size /= 1024.0;
        }
    }

    (
        f64::round(size * UPPER_LIMIT_RIGHT) / UPPER_LIMIT_RIGHT,
        SIZE_POSTFIX.get(i).unwrap().into(),
    )
}

#[cfg(test)]
mod tests {

    #[test]
    fn human_readable_size() {
        assert_eq!((999_f64, "B".into()), super::human_readable_size(999));
        assert_eq!((0.98, "K".into()), super::human_readable_size(1000));
        assert_eq!((999_f64, "K".into()), super::human_readable_size(1022976));
        assert_eq!((0.98, "M".into()), super::human_readable_size(1024000));
        assert_eq!(
            (999_f64, "M".into()),
            super::human_readable_size(1047527424)
        );
        assert_eq!((0.98, "G".into()), super::human_readable_size(1048576000));
        assert_eq!(
            (16.0, "X".into()),
            super::human_readable_size(u64::max_value())
        );
    }
}
