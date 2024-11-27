//! BSD 3-Clause License
//!
//! Copyright (c) 2024, Marcus Cvjeticanin, Chase Willden
//!
//! Redistribution and use in source and binary forms, with or without
//! modification, are permitted provided that the following conditions are met:
//!
//! 1. Redistributions of source code must retain the above copyright notice, this
//!    list of conditions and the following disclaimer.
//!
//! 2. Redistributions in binary form must reproduce the above copyright notice,
//!    this list of conditions and the following disclaimer in the documentation
//!    and/or other materials provided with the distribution.
//!
//! 3. Neither the name of the copyright holder nor the names of its
//!    contributors may be used to endorse or promote products derived from
//!    this software without specific prior written permission.
//!
//! THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
//! AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
//! IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//! DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
//! FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//! DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//! SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
//! CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
//! OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
//! OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use delta_common::tensor_ops::Tensor;
use delta_common::{Activation, Layer, Shape};

#[derive(Debug)]
pub struct Dense {
    weights: Option<Tensor>,
    bias: Option<Tensor>,
    units: usize,
    activation: Box<dyn Activation>,
}

impl Dense {
    pub fn new<A: Activation + 'static>(units: usize, activation: A) -> Self {
        Self {
            weights: None,
            bias: None,
            units,
            activation: Box::new(activation),
        }
    }
}

impl Layer for Dense {
    fn build(&mut self, input_shape: Shape) {
        self.weights = Some(Tensor::random(&Shape::from((
            input_shape.len(),
            self.units,
        ))));

        self.bias = Some(Tensor::zeros(&Shape::new(vec![self.units])));
    }

    fn forward(&mut self, input: &Tensor) -> Tensor {
        let z = input
            .matmul(&self.weights.as_ref().unwrap())
            .add(&self.bias.as_ref().unwrap());

        self.activation.activate(&z)
    }

    fn backward(&mut self, grad: &Tensor) -> Tensor {
        let _ = grad;
        todo!()
    }
}

#[derive(Debug)]
pub struct Flatten {
    input_shape: Shape,
}

impl Flatten {
    pub fn new(input_shape: Shape) -> Self {
        Self { input_shape }
    }
}

impl Layer for Flatten {
    fn build(&mut self, input_shape: Shape) {
        self.input_shape = input_shape;
    }

    fn forward(&mut self, input: &Tensor) -> Tensor {
        // Flatten the input tensor by reshaping it to a 1D vector
        Tensor::new(input.data.clone(), Shape::new(vec![1, input.shape.len()]))
    }

    fn backward(&mut self, grad: &Tensor) -> Tensor {
        // Reshape the gradient back to the original input shape
        grad.reshape(self.input_shape.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_layer() {
        let input = Tensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], Shape::new(vec![2, 3]));
        let mut flatten_layer = Flatten::new(Shape::new(vec![2, 3]));
        let output = flatten_layer.forward(&input);
        assert_eq!(output.data, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(output.shape.0, vec![1, 6]);
    }
}
