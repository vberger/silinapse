//! Constructions related to feed-forward networks

use std::cmp::min;

use num::{Float, one, zero};

use {Compute, BackpropTrain, SupervisedTrain};
use activations::ActivationFunction;
use training::{PerceptronRule, GradientDescent};

/// A feedforward layer
///
/// Such layer is composed of a set of output neurons, and have all its
/// inputs connected to all its outputs.
///
/// The effective computation is thus, if `X` is the vector of inputs,
/// `Y` the vector of outputs, `W` the internal weigths matrix, `B` the vector
/// of biases and `f()` the activation function (applied on all components of
/// the vector in parallel):
///
/// ```text
/// Y = f( W*X + B )
/// ```
///
/// The training of this layer consists on fitting the values of `W` and `B`.
pub struct FeedforwardLayer<F: Float, V: Fn(F) -> F, D: Fn(F) -> F> {
    inputs: usize,
    coeffs: Vec<F>,
    biases: Vec<F>,
    activation: ActivationFunction<F, V, D>
}

impl<F, V, D> FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    /// Creates a new linear feedforward layer with all its weights set
    /// to 1 and its biases set to 0
    pub fn new(inputs: usize,
               outputs: usize,
               activation: ActivationFunction<F, V, D>)
        -> FeedforwardLayer<F, V, D>
    {
        FeedforwardLayer {
            inputs: inputs,
            coeffs: vec![one(); inputs*outputs],
            biases: vec![zero(); outputs],
            activation: activation
        }
    }

    /// Creates a new linear feedforward layer with all its weights and biases
    /// generated by provided closure (for example a random number generator).
    pub fn new_from<G>(inputs: usize,
                       outputs: usize,
                       activation: ActivationFunction<F, V, D>,
                       mut generator: G)
        -> FeedforwardLayer<F, V, D>
        where G: FnMut() -> F
    {
        FeedforwardLayer {
            inputs: inputs,
            coeffs: (0..inputs*outputs).map(|_| generator()).collect(),
            biases: (0..outputs).map(|_| generator()).collect(),
            activation: activation
        }
    }
}

impl<F, V, D> Compute<F> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn compute(&self, input: &[F]) -> Vec<F> {
        let mut out = self.biases.clone();
        for j in 0..self.biases.len() {
            for i in 0..min(self.inputs, input.len()) {
                out[j] = out[j] + self.coeffs[j*self.inputs + i] * input[i]
            }
        }
        
        for o in &mut out {
            *o = (self.activation.value)(*o);
        }

        out
    }

    fn input_size(&self) -> usize {
        self.inputs
    }

    fn output_size(&self) -> usize {
        self.biases.len()
    }
}

impl<F, V, D> SupervisedTrain<F, PerceptronRule<F>> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn supervised_train(&mut self,
                        rule: &PerceptronRule<F>,
                        input: &[F],
                        target: &[F])
    {
        let out = self.compute(input);
        for j in 0..self.biases.len() {
            let diff = target.get(j).map(|v| *v).unwrap_or(zero()) - out[j];
            for i in 0..min(self.inputs, input.len()) {
                self.coeffs[i + j*self.inputs] =
                    self.coeffs[i + j*self.inputs] + rule.rate * diff * input[i];
            }
        }
    }
}

impl<F, V, D> BackpropTrain<F, GradientDescent<F>> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn backprop_train(&mut self,
                      rule: &GradientDescent<F>,
                      input: &[F],
                      target: &[F])
        -> Vec<F>
    {
        // we need to compute the intermediate states
        let mut out = self.biases.clone();
        for j in 0..self.biases.len() {
            for i in 0..min(self.inputs, input.len()) {
                out[j] = out[j] + self.coeffs[j*self.inputs + i] * input[i]
            }
        }

        let deltas = out.iter()
                            .map(|x| { (self.activation.derivative)(*x) })
                            .collect::<Vec<_>>();
        for o in &mut out {
            *o = (self.activation.value)(*o);
        }

        let mut returned = input.to_owned();
        for j in 0..self.biases.len() {
            for i in 0..min(self.inputs, input.len()) {
                returned[i] = returned[i] - self.coeffs[i + j*self.inputs]*deltas[j];
                self.coeffs[i + j*self.inputs] =
                    self.coeffs[i + j*self.inputs]
                    - rule.rate * input.get(i).map(|x| *x).unwrap_or(zero())
                                * deltas[j]
                                * ( out[j] - target.get(j).map(|x| *x).unwrap_or(zero()) )

            }
        }
        returned
    }
}

impl<F, V, D> SupervisedTrain<F, GradientDescent<F>> for FeedforwardLayer<F, V, D>
    where F: Float,
          V: Fn(F) -> F,
          D: Fn(F) -> F
{
    fn supervised_train(&mut self,
                        rule: &GradientDescent<F>,
                        input: &[F],
                        target: &[F])
    {
        self.backprop_train(rule, input, target);
    }
}

#[cfg(test)]
mod tests {
    use Compute;
    use activations::identity;
    use super::FeedforwardLayer;

    #[test]
    fn basics() {
        let layer = FeedforwardLayer::<f32, _, _>::new(7, 3, identity());
        assert_eq!(layer.input_size(), 7);
        assert_eq!(layer.output_size(), 3);
    }

    #[test]
    fn compute() {
        let layer = FeedforwardLayer::new_from(4, 2, identity(), || 0.5f32);
        let output = layer.compute(&[1.0, 1.0, 1.0, 1.0]);
        // all weigths and biases are 0.5, output should be 4*0.5 + 0.5 = 2.5
        for o in &output {
            assert!((o - 2.5).abs() < 0.00001);
        }
    }
}