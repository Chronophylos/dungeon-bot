use rand::{
    distributions::{Distribution, Uniform},
    RngCore,
};

#[cfg(debug_assertions)]
#[allow(dead_code)]
fn rng() -> Box<dyn RngCore> {
    use rand::SeedableRng;

    Box::new(rand_chacha::ChaCha20Rng::from_seed([
        0x46, 0x65, 0x65, 0x6c, 0x73, 0x44, 0x61, 0x6e, 0x6b, 0x4d, 0x61, 0x6e, 0x4b, 0x61, 0x70,
        0x70, 0x61, 0x31, 0x32, 0x33, 0x46, 0x65, 0x65, 0x6c, 0x73, 0x44, 0x61, 0x6e, 0x6b, 0x4d,
        0x61, 0x6e,
    ]))
}

#[cfg(not(debug_assertions))]
#[allow(dead_code)]
fn rng() -> Box<dyn RngCore> {
    Box::new(rand::thread_rng())
}

pub trait Dice<D>
where
    D: Distribution<i16>,
{
    /// The average roll
    fn avg() -> f32;

    /// The minimum roll
    fn min() -> i16;

    /// The maximum roll
    fn max() -> i16;

    fn distribution() -> D;

    /// Roll a single time
    fn roll_once() -> i16 {
        Self::distribution().sample(&mut rng())
    }

    /// Roll multiple times
    fn roll(n: usize) -> Vec<i16> {
        let dist = Self::distribution();
        let mut rng = rng();

        vec![0; n]
            .into_iter()
            .map(|_| dist.sample(&mut rng))
            .collect()
    }
}

macro_rules! impl_dice {
    ($dice:tt, $min:expr, $max:expr) => {
        pub struct $dice;
        impl Dice<Uniform<i16>> for $dice {
            fn min() -> i16 {
                $min
            }

            fn max() -> i16 {
                $max
            }

            fn avg() -> f32 {
                f32::from($min + $max) / 2.0
            }

            fn distribution() -> Uniform<i16> {
                Uniform::from($min..=$max)
            }
        }
    };
}

impl_dice!(D6,  1i16, 6i16);
impl_dice!(D10, 1i16, 10i16);
impl_dice!(D20, 1i16, 20i16);

#[cfg(all(test, debug_assertions))]
mod test {
    use super::*;

    #[test]
    fn is_rng_seeded() {
        let mut rng = rng();
        assert_eq!(rng.next_u64(), 15084443005315021354);
        assert_eq!(rng.next_u64(), 13672135499988464561);
        assert_eq!(rng.next_u64(), 4380429139279809442);
    }


    mod d6 {
        use super::*;

        #[test]
        fn roll20() {
            assert_eq!(
                D6::roll(20),
                vec![1, 5, 4, 5, 6, 2, 3, 6, 4, 4, 6, 1, 1, 2, 2, 5, 2, 4, 4, 3]
            )
        }

        #[test]
        fn min() {
            assert_eq!(D6::min(), 1);
        }

        #[test]
        fn max() {
            assert_eq!(D6::max(), 6);
        }

        #[test]
        fn avg() {
            assert_eq!(D6::avg(), 3.5);
        }
    }

    mod d10 {
        use super::*;

        #[test]
        fn roll20() {
            assert_eq!(
                D10::roll(20),
                vec![1, 9, 6, 8, 10, 3, 5, 9, 7, 7, 9, 1, 1, 3, 3, 8, 3, 6, 7, 4]
            )
        }

        #[test]
        fn min() {
            assert_eq!(D10::min(), 1);
        }

        #[test]
        fn max() {
            assert_eq!(D10::max(), 10);
        }

        #[test]
        fn avg() {
            assert_eq!(D10::avg(), 5.5);
        }
    }

    mod d20 {
        use super::*;

        #[test]
        fn roll20() {
            assert_eq!(
                D20::roll(20),
                [2, 17, 12, 15, 20, 5, 9, 17, 14, 13, 17, 1, 1, 6, 6, 15, 6, 12, 14, 8]
            )
        }

        #[test]
        fn min() {
            assert_eq!(D20::min(), 1);
        }

        #[test]
        fn max() {
            assert_eq!(D20::max(), 20);
        }

        #[test]
        fn avg() {
            assert_eq!(D20::avg(), 10.5);
        }
    }
}
