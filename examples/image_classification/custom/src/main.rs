use deltaml::activations::ReluActivation;
use deltaml::activations::SoftmaxActivation;
use deltaml::common::shape::Shape;
use deltaml::dataset::base::ImageDatasetOps;
use deltaml::dataset::CustomImageDataset;
use deltaml::losses::MeanSquaredLoss;
use deltaml::neuralnet::Sequential;
use deltaml::neuralnet::{Dense, Flatten};
use deltaml::optimizers::Adam;

#[tokio::main]
async fn main() {
    // Create a neural network
    let mut model = Sequential::new()
        .add(Flatten::new(Shape::new(vec![28, 28]))) // Input: 28x28, Output: 784
        .add(Dense::new(128, Some(ReluActivation::new()), true)) // Input: 784, Output: 128
        .add(Dense::new(10, Some(SoftmaxActivation::new()), false)); // Output: 10 classes

    // Display the model summary
    model.summary();

    // Define an optimizer
    let optimizer = Adam::new(0.001);

    // Compile the model
    model.compile(optimizer, MeanSquaredLoss::new());

    // Loading the train and test dataset
    let mut train_data = CustomImageDataset::load_train_from_csv("examples/image_classification/custom/dataset/train.csv").await;
    let test_data = CustomImageDataset::load_test_from_csv("examples/image_classification/custom/dataset/val.csv").await;

    println!("Training the model...");
    println!("Train dataset size: {}", train_data.len());

    let epoch = 10;
    let batch_size = 32;

    model.fit(&mut train_data, epoch, batch_size);

    // Evaluate the model
    let accuracy = model.evaluate(&test_data, batch_size);
    println!("Test Accuracy: {:.2}%", accuracy * 100.0);

    // Save the model
    model.save("model_path").unwrap();
}