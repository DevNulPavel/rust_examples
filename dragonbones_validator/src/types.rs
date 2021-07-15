
#[derive(Debug)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

impl std::ops::Mul<u32> for ImageSize {
    type Output = ImageSize;
    fn mul(self, rhs: u32) -> Self::Output {
        ImageSize{
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}