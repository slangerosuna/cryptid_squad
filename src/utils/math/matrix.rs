pub struct Matrix4(pub [[f32; 4]; 4]);

impl std::ops::Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.0[i][0] * rhs.0[0][j] +
                               self.0[i][1] * rhs.0[1][j] +
                               self.0[i][2] * rhs.0[2][j] +
                               self.0[i][3] * rhs.0[3][j];
            }
        }

        Self(result)
    }
}
