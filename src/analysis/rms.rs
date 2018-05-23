use analysis::traits::Chainable;

pub struct RMS {
    buffer: Vec<f32>,
}

impl RMS {
    pub fn new() -> RMS {
        RMS { buffer: Vec::new() }
    }
}

impl Chainable for RMS {
    fn update(&mut self, buffer: &Vec<Vec<f32>>) {

        let mut rms = 0f32;
        for i in 0..buffer.len()
        {
            let mut square_sum = 0.0f32;
            for x in 0..buffer[i].len() {
                square_sum += buffer[i][x] * buffer[i][x];
            }

            let square_mean = square_sum * 1.0f32 / buffer.len() as f32;

            rms += f32::sqrt(square_mean);
        }

        rms /= buffer.len() as f32;

        self.buffer = Vec::new();
        self.buffer.push(rms);
    }

    fn output(&self) -> &Vec<f32> {
        &self.buffer
    }
}
