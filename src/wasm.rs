use ndarray::Array1;

use crate::curve_fit;
use crate::func1d;
use crate::sas::*;
use crate::size_distribution;
use crate::standard;
use crate::utils::array1_to_vec;

use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Translates a string to a implemented function in rusfun to make them easily callable
pub fn get_function(function_name: &str) -> fn(&Array1<f64>, &Array1<f64>) -> Array1<f64> {
    match function_name {
        "linear" => standard::linear,
        "parabola" => standard::parabola,
        "sqrt" => standard::sqrt,
        "cos" => standard::cos,
        "sin" => standard::sin,
        "tan" => standard::tan,
        "exp" => standard::exp,
        "gaussian" => size_distribution::gaussian,
        "sas_sphere" => sphere::formfactor,
        "sas_cube" => cube::formfactor,
        _ => standard::zero,
    }
}

/// Interface function between input from wasm as Vec<f64> to crate with Array1<f64>
pub fn calculate_model(
    p: Vec<f64>,
    x: Vec<f64>,
    model: fn(&Array1<f64>, &Array1<f64>) -> Array1<f64>,
) -> Vec<f64> {
    array1_to_vec((model)(&Array1::from(p), &Array1::from(x)))
}

/// Calls Rust defined model functions by their function name for given parameters and a domain
#[wasm_bindgen]
pub fn model(function_name: &str, p: Vec<f64>, x: Vec<f64>) -> Vec<f64> {
    calculate_model(p, x, get_function(function_name))
}

/// The returned fit result from a fit() call
/// Fields need to be accessed in JS by their getter functions that have the same name
#[wasm_bindgen]
pub struct FitResult {
    parameters: Vec<f64>,
    parameter_std_errors: Vec<f64>,
    fitted_model: Vec<f64>,
    num_func_evaluation: usize,
    chi2: f64,
    redchi2: f64,
    R2: f64,
    convergence_message: String,
}

#[wasm_bindgen]
impl FitResult {
    pub fn parameters(&self) -> Vec<f64> {
        self.parameters.clone()
    }

    pub fn parameter_std_errors(&self) -> Vec<f64> {
        self.parameter_std_errors.clone()
    }

    pub fn fitted_model(&self) -> Vec<f64> {
        self.fitted_model.clone()
    }

    pub fn num_func_evaluation(&self) -> usize {
        self.num_func_evaluation
    }

    pub fn chi2(&self) -> f64 {
        self.chi2
    }

    pub fn redchi2(&self) -> f64 {
        self.redchi2
    }

    pub fn R2(&self) -> f64 {
        self.R2
    }

    pub fn convergence_message(&self) -> String {
        self.convergence_message.clone()
    }
}

/// Fit using the LM algorithm for model named model_name using initial
/// parameters p, data (x, y, sy), fitting only where there is a non-zero
/// value in vary_p
#[wasm_bindgen]
pub fn fit(
    model_name: &str,
    p: Vec<f64>,
    x: Vec<f64>,
    y: Vec<f64>,
    sy: Vec<f64>,
    vary_p: Vec<u8>,
) -> FitResult {
    let arr_p = Array1::from(p);
    let arr_x = Array1::from(x);
    let arr_y = Array1::from(y);
    let arr_sy = Array1::from(sy);
    let mut arr_vary_p: Vec<bool> = Vec::new();
    for entry in vary_p {
        arr_vary_p.push(entry > 0);
    }
    let arr_vary_p = Array1::from(arr_vary_p);

    let func = func1d::Func1D::new(&arr_p, &arr_x, get_function(model_name));
    let mut minimizer = curve_fit::Minimizer::init(&func, &arr_y, &arr_sy, &arr_vary_p, 0.01);
    minimizer.minimize();

    let R2 = minimizer.calculate_R2();

    FitResult {
        parameters: array1_to_vec(minimizer.minimizer_parameters),
        parameter_std_errors: array1_to_vec(minimizer.parameter_errors),
        num_func_evaluation: minimizer.num_func_evaluation,
        fitted_model: array1_to_vec(minimizer.minimizer_ymodel),
        chi2: minimizer.chi2,
        redchi2: minimizer.redchi2,
        R2,
        convergence_message: String::from(minimizer.convergence_message),
    }
}
