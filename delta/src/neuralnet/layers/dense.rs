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

use crate::common::{Activation, Layer, Optimizer, Shape, Tensor};
use log::debug;
use serde_json;

/// A dense (fully connected) layer.
#[derive(Debug)]
pub struct Dense {
    name: String,
    weights: Option<Tensor>,
    bias: Option<Tensor>,
    units: usize,
    activation: Option<Box<dyn Activation>>,
    trainable: bool,
    weights_grad: Option<Tensor>,
    bias_grad: Option<Tensor>,
    input: Option<Tensor>,
}

impl Dense {
    /// Creates a new dense layer.
    ///
    /// # Arguments
    ///
    /// * `units` - The number of output units.
    /// * `activation` - The activation function to use.
    /// * `trainable` - Whether the layer is trainable.
    pub fn new<A: Activation + 'static>(
        units: usize,
        activation: Option<A>,
        trainable: bool,
    ) -> Self {
        Dense {
            name: format!("dense_{}", units),
            weights: None,
            bias: None,
            units,
            activation: activation.map(|a| Box::new(a) as Box<dyn Activation>),
            trainable,
            weights_grad: None,
            bias_grad: None,
            input: None,
        }
    }
}

impl Layer for Dense {
    /// Builds the layer with the given input shape.
    ///
    /// # Arguments
    ///
    /// * `input_shape` - The shape of the input tensor.
    fn build(&mut self, input_shape: Shape) {
        debug!(
            "Building Dense layer with input shape: {:?} and units: {}",
            input_shape, self.units
        );
        let input_units = input_shape.0.last().expect("Input shape must not be empty");
        self.weights = Some(Tensor::random(vec![*input_units, self.units]));
        self.bias = Some(Tensor::zeros(vec![self.units]));
    }

    /// Performs a forward pass through the layer.
    ///
    /// # Arguments
    ///
    /// * `input` - The input tensor.
    ///
    /// # Returns
    ///
    /// The output tensor.
    fn forward(&mut self, input: &Tensor) -> Tensor {
        let weights = self.weights.as_ref().expect("Weights must be initialized");
        let bias = self.bias.as_ref().expect("Bias must be initialized");

        self.input = Some(input.clone());

        // Perform forward pass: Z = input · weights + bias
        let z = input.matmul(weights).add(bias);

        // Apply activation if present
        if let Some(ref activation) = self.activation {
            activation.activate(&z)
        } else {
            z
        }
    }

    /// Performs a backward pass through the layer.
    ///
    /// # Arguments
    ///
    /// * `grad` - The gradient tensor.
    ///
    /// # Returns
    ///
    /// The gradient tensor with respect to the input.
    fn backward(&mut self, grad: &Tensor) -> Tensor {
        // Ensure weights and input are initialized
        let weights = self.weights.as_ref().expect("Weights must be initialized");
        let input = self.input.as_ref().expect("Input must be initialized");

        // Calculate the gradient with respect to weights and bias
        let weights_grad = input.transpose().matmul(grad);
        let bias_grad = grad.sum_along_axis(0);

        // Store the gradients
        if self.trainable {
            self.weights_grad = Some(weights_grad);
            self.bias_grad = Some(bias_grad);
        }

        // Calculate the gradient with respect to the input
        let input_grad = grad.matmul(&weights.transpose());

        input_grad
    }

    /// Returns the output shape of the layer.
    ///
    /// # Returns
    ///
    /// A `Shape` representing the output shape of the layer.
    fn output_shape(&self) -> Shape {
        Shape::new(vec![self.units])
    }

    /// Returns the number of parameters in the layer.
    ///
    /// # Returns
    ///
    /// A `usize` representing the number of parameters in the layer.
    fn param_count(&self) -> (usize, usize) {
        let weights_count = self.weights.as_ref().map_or(0, |w| w.data.len());
        let bias_count = self.bias.as_ref().map_or(0, |b| b.data.len());
        (weights_count, bias_count)
    }

    /// Returns the name of the layer.
    ///
    /// # Returns
    ///
    /// A `&str` representing the name of the layer.
    fn name(&self) -> &str {
        &self.name
    }

    /// Updates the weights of the layer using the given gradient and optimizer.
    ///
    /// # Arguments
    ///
    /// * `grad` - The gradient tensor.
    /// * `optimizer` - The optimizer to use.
    fn update_weights(&mut self, optimizer: &mut Box<dyn Optimizer>) {
        if !self.trainable {
            return;
        }

        // Update weights
        if let Some(ref weights_grad) = self.weights_grad {
            optimizer.step(self.weights.as_mut().unwrap(), weights_grad);
        }

        if let Some(ref bias_grad) = self.bias_grad {
            optimizer.step(self.bias.as_mut().unwrap(), bias_grad);
        }

        // Clear gradients after update
        self.weights_grad = None;
        self.bias_grad = None;
    }

    fn get_weights(&self) -> serde_json::Value {
        serde_json::json!({
            "weights": self.weights.as_ref().map(|w| w.to_vec()),
            "bias": self.bias.as_ref().map(|b| b.to_vec())
        })
    }

    fn get_config(&self) -> serde_json::Value {
        serde_json::json!({
            "units": self.units,
            "trainable": self.trainable,
            "activation": self.activation.as_ref().map(|a| a.name())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activations::relu::ReluActivation;

    #[test]
    fn test_dense_layer() {
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]);
        let mut dense_layer = Dense::new(2, Some(ReluActivation::new()), true);
        dense_layer.build(Shape::new(vec![1, 3]));

        let output = dense_layer.forward(&input);

        assert_eq!(output.data.shape(), &[1, 2]);
        assert_eq!(output.data.len(), 2);
    }

    #[test]
    fn test_dense_layer_forward_pass() {
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]);
        let mut dense_layer = Dense::new(2, Some(ReluActivation::new()), true);
        dense_layer.build(Shape::new(vec![1, 3]));

        let output = dense_layer.forward(&input);

        assert_eq!(output.data.shape(), &[1, 2]);
        assert_eq!(output.data.len(), 2);
    }

    #[test]
    fn test_dense_layer_backward_pass() {
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]);
        let mut dense_layer = Dense::new(2, Some(ReluActivation::new()), true);
        dense_layer.input = Some(input.clone());
        dense_layer.build(Shape::new(vec![1, 3]));

        let grad = Tensor::new(vec![1.0, 2.0], vec![1, 2]);
        let output = dense_layer.backward(&grad);

        assert_eq!(output.data.shape(), &[1, 3]);
        assert_eq!(output.data.len(), 3);
    }

    #[test]
    fn test_dense_layer_initialization() {
        let dense_layer = Dense::new(5, None::<ReluActivation>, true);
        assert_eq!(dense_layer.units, 5);
        assert!(dense_layer.weights.is_none());
        assert!(dense_layer.bias.is_none());
    }

    #[test]
    fn test_dense_layer_with_no_activation() {
        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]);
        let mut dense_layer = Dense::new(4, None::<ReluActivation>, true);
        dense_layer.build(Shape::new(vec![1, 3]));

        let output = dense_layer.forward(&input);

        assert_eq!(output.data.len(), 4);
        // Verify that the output is computed without activation.
        // (Exact values depend on random weight initialization.)
    }

    #[test]
    fn test_dense_layer_output_shape() {
        let dense_layer = Dense::new(10, Some(ReluActivation::new()), true);
        assert_eq!(dense_layer.output_shape().0, vec![10]);
    }

    #[test]
    fn test_dense_layer_param_count() {
        let mut dense_layer = Dense::new(6, None::<ReluActivation>, true);
        dense_layer.build(Shape::new(vec![1, 4]));

        let (weights_count, bias_count) = dense_layer.param_count();
        assert_eq!(weights_count, 4 * 6); // 4 input units, 6 output units
        assert_eq!(bias_count, 6);
    }

    #[test]
    fn test_dense_layer_backward_with_no_trainable() {
        let mut dense_layer = Dense::new(4, None::<ReluActivation>, false);
        dense_layer.build(Shape::new(vec![1, 3]));

        let input = Tensor::new(vec![1.0, 2.0, 3.0], vec![1, 3]);
        dense_layer.input = Some(input);

        let grad = Tensor::new(vec![0.5, -0.5, 1.0, -1.0], vec![1, 4]);
        let output_grad = dense_layer.backward(&grad);

        // Ensure gradients are not stored when `trainable` is false.
        assert!(dense_layer.weights_grad.is_none());
        assert!(dense_layer.bias_grad.is_none());

        // Ensure output gradient is calculated.
        assert_eq!(output_grad.data.len(), 3);
    }

    #[test]
    fn test_dense_layer_with_zero_units() {
        let mut dense_layer = Dense::new(0, None::<ReluActivation>, true);
        dense_layer.build(Shape::new(vec![1, 3]));

        // Ensure the layer initializes with zero units without crashing.
        assert_eq!(dense_layer.output_shape().0, vec![0]);
        assert!(dense_layer.weights.is_some());
        assert!(dense_layer.bias.is_some());
    }

    #[test]
    fn test_dense_layer_with_large_input() {
        let input = Tensor::random(vec![1000, 512]); // Large input tensor
        let mut dense_layer = Dense::new(256, Some(ReluActivation::new()), true);
        dense_layer.build(Shape::new(vec![1000, 512]));

        let output = dense_layer.forward(&input);

        assert_eq!(output.data.shape(), &[1000, 256]);
        assert_eq!(output.data.len(), 1000 * 256);
    }
}
