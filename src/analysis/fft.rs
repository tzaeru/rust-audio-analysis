extern crate rustfft;
use self::rustfft::FFTplanner;
use self::rustfft::num_complex::Complex;
use self::rustfft::num_traits::Zero;

use analysis::traits::Chainable;

pub struct FFT {
    buffer: Vec<f32>,
    planner: FFTplanner<f32>,
    cache_buffer: Vec<f32>
}

impl FFT {
    pub fn new() -> FFT {
        FFT { buffer: Vec::new(), cache_buffer: Vec::new(), planner: FFTplanner::new(false)}
    }
}

impl Chainable for FFT {
    fn update(&mut self, buffer: &Vec<Vec<f32>>, samplerate: u32) {
        // We add the current sample to the private buffer.
        for i in 0..buffer[0].len()
        {
            self.cache_buffer.push(buffer[0][i]);
        }

        // C2 in Hz is ~65. We divide by 2 to make sure that no matter the offset,
        // frequencies of that wavelength fit for the FFT analysis. 
        if self.cache_buffer.len() > (samplerate/(65/2)) as usize
        {
            self.buffer = Vec::new();

            let mut input: Vec<Complex<f32>> = vec![Complex::zero(); self.cache_buffer.len()];
            for i in 0..self.cache_buffer.len()
            {
                input[i] = Complex::new(self.cache_buffer[i], 0f32);
            }

            let mut output: Vec<Complex<f32>> = vec![Complex::zero(); self.cache_buffer.len()];
            let fft = self.planner.plan_fft(self.cache_buffer.len());
            fft.process(&mut input, &mut output);

            for i in 0..output.len()
            {
                self.buffer.push(output[i].re);
            } 

            self.cache_buffer = Vec::new();
        }
    }

    fn output(&self) -> &Vec<f32> {
        &self.buffer
    }
}
