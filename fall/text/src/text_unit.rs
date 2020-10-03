use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextUnit(pub(super) u32);

impl TextUnit {
    pub fn utf8_len(self) -> usize {
        self.0 as usize
    }
}

pub fn tu(value: u32) -> TextUnit {
    TextUnit(value)
}

impl From<TextUnit> for u32 {
    fn from(tu: TextUnit) -> u32 {
        tu.0
    }
}

impl ops::Add<u32> for TextUnit {
    type Output = TextUnit;
    fn add(self, rhs: u32) -> TextUnit {
        TextUnit(self.0 + rhs)
    }
}

impl ops::Add<TextUnit> for TextUnit {
    type Output = TextUnit;
    fn add(self, rhs: TextUnit) -> TextUnit {
        TextUnit(self.0 + rhs.0)
    }
}

impl ops::AddAssign<TextUnit> for TextUnit {
    fn add_assign(&mut self, rhs: TextUnit) {
        self.0 += rhs.0
    }
}

impl ::std::iter::Sum for TextUnit {
    fn sum<I: Iterator<Item = TextUnit>>(iter: I) -> Self {
        TextUnit(iter.map(|u| u.0).sum())
    }
}

impl ops::Sub<u32> for TextUnit {
    type Output = TextUnit;
    fn sub(self, rhs: u32) -> TextUnit {
        TextUnit(self.0 - rhs)
    }
}

impl ops::Sub<TextUnit> for TextUnit {
    type Output = TextUnit;
    fn sub(self, rhs: TextUnit) -> TextUnit {
        TextUnit(self.0 - rhs.0)
    }
}

impl ops::SubAssign<TextUnit> for TextUnit {
    fn sub_assign(&mut self, rhs: TextUnit) {
        self.0 -= rhs.0
    }
}

impl fmt::Debug for TextUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Self as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Display for TextUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for TextUnit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TextUnit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Deserialize::deserialize(deserializer)?;
        Ok(TextUnit(value))
    }
}

impl ::rand::Rand for TextUnit {
    fn rand<R: ::rand::Rng>(rng: &mut R) -> TextUnit {
        TextUnit(u32::rand(rng))
    }
}
