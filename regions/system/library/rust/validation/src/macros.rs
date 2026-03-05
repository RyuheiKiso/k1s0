#[macro_export]
macro_rules! validate {
    ($errors:expr, $($validator:expr),+ $(,)?) => {
        $(
            if let Err(err) = $validator {
                $errors.add(err);
            }
        )+
    };
}
